//! Fetch public YouTube captions in pure Rust (InnerTube + timedtext).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::langs::{language_name, translation_catalog};
use crate::parse::is_id;
use bytes::Bytes;
use futures_util::stream::{self, Stream, StreamExt};
use std::env;
use std::io;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::Stdio;

const VR_UA: &str = "com.google.android.apps.youtube.vr.oculus/1.60.19 (Linux; U; Android 12; eureka-user Build/SQ3A.220605.009.A1) gzip";
const IOS_UA: &str = "com.google.ios.youtube/20.11.6 (iPhone14,5; U; CPU iOS 18_5 like Mac OS X;)";
const ANDROID_UA: &str = "com.google.android.youtube/21.29.366 (Linux; U; Android 16; en_US; SM-S908E Build/TP1A.220624.014) gzip";
const WEB_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";
const PLAYER_URL: &str = "https://www.youtube.com/youtubei/v1/player?prettyPrint=false";
const TRANSCRIPT_URL: &str = "https://www.youtube.com/youtubei/v1/get_transcript?prettyPrint=false";
const WEB_CLIENT_VERSION: &str = "2.20260722.01.00";
/// Public InnerTube keys shipped in official YouTube clients.
const WEB_INNERTUBE_KEY: &str = "AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8";
const CACHE_TTL: Duration = Duration::from_secs(45 * 60);
const CACHE_CAP: usize = 256;

fn http_client(cookies: bool) -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(18))
        .connect_timeout(Duration::from_secs(8))
        .gzip(true)
        .cookie_store(cookies)
        .build()
        .expect("http client")
}

/// ANDROID_VR InnerTube — no cookie jar (mixing WEB cookies with a VR UA gets 429s).
static HTTP_VR: Lazy<reqwest::Client> = Lazy::new(|| http_client(false));
/// Watch-page fallback — browser cookies only.
static HTTP_WEB: Lazy<reqwest::Client> = Lazy::new(|| http_client(true));
/// googlevideo: HTTP/1, no keep-alive. HTTP/2 reuse hangs the second Range request.
static HTTP_GV: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(8))
        .gzip(true)
        .http1_only()
        .pool_max_idle_per_host(0)
        .tcp_nodelay(true)
        .build()
        .expect("gv http")
});

static CACHE: Lazy<Mutex<Vec<(String, Instant, Arc<TranscriptDoc>)>>> =
    Lazy::new(|| Mutex::new(Vec::new()));
static AUDIO_CACHE: Lazy<Mutex<Vec<(String, Instant, Vec<AudioPick>)>>> =
    Lazy::new(|| Mutex::new(Vec::new()));
/// Skip timedtext after a 429 so we do not make the ban worse.
static RATE_LIMIT_UNTIL: Lazy<Mutex<Option<Instant>>> = Lazy::new(|| Mutex::new(None));
const RATE_LIMIT_COOLDOWN: Duration = Duration::from_secs(45);
static SKIP_GTX: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
pub struct FetchError {
    pub status: u16,
    pub message: String,
}

impl FetchError {
    fn new(status: u16, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

#[resuma::data]
#[derive(Debug)]
pub struct Cue {
    pub start_ms: u64,
    pub duration_ms: u64,
    pub text: String,
}

#[resuma::data]
#[derive(Debug)]
pub struct CaptionTrack {
    pub lang: String,
    pub name: String,
    pub kind: String,
    pub translatable: bool,
}

#[resuma::data]
#[derive(Debug)]
pub struct Chapter {
    pub start_ms: u64,
    pub title: String,
}

#[resuma::data]
#[derive(Debug)]
pub struct LangOpt {
    pub code: String,
    pub name: String,
}

#[resuma::data]
#[derive(Debug)]
pub struct TranscriptDoc {
    pub video_id: String,
    pub title: String,
    pub author: String,
    pub channel_id: String,
    pub duration_secs: u64,
    pub track: CaptionTrack,
    pub tracks: Vec<CaptionTrack>,
    pub translations: Vec<LangOpt>,
    pub cues: Vec<Cue>,
    pub chapters: Vec<Chapter>,
}

impl TranscriptDoc {
    pub fn plain_text(&self) -> String {
        self.cues
            .iter()
            .map(|c| c.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn word_count(&self) -> usize {
        self.plain_text().split_whitespace().count()
    }
}

#[derive(Debug, Clone)]
pub struct AudioPick {
    pub video_id: String,
    pub title: String,
    pub mime: String,
    pub ext: String,
    pub url: String,
    pub bitrate: u64,
    pub ua: String,
}

#[resuma::data]
#[derive(Debug)]
pub struct AudioMeta {
    pub video_id: String,
    pub title: String,
    pub ext: String,
    pub bitrate: u64,
}

impl AudioPick {
    pub fn meta(&self) -> AudioMeta {
        AudioMeta {
            video_id: self.video_id.clone(),
            title: self.title.clone(),
            ext: self.ext.clone(),
            bitrate: self.bitrate,
        }
    }
}

#[derive(Clone)]
struct RemoteTrack {
    lang: String,
    name: String,
    kind: String,
    translatable: bool,
    base_url: String,
}

pub fn track_key(lang: &str, kind: &str) -> String {
    if kind == "asr" {
        format!("{lang}:asr")
    } else {
        lang.to_string()
    }
}

/// `es:asr` (form values) or legacy `es|asr` query strings.
fn split_lang_want(w: &str) -> (&str, bool) {
    let w = w.trim();
    if let Some(code) = w.strip_suffix("|asr").or_else(|| w.strip_suffix(":asr")) {
        (code, true)
    } else {
        (w, false)
    }
}

pub async fn load_transcript(
    video_id: &str,
    lang: Option<&str>,
    tlang: Option<&str>,
) -> Result<Arc<TranscriptDoc>, FetchError> {
    if !is_id(video_id) {
        return Err(FetchError::new(400, "That is not a YouTube video id."));
    }
    let lang_key = lang.unwrap_or("").to_ascii_lowercase();
    let tlang_key = tlang.unwrap_or("").to_ascii_lowercase();
    let cache_key = format!("{video_id}|{lang_key}|{tlang_key}");

    if let Some(hit) = cache_get(&cache_key) {
        return Ok(hit);
    }

    // Already have this video’s source captions (same Fly instance): translate
    // those lines instead of hitting YouTube timedtext again (often 429).
    if !tlang_key.is_empty() {
        if let Some(src) = cache_get_source(video_id, &lang_key) {
            if let Ok(doc) = translated_from_source(src.as_ref(), &tlang_key).await {
                let doc = Arc::new(doc);
                cache_put(cache_key, doc.clone());
                return Ok(doc);
            }
        }
    }

    match fetch_uncached(video_id, lang, tlang).await {
        Ok(doc) => {
            let doc = Arc::new(doc);
            cache_put(cache_key, doc.clone());
            Ok(doc)
        }
        Err(e) if !tlang_key.is_empty() => {
            let src = if let Some(cached) = cache_get_source(video_id, &lang_key) {
                cached
            } else {
                Arc::new(fetch_uncached(video_id, lang, None).await?)
            };
            let doc = translated_from_source(src.as_ref(), &tlang_key)
                .await
                .map_err(|_| e)?;
            let doc = Arc::new(doc);
            cache_put(cache_key, doc.clone());
            Ok(doc)
        }
        Err(e) => Err(e),
    }
}

async fn fetch_uncached(
    video_id: &str,
    lang: Option<&str>,
    tlang: Option<&str>,
) -> Result<TranscriptDoc, FetchError> {
    let (meta, tracks, chapters) = player_bundle(video_id).await?;
    if tracks.is_empty() {
        return Err(FetchError::new(
            404,
            "This video has no captions. Try another video, or one with auto-captions turned on.",
        ));
    }

    let ordered = tracks_for_fetch(&tracks, lang);
    if ordered.is_empty() {
        return Err(FetchError::new(404, "No caption track matches that language."));
    }

    let mut chosen = ordered[0];
    let mut body: Option<String> = None;
    let mut last_err: Option<FetchError> = None;
    for track in &ordered {
        if needs_pot(&track.base_url) {
            continue;
        }
        let timed_url = caption_url(
            &track.base_url,
            tlang.filter(|s| !s.eq_ignore_ascii_case(&track.lang)),
        );
        match fetch_timedtext(&timed_url).await {
            Ok(b) => {
                chosen = *track;
                body = Some(b);
                break;
            }
            Err(e) if e.status == 429 => {
                *RATE_LIMIT_UNTIL.lock() = Some(Instant::now() + RATE_LIMIT_COOLDOWN);
                return Err(e);
            }
            Err(e) => last_err = Some(e),
        }
    }

    let mut cues = if let Some(raw) = body {
        parse_captions(&raw)
    } else {
        Vec::new()
    };

    if cues.is_empty() {
        let lang_code = lang
            .map(split_lang_want)
            .map(|(code, _)| code.to_string())
            .unwrap_or_else(|| chosen.lang.clone());
        let asr = chosen.kind == "asr" || lang.is_some_and(|s| split_lang_want(s).1);
        if let Ok(from_panel) = innertube_transcript(video_id, &lang_code, asr).await {
            if !from_panel.is_empty() {
                cues = from_panel;
            }
        }
        if cues.is_empty() {
            if let Ok(from_panel) = innertube_transcript(video_id, &lang_code, !asr).await {
                cues = from_panel;
            }
        }
    }

    if cues.is_empty() {
        return Err(last_err.unwrap_or_else(|| {
            FetchError::new(
                502,
                "YouTube sent empty captions (this video may require a browser token).",
            )
        }));
    }
    cues.sort_by_key(|c| c.start_ms);
    stitch_durations(&mut cues);
    collapse_repeat_cues(&mut cues);

    let track = CaptionTrack {
        lang: chosen.lang.clone(),
        name: display_track_name(chosen),
        kind: chosen.kind.clone(),
        translatable: chosen.translatable,
    };
    let track_list = tracks
        .iter()
        .map(|t| CaptionTrack {
            lang: t.lang.clone(),
            name: display_track_name(t),
            kind: t.kind.clone(),
            translatable: t.translatable,
        })
        .collect();

    Ok(TranscriptDoc {
        video_id: video_id.to_string(),
        title: meta.title,
        author: meta.author,
        channel_id: meta.channel_id,
        duration_secs: meta.duration_secs,
        track,
        tracks: track_list,
        translations: translation_catalog()
            .into_iter()
            .map(|(code, name)| LangOpt { code, name })
            .collect(),
        cues,
        chapters,
    })
}

struct Meta {
    title: String,
    author: String,
    channel_id: String,
    duration_secs: u64,
}

struct TubeClient {
    name: &'static str,
    version: &'static str,
    ua: &'static str,
    client_header: &'static str,
    api_key: Option<&'static str>,
    extra: Value,
}

fn listing_clients() -> [TubeClient; 4] {
    [
        // ANDROID first: its timedtext URLs usually omit exp=xpe (no PoToken).
        TubeClient {
            name: "ANDROID",
            version: "21.29.366",
            ua: ANDROID_UA,
            client_header: "3",
            api_key: None,
            extra: json!({
                "androidSdkVersion": 33,
                "osName": "Android",
                "osVersion": "16",
                "platform": "MOBILE"
            }),
        },
        TubeClient {
            name: "IOS",
            version: "20.11.6",
            ua: IOS_UA,
            client_header: "5",
            api_key: Some(WEB_INNERTUBE_KEY),
            extra: json!({
                "deviceMake": "Apple",
                "deviceModel": "iPhone14,5",
                "osName": "iPhone",
                "osVersion": "18.5.0.22F76"
            }),
        },
        TubeClient {
            name: "TVHTML5",
            version: WEB_CLIENT_VERSION,
            ua: WEB_UA,
            client_header: "7",
            api_key: Some(WEB_INNERTUBE_KEY),
            extra: json!({}),
        },
        TubeClient {
            name: "ANDROID_VR",
            version: "1.60.19",
            ua: VR_UA,
            client_header: "28",
            api_key: None,
            extra: json!({
                "deviceMake": "Oculus",
                "deviceModel": "Quest 3",
                "androidSdkVersion": 32,
                "osName": "Android",
                "osVersion": "12"
            }),
        },
    ]
}

async fn player_bundle(
    video_id: &str,
) -> Result<(Meta, Vec<RemoteTrack>, Vec<Chapter>), FetchError> {
    let mut meta: Option<Meta> = None;
    let mut tracks: Vec<RemoteTrack> = Vec::new();
    let mut chapters: Vec<Chapter> = Vec::new();
    let mut audios: Vec<RawAudio> = Vec::new();

    for hl in ["en", "es"] {
        for client in listing_clients() {
            if let Ok(player) = innertube_player(video_id, &client, hl).await {
                if let Some((m, t, ch)) = extract_bundle(&player) {
                    if meta.is_none() {
                        meta = Some(m);
                    }
                    merge_tracks(&mut tracks, t);
                    if chapters.is_empty() {
                        chapters = ch;
                    }
                }
                merge_audio(&mut audios, extract_audio(&player, client.ua));
            }
        }
        if let Ok(player) = watch_page_player(video_id, hl).await {
            if let Some((m, t, ch)) = extract_bundle(&player) {
                if meta.is_none() {
                    meta = Some(m);
                }
                merge_tracks(&mut tracks, t);
                if chapters.is_empty() {
                    chapters = ch;
                }
            }
            merge_audio(&mut audios, extract_audio(&player, WEB_UA));
        }
        if tracks.iter().any(|t| !needs_pot(&t.base_url)) && best_audio(&audios).is_some() {
            break;
        }
    }

    let Some(meta) = meta else {
        return Err(FetchError::new(
            404,
            "Could not read this video. It may be private, age-restricted, or removed.",
        ));
    };
    if !audios.is_empty() {
        cache_audios(video_id, &meta.title, audios);
    }
    Ok((meta, tracks, chapters))
}

async fn innertube_player(
    video_id: &str,
    client: &TubeClient,
    hl: &str,
) -> Result<Value, FetchError> {
    let mut client_obj = json!({
        "clientName": client.name,
        "clientVersion": client.version,
        "hl": hl,
        "gl": "US",
        "userAgent": client.ua
    });
    if let Some(map) = client_obj.as_object_mut() {
        if let Some(extra) = client.extra.as_object() {
            for (k, v) in extra {
                map.insert(k.clone(), v.clone());
            }
        }
    }
    let payload = json!({
        "context": { "client": client_obj },
        "videoId": video_id,
        "contentCheckOk": true,
        "racyCheckOk": true
    });
    let url = match client.api_key {
        Some(key) => format!("{PLAYER_URL}&key={key}"),
        None => PLAYER_URL.to_string(),
    };
    let resp = HTTP_VR
        .post(&url)
        .header("User-Agent", client.ua)
        .header("Content-Type", "application/json")
        .header("Origin", "https://www.youtube.com")
        .header("X-YouTube-Client-Name", client.client_header)
        .header("X-YouTube-Client-Version", client.version)
        .json(&payload)
        .send()
        .await
        .map_err(|e| FetchError::new(502, format!("Could not reach YouTube ({e}).")))?;
    let status = resp.status().as_u16();
    let val: Value = resp
        .json()
        .await
        .map_err(|_| FetchError::new(502, "YouTube sent a response we could not read."))?;
    if status >= 400 || val.get("error").is_some() {
        return Err(FetchError::new(status.max(400), "YouTube player request failed."));
    }
    let play = val
        .pointer("/playabilityStatus/status")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if play == "LOGIN_REQUIRED" || play == "UNPLAYABLE" || play == "ERROR" {
        if val.get("videoDetails").is_none() {
            return Err(FetchError::new(
                404,
                "Could not read this video. It may be private, age-restricted, or removed.",
            ));
        }
    }
    Ok(val)
}

async fn watch_page_player(video_id: &str, hl: &str) -> Result<Value, FetchError> {
    let url = format!(
        "https://www.youtube.com/watch?v={video_id}&hl={hl}&bpctr=9999999999&has_verified=1"
    );
    let html = HTTP_WEB
        .get(url)
        .header("User-Agent", WEB_UA)
        .header("Accept-Language", "en-US,en;q=0.9,es;q=0.8")
        .header("Cookie", "CONSENT=YES+; SOCS=CAI")
        .send()
        .await
        .map_err(|e| FetchError::new(502, format!("Could not open the watch page ({e}).")))?
        .text()
        .await
        .map_err(|_| FetchError::new(502, "Could not read the watch page."))?;
    extract_player_json(&html).ok_or_else(|| {
        FetchError::new(
            404,
            "Could not find captions on the watch page. YouTube may be blocking this network.",
        )
    })
}

fn extract_player_json(html: &str) -> Option<Value> {
    for marker in ["ytInitialPlayerResponse", "ytInitialData"] {
        for v in json_after_all(html, marker) {
            if v.get("videoDetails").is_some() || v.get("captions").is_some() {
                return Some(v);
            }
        }
    }
    None
}

fn json_after_all(hay: &str, marker: &str) -> Vec<Value> {
    let mut out = Vec::new();
    let mut from = 0usize;
    while let Some(rel) = hay[from..].find(marker) {
        let i = from + rel;
        let rest = hay[i + marker.len()..].trim_start();
        let rest = rest.trim_start_matches('=').trim_start();
        let mut de = serde_json::Deserializer::from_str(rest);
        if let Ok(v) = Deserialize::deserialize(&mut de) {
            out.push(v);
        }
        from = i + marker.len();
        if out.len() >= 3 {
            break;
        }
    }
    out
}

fn extract_bundle(player: &Value) -> Option<(Meta, Vec<RemoteTrack>, Vec<Chapter>)> {
    let details = player.get("videoDetails")?;
    let mut title = details
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if title.is_empty() {
        title = details
            .get("videoId")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
    }
    if title.is_empty() {
        return None;
    }
    let meta = Meta {
        title,
        author: details
            .get("author")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        channel_id: details
            .get("channelId")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        duration_secs: details
            .get("lengthSeconds")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .or_else(|| details.get("lengthSeconds").and_then(|v| v.as_u64()))
            .unwrap_or(0),
    };
    let tracks = caption_tracks(player);
    let chapters = chapters_from(player);
    Some((meta, tracks, chapters))
}

fn caption_tracks(player: &Value) -> Vec<RemoteTrack> {
    let mut out = Vec::new();
    collect_caption_tracks(player, &mut out);
    dedupe_tracks(out)
}

fn collect_caption_tracks(v: &Value, out: &mut Vec<RemoteTrack>) {
    match v {
        Value::Array(arr) => {
            let looks_like_tracks = arr.iter().any(|item| {
                item.get("baseUrl")
                    .and_then(|u| u.as_str())
                    .is_some_and(|s| s.contains("timedtext") || s.contains("tts"))
                    || item.get("languageCode").is_some() && item.get("baseUrl").is_some()
            });
            if looks_like_tracks {
                for item in arr {
                    if let Some(t) = track_from_item(item) {
                        out.push(t);
                    }
                }
                return;
            }
            for x in arr {
                collect_caption_tracks(x, out);
            }
        }
        Value::Object(map) => {
            if let Some(arr) = map.get("captionTracks").and_then(|x| x.as_array()) {
                for item in arr {
                    if let Some(t) = track_from_item(item) {
                        out.push(t);
                    }
                }
            }
            for x in map.values() {
                collect_caption_tracks(x, out);
            }
        }
        _ => {}
    }
}

fn track_from_item(item: &Value) -> Option<RemoteTrack> {
    let base_url = item
        .get("baseUrl")
        .or_else(|| item.get("url"))
        .and_then(|v| v.as_str())
        .filter(|u| !u.is_empty())
        .map(absolutize_caption_url)?;
    let lang = item
        .get("languageCode")
        .and_then(|v| v.as_str())
        .unwrap_or("und")
        .to_string();
    let name = json_text(item.get("name")).unwrap_or_else(|| language_name(&lang));
    let kind = item
        .get("kind")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let translatable = item
        .get("isTranslatable")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    Some(RemoteTrack {
        lang,
        name,
        kind,
        translatable,
        base_url,
    })
}

fn needs_pot(url: &str) -> bool {
    url.contains("exp=xpe") || url.contains("exp=xp")
}

fn caption_url_rank(url: &str) -> u8 {
    if needs_pot(url) {
        0
    } else if url.contains("fmt=srv3") {
        2
    } else {
        1
    }
}

fn absolutize_caption_url(url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else if url.starts_with('/') {
        format!("https://www.youtube.com{url}")
    } else {
        format!("https://www.youtube.com/{url}")
    }
}

fn tracks_for_fetch<'a>(tracks: &'a [RemoteTrack], lang: Option<&str>) -> Vec<&'a RemoteTrack> {
    if lang.is_some() && pick_track(tracks, lang).is_none() {
        return Vec::new();
    }
    let mut out: Vec<&RemoteTrack> = tracks.iter().collect();
    out.sort_by(|a, b| {
        caption_url_rank(&b.base_url)
            .cmp(&caption_url_rank(&a.base_url))
            .then_with(|| track_lang_rank(b, lang).cmp(&track_lang_rank(a, lang)))
    });
    out
}

fn track_lang_rank(t: &RemoteTrack, lang: Option<&str>) -> u8 {
    let wanted = lang.map(|s| s.trim()).filter(|s| !s.is_empty());
    if let Some(w) = wanted {
        let (code, asr) = split_lang_want(w);
        let lang_hit = t.lang.eq_ignore_ascii_case(code)
            || t.lang
                .split(['-', '_'])
                .next()
                .unwrap_or("")
                .eq_ignore_ascii_case(code.split(['-', '_']).next().unwrap_or(""));
        match (lang_hit, asr, t.kind == "asr") {
            (true, true, true) => 6,
            (true, false, false) => 6,
            (true, false, true) => 5,
            (true, true, false) => 5,
            _ => 0,
        }
    } else {
        let en = t.lang.to_ascii_lowercase().starts_with("en");
        match (t.kind == "asr", en) {
            (false, true) => 4,
            (false, false) => 3,
            (true, true) => 2,
            (true, false) => 1,
        }
    }
}

fn merge_tracks(into: &mut Vec<RemoteTrack>, extra: Vec<RemoteTrack>) {
    for t in extra {
        if let Some(existing) = into
            .iter_mut()
            .find(|e| e.lang.eq_ignore_ascii_case(&t.lang) && e.kind == t.kind)
        {
            if caption_url_rank(&t.base_url) > caption_url_rank(&existing.base_url) {
                *existing = t;
            }
        } else {
            into.push(t);
        }
    }
}

fn dedupe_tracks(tracks: Vec<RemoteTrack>) -> Vec<RemoteTrack> {
    let mut out = Vec::new();
    merge_tracks(&mut out, tracks);
    out
}

fn json_text(v: Option<&Value>) -> Option<String> {
    let v = v?;
    if let Some(s) = v.as_str() {
        return Some(s.to_string());
    }
    if let Some(s) = v.get("simpleText").and_then(|x| x.as_str()) {
        return Some(s.to_string());
    }
    if let Some(runs) = v.get("runs").and_then(|x| x.as_array()) {
        let s: String = runs
            .iter()
            .filter_map(|r| r.get("text").and_then(|t| t.as_str()))
            .collect();
        if !s.is_empty() {
            return Some(s);
        }
    }
    None
}

fn pick_track<'a>(tracks: &'a [RemoteTrack], lang: Option<&str>) -> Option<&'a RemoteTrack> {
    let wanted = lang.map(|s| s.trim()).filter(|s| !s.is_empty());
    if let Some(w) = wanted {
        let (code, asr) = split_lang_want(w);
        if asr {
            return tracks
                .iter()
                .find(|t| t.kind == "asr" && t.lang.eq_ignore_ascii_case(code))
                .or_else(|| tracks.iter().find(|t| t.lang.eq_ignore_ascii_case(code)));
        }
        tracks
            .iter()
            .find(|t| t.lang.eq_ignore_ascii_case(code) && t.kind != "asr")
            .or_else(|| tracks.iter().find(|t| t.lang.eq_ignore_ascii_case(code)))
            .or_else(|| {
                tracks.iter().find(|t| {
                    t.lang
                        .split(['-', '_'])
                        .next()
                        .unwrap_or("")
                        .eq_ignore_ascii_case(code.split(['-', '_']).next().unwrap_or(""))
                })
            })
    } else {
        tracks
            .iter()
            .find(|t| t.kind != "asr" && t.lang.to_ascii_lowercase().starts_with("en"))
            .or_else(|| tracks.iter().find(|t| t.kind != "asr"))
            .or_else(|| tracks.iter().find(|t| t.lang.to_ascii_lowercase().starts_with("en")))
            .or_else(|| tracks.first())
    }
}

fn display_track_name(t: &RemoteTrack) -> String {
    let mut name = if t.name.is_empty() {
        language_name(&t.lang)
    } else {
        t.name.clone()
    };
    if t.kind == "asr" && !name.to_ascii_lowercase().contains("auto") {
        name.push_str(" (auto)");
    }
    name
}

fn caption_url(base: &str, tlang: Option<&str>) -> String {
    let mut url = set_query(base, "fmt", "json3");
    if let Some(tl) = tlang.filter(|s| !s.is_empty()) {
        url = set_query(&url, "tlang", tl);
    }
    url
}

/// Patch one query param without re-encoding the rest.
/// Rebuilding via `Url` percent-encodes `sparams=ip,ipbits` and breaks YouTube signatures.
fn set_query(url: &str, key: &str, value: &str) -> String {
    let amp = format!("&{key}=");
    let qst = format!("?{key}=");
    let start = url.find(&amp).or_else(|| url.find(&qst));
    if let Some(i) = start {
        let val_at = i + amp.len();
        let end = url[val_at..].find('&').map(|j| val_at + j).unwrap_or(url.len());
        let mut out = String::with_capacity(url.len() + value.len());
        out.push_str(&url[..val_at]);
        out.push_str(value);
        out.push_str(&url[end..]);
        out
    } else {
        let sep = if url.contains('?') { '&' } else { '?' };
        format!("{url}{sep}{key}={value}")
    }
}

async fn fetch_timedtext(url: &str) -> Result<String, FetchError> {
    if let Some(until) = *RATE_LIMIT_UNTIL.lock() {
        if Instant::now() < until {
            return Err(FetchError::new(
                429,
                "YouTube is rate-limiting caption downloads from this network. Try again in a minute.",
            ));
        }
    }
    let formats = ["json3", "srv3", "vtt"];
    let agents: [(&reqwest::Client, &str); 3] = [
        (&*HTTP_VR, ANDROID_UA),
        (&*HTTP_VR, IOS_UA),
        (&*HTTP_WEB, WEB_UA),
    ];
    let mut last_err: Option<FetchError> = None;
    for fmt in formats {
        let u = set_query(url, "fmt", fmt);
        for (client, ua) in agents {
            match timedtext_once(client, ua, &u).await {
                Ok(body) if usable_captions(&body) => return Ok(body),
                Err(e) if e.status == 429 => return Err(e),
                Ok(_) => {
                    last_err = Some(FetchError::new(
                        502,
                        "YouTube sent empty captions (this video may require a browser token).",
                    ));
                }
                Err(e) => last_err = Some(e),
            }
        }
    }
    Err(last_err.unwrap_or_else(|| {
        FetchError::new(
            502,
            "YouTube sent empty captions (this video may require a browser token).",
        )
    }))
}

fn usable_captions(body: &str) -> bool {
    !body.trim().is_empty() && !parse_captions(body).is_empty()
}

async fn timedtext_once(
    client: &reqwest::Client,
    ua: &str,
    url: &str,
) -> Result<String, FetchError> {
    let resp = client
        .get(url)
        .header("User-Agent", ua)
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Origin", "https://www.youtube.com")
        .header("Referer", "https://www.youtube.com/")
        .send()
        .await
        .map_err(|e| FetchError::new(502, format!("Caption request failed ({e}).")))?;
    let status = resp.status().as_u16();
    if status == 429 {
        return Err(FetchError::new(
            429,
            "YouTube is rate-limiting caption downloads from this network. Try again in a minute.",
        ));
    }
    if status >= 400 {
        return Err(FetchError::new(status, "YouTube refused the caption file."));
    }
    let body = resp
        .text()
        .await
        .map_err(|_| FetchError::new(502, "Caption file could not be read."))?;
    Ok(body)
}

async fn innertube_transcript(
    video_id: &str,
    lang: &str,
    asr: bool,
) -> Result<Vec<Cue>, FetchError> {
    let params = transcript_params(video_id, lang, asr);
    let payload = json!({
        "context": {
            "client": {
                "clientName": "WEB",
                "clientVersion": WEB_CLIENT_VERSION,
                "hl": lang,
                "gl": "US"
            }
        },
        "params": params
    });
    let resp = HTTP_WEB
        .post(TRANSCRIPT_URL)
        .header("User-Agent", WEB_UA)
        .header("Content-Type", "application/json")
        .header("Origin", "https://www.youtube.com")
        .header("X-YouTube-Client-Name", "1")
        .header("X-YouTube-Client-Version", WEB_CLIENT_VERSION)
        .json(&payload)
        .send()
        .await
        .map_err(|e| FetchError::new(502, format!("Could not reach YouTube transcript ({e}).")))?;
    if !resp.status().is_success() {
        return Err(FetchError::new(
            resp.status().as_u16(),
            "YouTube transcript panel request failed.",
        ));
    }
    let val: Value = resp
        .json()
        .await
        .map_err(|_| FetchError::new(502, "YouTube transcript panel was not JSON."))?;
    if val.get("error").is_some() {
        return Err(FetchError::new(400, "YouTube transcript panel request failed."));
    }
    let cues = parse_transcript_panel(&val);
    if cues.is_empty() {
        return Err(FetchError::new(404, "Transcript panel had no lines."));
    }
    Ok(cues)
}

fn transcript_params(video_id: &str, lang: &str, asr: bool) -> String {
    let kind = if asr { "asr" } else { "" };
    let mut inner = pb_string(1, kind);
    inner.extend(pb_string(2, lang));
    inner.extend(pb_string(3, ""));
    let mut msg = pb_string(1, video_id);
    msg.extend(pb_bytes(2, &inner));
    msg.extend(pb_varint_field(3, 1));
    msg.extend(pb_string(
        5,
        "engagement-panel-searchable-transcript-search-panel",
    ));
    msg.extend(pb_varint_field(6, 1));
    msg.extend(pb_varint_field(7, 1));
    msg.extend(pb_varint_field(8, 1));
    b64url_nopad(&msg)
}

fn pb_varint(mut n: u64) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let mut b = (n & 0x7f) as u8;
        n >>= 7;
        if n != 0 {
            b |= 0x80;
        }
        out.push(b);
        if n == 0 {
            break;
        }
    }
    out
}

fn pb_string(field: u32, s: &str) -> Vec<u8> {
    pb_bytes(field, s.as_bytes())
}

fn pb_bytes(field: u32, data: &[u8]) -> Vec<u8> {
    let mut out = pb_varint(u64::from(field) << 3 | 2);
    out.extend(pb_varint(data.len() as u64));
    out.extend_from_slice(data);
    out
}

fn pb_varint_field(field: u32, n: u64) -> Vec<u8> {
    let mut out = pb_varint(u64::from(field) << 3);
    out.extend(pb_varint(n));
    out
}

fn b64url_nopad(input: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    let mut i = 0;
    while i < input.len() {
        let b0 = input[i];
        let b1 = if i + 1 < input.len() { input[i + 1] } else { 0 };
        let b2 = if i + 2 < input.len() { input[i + 2] } else { 0 };
        let triple = u32::from(b0) << 16 | u32::from(b1) << 8 | u32::from(b2);
        out.push(A[((triple >> 18) & 63) as usize] as char);
        out.push(A[((triple >> 12) & 63) as usize] as char);
        if i + 1 < input.len() {
            out.push(A[((triple >> 6) & 63) as usize] as char);
        }
        if i + 2 < input.len() {
            out.push(A[(triple & 63) as usize] as char);
        }
        i += 3;
    }
    out
}

fn parse_transcript_panel(v: &Value) -> Vec<Cue> {
    let mut out = Vec::new();
    collect_transcript_cues(v, &mut out);
    out
}

fn collect_transcript_cues(v: &Value, out: &mut Vec<Cue>) {
    match v {
        Value::Array(arr) => {
            for x in arr {
                collect_transcript_cues(x, out);
            }
        }
        Value::Object(map) => {
            if let Some(seg) = map.get("transcriptSegmentRenderer") {
                if let Some(cue) = cue_from_segment(seg) {
                    out.push(cue);
                }
                return;
            }
            for x in map.values() {
                collect_transcript_cues(x, out);
            }
        }
        _ => {}
    }
}

fn cue_from_segment(seg: &Value) -> Option<Cue> {
    let start_ms = json_u64(seg.get("startMs"));
    let end_ms = json_u64(seg.get("endMs"));
    let text = json_text(seg.get("snippet")).unwrap_or_default();
    let text = normalize_caption(&text);
    if text.is_empty() {
        return None;
    }
    Some(Cue {
        start_ms,
        duration_ms: end_ms.saturating_sub(start_ms),
        text,
    })
}

pub fn parse_captions(body: &str) -> Vec<Cue> {
    let trimmed = body.trim_start();
    if trimmed.starts_with('{') {
        if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
            return parse_json3(&v);
        }
        return Vec::new();
    }
    if trimmed.starts_with("WEBVTT") {
        return parse_vtt(trimmed);
    }
    if is_caption_xml(trimmed) {
        return parse_srv3(trimmed);
    }
    Vec::new()
}

fn is_caption_xml(s: &str) -> bool {
    let head = s.get(..800).unwrap_or(s).to_ascii_lowercase();
    head.contains("<timedtext")
        || head.contains("<transcript")
        || head.contains("<text start")
        || head.contains("<text t=")
}

fn parse_json3(v: &Value) -> Vec<Cue> {
    let mut out = Vec::new();
    let Some(events) = v.get("events").and_then(|e| e.as_array()) else {
        return out;
    };
    for ev in events {
        let segs = match ev.get("segs").and_then(|s| s.as_array()) {
            Some(s) => s,
            None => continue,
        };
        let mut text = String::new();
        for seg in segs {
            if let Some(u) = seg.get("utf8").and_then(|x| x.as_str()) {
                text.push_str(u);
            }
        }
        let text = normalize_caption(&text);
        if text.is_empty() {
            continue;
        }
        let start_ms = json_u64(ev.get("tStartMs"));
        let duration_ms = json_u64(ev.get("dDurationMs"));
        out.push(Cue {
            start_ms,
            duration_ms,
            text,
        });
    }
    out
}

fn parse_vtt(body: &str) -> Vec<Cue> {
    let mut out = Vec::new();
    let mut lines = body.lines().peekable();
    while let Some(line) = lines.next() {
        if !line.contains("-->") {
            continue;
        }
        let mut parts = line.split("-->");
        let start = parse_vtt_time(parts.next().unwrap_or("").trim());
        let end = parse_vtt_time(parts.next().unwrap_or("").trim());
        let mut text = String::new();
        while let Some(next) = lines.peek() {
            if next.trim().is_empty() || next.contains("-->") {
                break;
            }
            let n = lines.next().unwrap();
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(n.trim());
        }
        let text = normalize_caption(&text);
        if text.is_empty() {
            continue;
        }
        out.push(Cue {
            start_ms: start,
            duration_ms: end.saturating_sub(start),
            text,
        });
    }
    out
}

fn parse_vtt_time(s: &str) -> u64 {
    let s = s.split_whitespace().next().unwrap_or(s);
    let s = s.replace(',', ".");
    let mut h = 0u64;
    let mut m = 0u64;
    let rest;
    let bits: Vec<&str> = s.split(':').collect();
    match bits.len() {
        3 => {
            h = bits[0].parse().unwrap_or(0);
            m = bits[1].parse().unwrap_or(0);
            rest = bits[2];
        }
        2 => {
            m = bits[0].parse().unwrap_or(0);
            rest = bits[1];
        }
        _ => rest = bits.first().copied().unwrap_or("0"),
    }
    let mut sec_parts = rest.split('.');
    let sec: u64 = sec_parts.next().unwrap_or("0").parse().unwrap_or(0);
    let mut frac = sec_parts.next().unwrap_or("0").to_string();
    frac.truncate(3);
    while frac.len() < 3 {
        frac.push('0');
    }
    let ms: u64 = frac.parse().unwrap_or(0);
    h * 3_600_000 + m * 60_000 + sec * 1000 + ms
}

fn parse_srv3(body: &str) -> Vec<Cue> {
    let mut out = Vec::new();
    for (tag, start_attr, dur_attr) in [("p", "t", "d"), ("text", "start", "dur")] {
        let open = format!("<{tag}");
        let mut idx = 0usize;
        let bytes = body.as_bytes();
        while let Some(rel) = body[idx..].find(&open) {
            let start_i = idx + rel;
            let Some(tag_end) = body[start_i..].find('>') else {
                break;
            };
            let head = &body[start_i..start_i + tag_end];
            let content_start = start_i + tag_end + 1;
            let close = format!("</{tag}>");
            let Some(rel_close) = body[content_start..].find(&close) else {
                idx = content_start;
                continue;
            };
            let raw = &body[content_start..content_start + rel_close];
            idx = content_start + rel_close + close.len();
            let text = normalize_caption(&strip_tags(raw));
            if text.is_empty() {
                continue;
            }
            let start = attr_num(head, start_attr);
            let mut dur = attr_num(head, dur_attr);
            // `text start` is often seconds as float; `p t` is ms.
            let start_ms = if tag == "text" && !head.contains("t=") {
                (start * 1000.0) as u64
            } else {
                start as u64
            };
            if tag == "text" && dur < 50.0 {
                dur *= 1000.0;
            }
            out.push(Cue {
                start_ms,
                duration_ms: dur.max(0.0) as u64,
                text,
            });
        }
        if !out.is_empty() {
            return out;
        }
        let _ = bytes;
    }
    out
}

fn attr_num(head: &str, name: &str) -> f64 {
    let key = format!("{name}=\"");
    if let Some(i) = head.find(&key) {
        let rest = &head[i + key.len()..];
        let end = rest.find('"').unwrap_or(rest.len());
        return rest[..end].parse().unwrap_or(0.0);
    }
    let key = format!("{name}='");
    if let Some(i) = head.find(&key) {
        let rest = &head[i + key.len()..];
        let end = rest.find('\'').unwrap_or(rest.len());
        return rest[..end].parse().unwrap_or(0.0);
    }
    0.0
}

fn strip_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    html_unescape(&out)
}

fn html_unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

fn normalize_caption(s: &str) -> String {
    let s = html_unescape(s);
    let s = s.replace('\n', " ");
    let s: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
    s.trim().to_string()
}

fn json_u64(v: Option<&Value>) -> u64 {
    match v {
        Some(Value::Number(n)) => n
            .as_u64()
            .or_else(|| n.as_f64().map(|f| f.max(0.0) as u64))
            .unwrap_or(0),
        Some(Value::String(s)) => s.parse().unwrap_or(0),
        _ => 0,
    }
}

fn stitch_durations(cues: &mut [Cue]) {
    for i in 0..cues.len() {
        if cues[i].duration_ms == 0 {
            let next = cues.get(i + 1).map(|c| c.start_ms).unwrap_or(cues[i].start_ms + 2000);
            cues[i].duration_ms = next.saturating_sub(cues[i].start_ms).max(400);
        }
    }
}

fn collapse_repeat_cues(cues: &mut Vec<Cue>) {
    cues.dedup_by(|later, earlier| later.text == earlier.text);
}

fn chapters_from(player: &Value) -> Vec<Chapter> {
    let mut out = Vec::new();
    walk_chapters(player, &mut out);
    out.sort_by_key(|c| c.start_ms);
    out.dedup_by_key(|c| c.start_ms);
    out
}

fn walk_chapters(v: &Value, out: &mut Vec<Chapter>) {
    match v {
        Value::Array(arr) => {
            for x in arr {
                walk_chapters(x, out);
            }
        }
        Value::Object(map) => {
            if let Some(renderer) = map.get("macroMarkersListItemRenderer") {
                if let Some(ch) = chapter_from_marker(renderer) {
                    out.push(ch);
                }
            }
            if let Some(renderer) = map.get("chapterRenderer") {
                if let Some(ch) = chapter_from_marker(renderer) {
                    out.push(ch);
                }
            }
            for x in map.values() {
                walk_chapters(x, out);
            }
        }
        _ => {}
    }
}

fn chapter_from_marker(v: &Value) -> Option<Chapter> {
    let title = json_text(v.get("title"))?;
    let secs = v
        .pointer("/onTap/watchEndpoint/startTimeSeconds")
        .and_then(|x| x.as_u64())
        .or_else(|| v.get("timeRangeStartMillis").and_then(|x| x.as_u64()).map(|ms| ms / 1000))
        .or_else(|| {
            v.get("startTimeSeconds")
                .and_then(|x| x.as_u64().or_else(|| x.as_str()?.parse().ok()))
        })?;
    Some(Chapter {
        start_ms: secs * 1000,
        title,
    })
}

fn cache_get(key: &str) -> Option<Arc<TranscriptDoc>> {
    let mut g = CACHE.lock();
    g.retain(|(_, at, _)| at.elapsed() < CACHE_TTL);
    g.iter()
        .find(|(k, _, _)| k == key)
        .map(|(_, _, d)| d.clone())
}

fn cache_get_source(video_id: &str, lang_key: &str) -> Option<Arc<TranscriptDoc>> {
    if !lang_key.is_empty() {
        if let Some(hit) = cache_get(&format!("{video_id}|{lang_key}|")) {
            return Some(hit);
        }
    }
    cache_get(&format!("{video_id}||"))
}

const CUE_SEP: &str = "\n\u{E000}\n";
const GTX_URL: &str = "https://translate.googleapis.com/translate_a/single";

/// Translate cue text, keeping start/duration. Used when YouTube tlang is blocked.
pub async fn translate_cue_texts(
    mut cues: Vec<Cue>,
    source_lang: &str,
    tlang: &str,
) -> Result<Vec<Cue>, FetchError> {
    let tl = tlang.trim().to_ascii_lowercase();
    if tl.is_empty() {
        return Ok(cues);
    }
    let sl = source_lang
        .trim()
        .split([':', '|', '-', '_'])
        .next()
        .unwrap_or("auto")
        .to_ascii_lowercase();
    let sl = if sl.is_empty() || sl == "asr" {
        "auto"
    } else {
        sl.as_str()
    };
    if sl != "auto" && sl.eq_ignore_ascii_case(&tl) {
        return Ok(cues);
    }
    if cues.is_empty() {
        return Ok(cues);
    }
    if cues.len() > 25_000 {
        return Err(FetchError::new(400, "That caption file is too large."));
    }

    let texts: Vec<String> = cues.iter().map(|c| c.text.clone()).collect();
    let translated = gtx_lines(sl, &tl, &texts).await?;
    if translated.len() != cues.len() {
        return Err(FetchError::new(502, "Translation returned the wrong number of lines."));
    }
    for (cue, text) in cues.iter_mut().zip(translated) {
        cue.text = text;
    }
    Ok(cues)
}

async fn translated_from_source(src: &TranscriptDoc, tlang: &str) -> Result<TranscriptDoc, FetchError> {
    let cues = translate_cue_texts(src.cues.clone(), &src.track.lang, tlang).await?;
    let mut track = src.track.clone();
    let dest = language_name(tlang);
    if !track.name.contains(&dest) {
        track.name = format!("{} → {dest}", track.name);
    }
    Ok(TranscriptDoc {
        video_id: src.video_id.clone(),
        title: src.title.clone(),
        author: src.author.clone(),
        channel_id: src.channel_id.clone(),
        duration_secs: src.duration_secs,
        track,
        tracks: src.tracks.clone(),
        translations: src.translations.clone(),
        cues,
        chapters: src.chapters.clone(),
    })
}

async fn gtx_lines(sl: &str, tl: &str, lines: &[String]) -> Result<Vec<String>, FetchError> {
    let mut out = Vec::with_capacity(lines.len());
    let mut batch: Vec<&str> = Vec::new();
    let mut batch_chars = 0usize;
    for line in lines {
        let extra = line.len() + CUE_SEP.len();
        if !batch.is_empty() && batch_chars + extra > 450 {
            out.extend(gtx_batch(sl, tl, &batch).await?);
            batch.clear();
            batch_chars = 0;
        }
        batch.push(line.as_str());
        batch_chars += extra;
    }
    if !batch.is_empty() {
        out.extend(gtx_batch(sl, tl, &batch).await?);
    }
    Ok(out)
}

async fn gtx_batch(sl: &str, tl: &str, batch: &[&str]) -> Result<Vec<String>, FetchError> {
    if batch.len() == 1 {
        let one = gtx_text(sl, tl, batch[0]).await?;
        return Ok(vec![one]);
    }
    let joined = batch.join(CUE_SEP);
    let rendered = gtx_text(sl, tl, &joined).await?;
    let parts: Vec<String> = rendered.split(CUE_SEP).map(|s| s.to_string()).collect();
    if parts.len() == batch.len() {
        return Ok(parts);
    }
    let mut out = Vec::with_capacity(batch.len());
    for line in batch {
        out.push(gtx_text(sl, tl, line).await?);
    }
    Ok(out)
}

async fn gtx_text(sl: &str, tl: &str, text: &str) -> Result<String, FetchError> {
    if text.trim().is_empty() {
        return Ok(text.to_string());
    }
    if !SKIP_GTX.load(Ordering::Relaxed) {
        match gtx_text_once(sl, tl, text).await {
            Ok(s) => return Ok(s),
            Err(_) => SKIP_GTX.store(true, Ordering::Relaxed),
        }
    }
    mymemory_text(sl, tl, text).await
}

async fn gtx_text_once(sl: &str, tl: &str, text: &str) -> Result<String, FetchError> {
    let resp = HTTP_VR
        .get(GTX_URL)
        .query(&[
            ("client", "gtx"),
            ("sl", sl),
            ("tl", tl),
            ("dt", "t"),
            ("q", text),
        ])
        .header("User-Agent", WEB_UA)
        .send()
        .await
        .map_err(|e| FetchError::new(502, format!("Translation request failed ({e}).")))?;
    let status = resp.status().as_u16();
    if status == 429 {
        return Err(FetchError::new(
            429,
            "Translation is rate-limited right now. Try Apply again in a minute.",
        ));
    }
    if status >= 400 {
        return Err(FetchError::new(status, "Translation request was refused."));
    }
    let body = resp
        .text()
        .await
        .map_err(|_| FetchError::new(502, "Translation response could not be read."))?;
    if body.trim_start().starts_with('<') {
        return Err(FetchError::new(502, "Translation response was not usable."));
    }
    parse_gtx_body(&body).ok_or_else(|| FetchError::new(502, "Translation response was not usable."))
}

const MYMEMORY_URL: &str = "https://api.mymemory.translated.net/get";

async fn mymemory_text(sl: &str, tl: &str, text: &str) -> Result<String, FetchError> {
    let pair = format!("{sl}|{tl}");
    let resp = HTTP_VR
        .get(MYMEMORY_URL)
        .query(&[("q", text), ("langpair", pair.as_str())])
        .header("User-Agent", WEB_UA)
        .send()
        .await
        .map_err(|e| FetchError::new(502, format!("Translation request failed ({e}).")))?;
    if !resp.status().is_success() {
        return Err(FetchError::new(
            resp.status().as_u16(),
            "Translation request was refused.",
        ));
    }
    let v: Value = resp
        .json()
        .await
        .map_err(|_| FetchError::new(502, "Translation response was not JSON."))?;
    let status = v.get("responseStatus").and_then(|x| x.as_u64()).unwrap_or(0);
    let translated = v
        .pointer("/responseData/translatedText")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .trim();
    if status != 200 || translated.is_empty() {
        return Err(FetchError::new(502, "Translation response was not usable."));
    }
    Ok(html_escape::decode_html_entities(translated).into_owned())
}

fn parse_gtx_body(body: &str) -> Option<String> {
    let v: Value = serde_json::from_str(body.trim()).ok()?;
    let rows = v.get(0)?.as_array()?;
    let mut out = String::new();
    for row in rows {
        if let Some(s) = row.get(0).and_then(|x| x.as_str()) {
            out.push_str(s);
        }
    }
    Some(out)
}

fn cache_put(key: String, doc: Arc<TranscriptDoc>) {
    let mut g = CACHE.lock();
    g.retain(|(_, at, _)| at.elapsed() < CACHE_TTL);
    if g.len() >= CACHE_CAP {
        g.remove(0);
    }
    g.push((key, Instant::now(), doc));
}

#[derive(Clone)]
struct RawAudio {
    mime: String,
    ext: String,
    url: String,
    bitrate: u64,
    itag: u64,
    ua: String,
}

fn extract_audio(player: &Value, ua: &str) -> Vec<RawAudio> {
    let mut out = Vec::new();
    for key in ["/streamingData/adaptiveFormats", "/streamingData/formats"] {
        let Some(arr) = player.pointer(key).and_then(|v| v.as_array()) else {
            continue;
        };
        for item in arr {
            if let Some(a) = raw_audio_from_item(item, ua) {
                out.push(a);
            }
        }
    }
    out
}

fn raw_audio_from_item(item: &Value, ua: &str) -> Option<RawAudio> {
    let mime = item.get("mimeType").and_then(|v| v.as_str()).unwrap_or("");
    if !mime.to_ascii_lowercase().starts_with("audio/") {
        return None;
    }
    let url = item
        .get("url")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())?;
    let bitrate = item
        .get("bitrate")
        .and_then(|v| v.as_u64())
        .or_else(|| item.get("averageBitrate").and_then(|v| v.as_u64()))
        .unwrap_or(0);
    let itag = item.get("itag").and_then(|v| v.as_u64()).unwrap_or(0);
    Some(RawAudio {
        ext: ext_from_mime(mime),
        mime: mime.to_string(),
        url: url.to_string(),
        bitrate,
        itag,
        ua: ua.to_string(),
    })
}

fn ext_from_mime(mime: &str) -> String {
    let head = mime.split(';').next().unwrap_or(mime).trim().to_ascii_lowercase();
    match head.as_str() {
        "audio/mp4" | "audio/m4a" => "m4a".into(),
        "audio/webm" => "webm".into(),
        "audio/mpeg" => "mp3".into(),
        "audio/ogg" | "audio/opus" => "ogg".into(),
        _ => "m4a".into(),
    }
}

fn audio_rank(a: &RawAudio) -> (u8, u64) {
    let prefer = match a.ext.as_str() {
        "m4a" => 3,
        "mp3" => 2,
        "webm" => 1,
        _ => 0,
    };
    (prefer, a.bitrate)
}

fn merge_audio(into: &mut Vec<RawAudio>, extra: Vec<RawAudio>) {
    for t in extra {
        if let Some(existing) = into.iter_mut().find(|e| {
            e.itag == t.itag && e.ext == t.ext && e.ua == t.ua
        }) {
            if audio_rank(&t) > audio_rank(existing) {
                *existing = t;
            }
        } else {
            into.push(t);
        }
    }
}

fn best_audio(audios: &[RawAudio]) -> Option<RawAudio> {
    audios.iter().max_by_key(|a| audio_rank(a)).cloned()
}

fn cache_audios(video_id: &str, title: &str, mut audios: Vec<RawAudio>) {
    audios.sort_by(|a, b| audio_rank(b).cmp(&audio_rank(a)));
    let picks: Vec<AudioPick> = audios
        .into_iter()
        .map(|a| AudioPick {
            video_id: video_id.to_string(),
            title: title.to_string(),
            mime: a.mime,
            ext: a.ext,
            url: a.url,
            bitrate: a.bitrate,
            ua: a.ua,
        })
        .collect();
    if picks.is_empty() {
        return;
    }
    let mut g = AUDIO_CACHE.lock();
    g.retain(|(_, at, _)| at.elapsed() < CACHE_TTL);
    g.retain(|(id, _, _)| id != video_id);
    if g.len() >= CACHE_CAP {
        g.remove(0);
    }
    g.push((video_id.to_string(), Instant::now(), picks));
}

#[allow(dead_code)]
fn audio_cache_drop(video_id: &str) {
    AUDIO_CACHE.lock().retain(|(id, _, _)| id != video_id);
}

fn audio_cache_get(video_id: &str) -> Option<Vec<AudioPick>> {
    let mut g = AUDIO_CACHE.lock();
    g.retain(|(_, at, _)| at.elapsed() < CACHE_TTL);
    g.iter()
        .find(|(k, _, _)| k == video_id)
        .map(|(_, _, d)| d.clone())
}

fn audio_missing() -> FetchError {
    FetchError::new(
        404,
        "YouTube did not expose a plain audio URL for this video (it may need a signature). Try another public video.",
    )
}

pub async fn pick_audio(video_id: &str) -> Result<AudioPick, FetchError> {
    let picks = load_audio_picks(video_id).await?;
    picks.into_iter().next().ok_or_else(audio_missing)
}

async fn load_audio_picks(video_id: &str) -> Result<Vec<AudioPick>, FetchError> {
    if !is_id(video_id) {
        return Err(FetchError::new(400, "That is not a YouTube video id."));
    }
    if let Some(hit) = audio_cache_get(video_id) {
        if !hit.is_empty() {
            return Ok(hit);
        }
    }
    fill_audio_cache(video_id).await?;
    audio_cache_get(video_id)
        .filter(|v| !v.is_empty())
        .ok_or_else(audio_missing)
}

async fn fill_audio_cache(video_id: &str) -> Result<(), FetchError> {
    let clients = listing_clients();
    let mut audios: Vec<RawAudio> = Vec::new();
    let mut title = String::new();
    // iOS googlevideo URLs accept 1 MiB ranges; try that client first.
    for idx in [1usize, 0, 2, 3] {
        let client = &clients[idx];
        let Ok(player) = innertube_player(video_id, client, "en").await else {
            continue;
        };
        if title.is_empty() {
            if let Some(t) = json_text(player.pointer("/videoDetails/title")) {
                title = t;
            }
        }
        merge_audio(&mut audios, extract_audio(&player, client.ua));
        if !audios.is_empty() {
            let label = if title.is_empty() { video_id } else { title.as_str() };
            cache_audios(video_id, label, audios);
            return Ok(());
        }
    }
    if let Ok(player) = watch_page_player(video_id, "en").await {
        if title.is_empty() {
            if let Some(t) = json_text(player.pointer("/videoDetails/title")) {
                title = t;
            }
        }
        merge_audio(&mut audios, extract_audio(&player, WEB_UA));
    }
    if audios.is_empty() {
        return Err(audio_missing());
    }
    let label = if title.is_empty() { video_id } else { title.as_str() };
    cache_audios(video_id, label, audios);
    Ok(())
}

async fn range_get(url: &str, ua: &str, start: u64, end: u64) -> Result<reqwest::Response, FetchError> {
    HTTP_GV
        .get(url)
        .header("User-Agent", ua)
        .header("Accept", "*/*")
        .header("Range", format!("bytes={start}-{end}"))
        .header("Connection", "close")
        .send()
        .await
        .map_err(|e| FetchError::new(502, format!("Audio download failed ({e}).")))
}

fn content_range_total(resp: &reqwest::Response) -> Option<u64> {
    resp.headers()
        .get("content-range")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.rsplit('/').next())
        .and_then(|s| s.parse().ok())
}

/// googlevideo 403s open-ended and large Range requests; ~1 MiB slices work.
const GV_CHUNK: u64 = 1024 * 1024;
const GV_MAX: u64 = 80 * 1024 * 1024;

pub type AudioByteStream = Pin<Box<dyn Stream<Item = Result<Bytes, io::Error>> + Send>>;

fn rest_audio_chunks(
    url: String,
    ua: String,
    off: u64,
    total: u64,
) -> impl Stream<Item = Result<Bytes, io::Error>> + Send {
    stream::unfold(off, move |off| {
        let url = url.clone();
        let ua = ua.clone();
        async move {
            if off >= total {
                return None;
            }
            let end = (off + GV_CHUNK - 1).min(total - 1);
            let mut last_err = None;
            for _ in 0..3u8 {
                match range_get(&url, &ua, off, end).await {
                    Ok(resp) if resp.status().as_u16() < 400 => match resp.bytes().await {
                        Ok(b) => {
                            let n = b.len() as u64;
                            if n == 0 {
                                return None;
                            }
                            return Some((Ok(b), off + n));
                        }
                        Err(e) => last_err = Some(io::Error::other(e.to_string())),
                    },
                    Ok(resp) => {
                        last_err = Some(io::Error::other(format!("YouTube {}", resp.status())));
                    }
                    Err(e) => last_err = Some(io::Error::other(e.message)),
                }
            }
            Some((
                Err(last_err.unwrap_or_else(|| io::Error::other("audio chunk failed"))),
                total,
            ))
        }
    })
}

async fn open_audio_stream(url: &str, ua: &str) -> Result<(u64, AudioByteStream), FetchError> {
    let probe = range_get(url, ua, 0, 1023).await?;
    let status = probe.status().as_u16();
    if status == 200 {
        let b = probe
            .bytes()
            .await
            .map_err(|e| FetchError::new(502, format!("Audio download failed ({e}).")))?;
        let n = b.len() as u64;
        let s = stream::once(async move { Ok::<Bytes, io::Error>(b) });
        return Ok((n, Box::pin(s)));
    }
    if status >= 400 {
        return Err(FetchError::new(
            status,
            "YouTube refused the audio stream. Refresh and try again.",
        ));
    }
    let total = content_range_total(&probe).unwrap_or(0);
    if total == 0 {
        return Err(FetchError::new(502, "YouTube did not report the audio size."));
    }
    if total > GV_MAX {
        return Err(FetchError::new(
            413,
            "That audio file is too large to proxy. Open the video on YouTube instead.",
        ));
    }
    let first = probe
        .bytes()
        .await
        .map_err(|e| FetchError::new(502, format!("Audio download failed ({e}).")))?;
    let off = first.len() as u64;
    let s = stream::once(async move { Ok::<Bytes, io::Error>(first) }).chain(rest_audio_chunks(
        url.to_string(),
        ua.to_string(),
        off,
        total,
    ));
    Ok((total, Box::pin(s)))
}

fn ytdlp_path() -> Option<PathBuf> {
    if let Ok(p) = env::var("YTDLP") {
        let path = PathBuf::from(p);
        if path.is_file() {
            return Some(path);
        }
    }
    for candidate in ["/usr/local/bin/yt-dlp", "/usr/bin/yt-dlp", "./yt-dlp"] {
        let path = Path::new(candidate);
        if path.is_file() {
            return Some(path.to_path_buf());
        }
    }
    None
}

fn placeholder_pick(video_id: &str) -> AudioPick {
    AudioPick {
        video_id: video_id.to_string(),
        title: video_id.to_string(),
        mime: "audio/mp4".into(),
        ext: "m4a".into(),
        url: String::new(),
        bitrate: 0,
        ua: String::new(),
    }
}

async fn peek_audio_len(url: &str, ua: &str) -> Result<u64, FetchError> {
    let probe = range_get(url, ua, 0, 1023).await?;
    let status = probe.status().as_u16();
    if status == 200 {
        return Ok(probe.content_length().unwrap_or(0));
    }
    if status >= 400 {
        return Err(FetchError::new(
            status,
            "YouTube refused the audio stream. Refresh and try again.",
        ));
    }
    content_range_total(&probe).ok_or_else(|| {
        FetchError::new(502, "YouTube did not report the audio size.")
    })
}

async fn stream_ytdlp(
    video_id: &str,
    mut pick: AudioPick,
    format: &str,
    extra: &[&str],
    fail: &str,
) -> Result<(AudioPick, u64, AudioByteStream), FetchError> {
    let bin = ytdlp_path().ok_or_else(|| {
        FetchError::new(
            503,
            "Media download is not available on this server yet. Try again after the next deploy.",
        )
    })?;
    let url = format!("https://www.youtube.com/watch?v={video_id}");
    let mut cmd = tokio::process::Command::new(bin);
    if Path::new("/usr/bin/ffmpeg").is_file() {
        cmd.env("FFMPEG", "/usr/bin/ffmpeg");
    }
    let mut child = cmd
        .args([
            "-f",
            format,
            "-o",
            "-",
            "--no-progress",
            "--no-warnings",
            "--no-playlist",
        ])
        .args(extra)
        .args(["--", &url])
        .env("HOME", "/tmp")
        .env("XDG_CACHE_HOME", "/tmp")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| FetchError::new(502, format!("{fail} ({e}).")))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| FetchError::new(502, fail.to_string()))?;
    tokio::spawn(async move {
        let _ = child.wait().await;
    });
    let stream = tokio_util::io::ReaderStream::new(stdout)
        .map(|item| item.map_err(|e| io::Error::other(e.to_string())));
    if pick.ext.is_empty() {
        pick.ext = "mp4".into();
    }
    Ok((pick, 0, Box::pin(stream)))
}

fn ffmpeg_available() -> bool {
    env::var("FFMPEG")
        .ok()
        .map(|p| Path::new(&p).is_file())
        .unwrap_or(false)
        || Path::new("/usr/bin/ffmpeg").is_file()
}

pub fn normalize_video_quality(raw: Option<&str>) -> &'static str {
    match raw.unwrap_or("720").trim().to_ascii_lowercase().as_str() {
        "360" | "360p" => "360",
        "480" | "480p" => "480",
        "1080" | "1080p" => "1080",
        "best" | "max" => "best",
        _ => "720",
    }
}

fn video_ytdlp_format(quality: &str, ffmpeg: bool) -> String {
    let height = match quality {
        "360" => Some(360u32),
        "480" => Some(480),
        "1080" => Some(1080),
        "best" => None,
        _ => Some(720),
    };
    match (ffmpeg, height) {
        (true, None) => "bv*+ba/b".into(),
        (true, Some(h)) => format!("b[height<={h}][ext=mp4]/bv*[height<={h}]+ba/b[height<={h}]/b"),
        (false, None) => "best[ext=mp4]/best".into(),
        (false, Some(h)) => {
            format!("best[height<={h}][ext=mp4]/best[height<={h}]/best")
        }
    }
}

pub async fn download_video(
    video_id: &str,
    quality: &str,
) -> Result<(AudioPick, u64, AudioByteStream), FetchError> {
    if !is_id(video_id) {
        return Err(FetchError::new(400, "That is not a YouTube video id."));
    }
    let q = normalize_video_quality(Some(quality));
    let ffmpeg = ffmpeg_available();
    let spec = video_ytdlp_format(q, ffmpeg);
    let extra: Vec<&str> = if ffmpeg {
        vec!["--merge-output-format", "mp4"]
    } else {
        Vec::new()
    };
    let mut meta = load_audio_picks(video_id)
        .await
        .ok()
        .and_then(|p| p.into_iter().next())
        .unwrap_or_else(|| placeholder_pick(video_id));
    meta.ext = "mp4".into();
    meta.mime = "video/mp4".into();
    stream_ytdlp(
        video_id,
        meta,
        &spec,
        &extra,
        "Could not start video download",
    )
    .await
}

pub fn normalize_audio_format(raw: Option<&str>) -> &'static str {
    match raw.unwrap_or("m4a").trim().to_ascii_lowercase().as_str() {
        "mp3" | "mpeg" => "mp3",
        "opus" | "webm" | "ogg" => "opus",
        "wav" => "wav",
        _ => "m4a",
    }
}

fn audio_ytdlp_plan(fmt: &str) -> (&'static str, &'static [&'static str], &'static str, &'static str) {
    match fmt {
        "mp3" => (
            "bestaudio/best",
            &["-x", "--audio-format", "mp3"],
            "audio/mpeg",
            "mp3",
        ),
        "opus" => (
            "bestaudio/best",
            &["-x", "--audio-format", "opus"],
            "audio/ogg",
            "opus",
        ),
        "wav" => (
            "bestaudio/best",
            &["-x", "--audio-format", "wav"],
            "audio/wav",
            "wav",
        ),
        _ => (
            "bestaudio[ext=m4a]/bestaudio/best",
            &[],
            "audio/mp4",
            "m4a",
        ),
    }
}

pub async fn download_audio(
    video_id: &str,
    fmt: &str,
) -> Result<(AudioPick, u64, AudioByteStream), FetchError> {
    if !is_id(video_id) {
        return Err(FetchError::new(400, "That is not a YouTube video id."));
    }
    let fmt = normalize_audio_format(Some(fmt));
    let (spec, extra, mime, ext) = audio_ytdlp_plan(fmt);
    if !extra.is_empty() && !ffmpeg_available() {
        return Err(FetchError::new(
            503,
            "MP3 and other converted formats need ffmpeg on the server. Try M4A, or wait for the next deploy.",
        ));
    }
    if fmt == "m4a" {
        let picks = load_audio_picks(video_id).await.unwrap_or_default();
        for pick in &picks {
            match peek_audio_len(&pick.url, &pick.ua).await {
                Ok(len) if len > 0 && len <= GV_CHUNK => {
                    return open_audio_stream(&pick.url, &pick.ua)
                        .await
                        .map(|(n, s)| {
                            let mut p = pick.clone();
                            p.mime = mime.into();
                            p.ext = ext.into();
                            (p, n, s)
                        });
                }
                _ => {}
            }
        }
        let mut meta = picks
            .into_iter()
            .next()
            .unwrap_or_else(|| placeholder_pick(video_id));
        meta.mime = mime.into();
        meta.ext = ext.into();
        return stream_ytdlp(
            video_id,
            meta,
            spec,
            extra,
            "Could not start audio download",
        )
        .await;
    }
    let mut meta = load_audio_picks(video_id)
        .await
        .ok()
        .and_then(|p| p.into_iter().next())
        .unwrap_or_else(|| placeholder_pick(video_id));
    meta.mime = mime.into();
    meta.ext = ext.into();
    stream_ytdlp(
        video_id,
        meta,
        spec,
        extra,
        "Could not start audio download",
    )
    .await
}

/// Store a transcript fetched in the user's browser (extension / bookmarklet).
pub fn ingest_client_doc(
    video_id: String,
    title: String,
    author: String,
    duration_secs: u64,
    lang: String,
    kind: String,
    tlang: String,
    mut cues: Vec<Cue>,
) -> Result<Arc<TranscriptDoc>, FetchError> {
    if !is_id(&video_id) {
        return Err(FetchError::new(400, "That is not a YouTube video id."));
    }
    if cues.is_empty() {
        return Err(FetchError::new(400, "No caption lines were sent."));
    }
    if cues.len() > 25_000 {
        return Err(FetchError::new(400, "That caption file is too large."));
    }
    for cue in &mut cues {
        if cue.text.len() > 2_000 {
            cue.text.truncate(2_000);
        }
    }
    cues.sort_by_key(|c| c.start_ms);
    stitch_durations(&mut cues);
    collapse_repeat_cues(&mut cues);
    let lang = lang.to_ascii_lowercase();
    let kind = kind.to_ascii_lowercase();
    let tlang = tlang.to_ascii_lowercase();
    let title = if title.trim().is_empty() {
        video_id.clone()
    } else {
        title.chars().take(400).collect()
    };
    let track = CaptionTrack {
        name: {
            let mut n = language_name(&lang);
            if kind == "asr" {
                n.push_str(" (auto)");
            }
            n
        },
        lang: lang.clone(),
        kind: kind.clone(),
        translatable: true,
    };
    let doc = Arc::new(TranscriptDoc {
        video_id: video_id.clone(),
        title,
        author: author.chars().take(200).collect(),
        channel_id: String::new(),
        duration_secs,
        tracks: vec![track.clone()],
        track,
        translations: translation_catalog()
            .into_iter()
            .map(|(code, name)| LangOpt { code, name })
            .collect(),
        cues,
        chapters: vec![],
    });
    let lang_key = if kind == "asr" && !lang.is_empty() {
        format!("{lang}:asr")
    } else {
        lang
    };
    cache_put(format!("{video_id}|{lang_key}|{tlang}"), doc.clone());
    cache_put(format!("{video_id}||"), doc.clone());
    Ok(doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json3_skips_window_events() {
        let v: Value = serde_json::from_str(
            r#"{
              "events": [
                {"tStartMs": 0, "id": 1},
                {"tStartMs": 1200, "dDurationMs": 1800, "segs": [{"utf8": "Never "}, {"utf8": "gonna"}]}
              ]
            }"#,
        )
        .unwrap();
        let cues = parse_json3(&v);
        assert_eq!(cues.len(), 1);
        assert_eq!(cues[0].text, "Never gonna");
        assert_eq!(cues[0].start_ms, 1200);
    }

    #[test]
    fn vtt_times() {
        let body = "WEBVTT\n\n00:00:01.500 --> 00:00:03.000\nHello\n\n";
        let cues = parse_vtt(body);
        assert_eq!(cues[0].start_ms, 1500);
        assert_eq!(cues[0].text, "Hello");
    }

    #[test]
    fn json3_accepts_float_timestamps() {
        let v: Value = serde_json::from_str(
            r#"{"events":[{"tStartMs": 1500.0, "dDurationMs": 900.5, "segs":[{"utf8":"Hi"}]}]}"#,
        )
        .unwrap();
        let cues = parse_json3(&v);
        assert_eq!(cues[0].start_ms, 1500);
    }

    #[test]
    fn set_query_replaces_fmt() {
        let u = set_query(
            "https://www.youtube.com/api/timedtext?v=x&fmt=srv3&lang=en",
            "fmt",
            "json3",
        );
        assert!(u.contains("fmt=json3"));
        assert!(!u.contains("fmt=srv3"));
    }

    #[test]
    fn set_query_keeps_sparams_commas() {
        let base = "https://www.youtube.com/api/timedtext?v=dQw4w9WgXcQ&expire=1&sparams=ip,ipbits,expire,v&signature=ABC/def+ghi&lang=en";
        let u = set_query(base, "fmt", "json3");
        assert!(u.contains("sparams=ip,ipbits,expire,v"), "{u}");
        assert!(u.contains("signature=ABC/def+ghi"), "{u}");
        assert!(u.contains("fmt=json3"));
        assert!(!u.contains("%2C"));
        let again = set_query(&u, "tlang", "es");
        assert!(again.contains("sparams=ip,ipbits,expire,v"));
        assert!(again.contains("tlang=es"));
    }

    #[test]
    fn parse_captions_ignores_html_error_pages() {
        let html = "<!DOCTYPE html><html><body><p>Too many requests</p><p>Retry later</p></body></html>";
        assert!(parse_captions(html).is_empty());
    }

    #[test]
    fn parse_srv3_timedtext() {
        let xml = r#"<?xml version="1.0"?><timedtext><body><p t="1200" d="1800">Hello there</p></body></timedtext>"#;
        let cues = parse_captions(xml);
        assert_eq!(cues.len(), 1);
        assert_eq!(cues[0].text, "Hello there");
        assert_eq!(cues[0].start_ms, 1200);
    }

    #[test]
    fn collapse_consecutive_asr_repeats() {
        let mut cues = vec![
            Cue { start_ms: 0, duration_ms: 800, text: "Hello".into() },
            Cue { start_ms: 400, duration_ms: 800, text: "Hello".into() },
            Cue { start_ms: 1200, duration_ms: 800, text: "World".into() },
        ];
        collapse_repeat_cues(&mut cues);
        assert_eq!(cues.len(), 2);
        assert_eq!(cues[0].text, "Hello");
        assert_eq!(cues[1].text, "World");
    }

    #[test]
    fn pick_asr_vs_human() {
        let tracks = vec![
            RemoteTrack {
                lang: "en".into(),
                name: "English".into(),
                kind: String::new(),
                translatable: true,
                base_url: "https://example/h".into(),
            },
            RemoteTrack {
                lang: "en".into(),
                name: "English".into(),
                kind: "asr".into(),
                translatable: true,
                base_url: "https://example/a".into(),
            },
        ];
        assert_eq!(pick_track(&tracks, Some("en")).unwrap().kind, "");
        assert_eq!(pick_track(&tracks, Some("en|asr")).unwrap().kind, "asr");
        assert_eq!(pick_track(&tracks, Some("en:asr")).unwrap().kind, "asr");
        assert_eq!(track_key("en", "asr"), "en:asr");
    }

    #[test]
    fn caption_tracks_walks_nested_json() {
        let v: Value = serde_json::from_str(
            r#"{
              "wrapper": {
                "captions": {
                  "playerCaptionsTracklistRenderer": {
                    "captionTracks": [
                      {
                        "baseUrl": "https://www.youtube.com/api/timedtext?v=x&lang=es",
                        "languageCode": "es",
                        "kind": "asr",
                        "name": {"simpleText": "Spanish"}
                      }
                    ]
                  }
                }
              }
            }"#,
        )
        .unwrap();
        let tracks = caption_tracks(&v);
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].lang, "es");
        assert_eq!(tracks[0].kind, "asr");
        assert!(tracks[0].base_url.contains("timedtext"));
    }

    #[test]
    fn merge_prefers_android_srv3_over_pot_url() {
        let mut tracks = vec![RemoteTrack {
            lang: "en".into(),
            name: "English".into(),
            kind: String::new(),
            translatable: true,
            base_url: "https://www.youtube.com/api/timedtext?v=x&exp=xpe&lang=en".into(),
        }];
        merge_tracks(
            &mut tracks,
            vec![RemoteTrack {
                lang: "en".into(),
                name: "English".into(),
                kind: String::new(),
                translatable: true,
                base_url: "https://www.youtube.com/api/timedtext?v=x&fmt=srv3&lang=en".into(),
            }],
        );
        assert_eq!(tracks.len(), 1);
        assert!(tracks[0].base_url.contains("fmt=srv3"));
        assert!(!needs_pot(&tracks[0].base_url));
    }

    #[test]
    fn absolutize_relative_timedtext() {
        assert_eq!(
            absolutize_caption_url("/api/timedtext?v=x"),
            "https://www.youtube.com/api/timedtext?v=x"
        );
    }

    #[test]
    fn extract_audio_skips_video_and_cipher() {
        let v: Value = serde_json::from_str(
            r#"{
              "streamingData": {
                "adaptiveFormats": [
                  {"itag": 137, "mimeType": "video/mp4", "url": "https://x/v"},
                  {"itag": 140, "mimeType": "audio/mp4; codecs=\"mp4a.40.2\"", "bitrate": 128000, "url": "https://x/a"},
                  {"itag": 251, "mimeType": "audio/webm", "signatureCipher": "s=1&url=https://x/b"}
                ]
              }
            }"#,
        )
        .unwrap();
        let audios = extract_audio(&v, "test-ua");
        assert_eq!(audios.len(), 1);
        assert_eq!(audios[0].ext, "m4a");
        assert_eq!(audios[0].ua, "test-ua");
        assert_eq!(best_audio(&audios).unwrap().url, "https://x/a");
    }

    #[test]
    fn video_quality_and_format_strings() {
        assert_eq!(normalize_video_quality(None), "720");
        assert_eq!(normalize_video_quality(Some("1080p")), "1080");
        assert_eq!(normalize_video_quality(Some("best")), "best");
        let muxed = video_ytdlp_format("720", false);
        assert!(muxed.contains("height<=720"));
        let dash = video_ytdlp_format("1080", true);
        assert!(dash.contains("bv*"));
    }

    #[test]
    fn audio_format_aliases() {
        assert_eq!(normalize_audio_format(None), "m4a");
        assert_eq!(normalize_audio_format(Some("MP3")), "mp3");
        assert_eq!(normalize_audio_format(Some("webm")), "opus");
        assert_eq!(audio_ytdlp_plan("mp3").3, "mp3");
    }

    #[test]
    fn transcript_params_are_urlsafe() {
        let p = transcript_params("dQw4w9WgXcQ", "en", true);
        assert!(!p.is_empty());
        assert!(p.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn parse_transcript_panel_segments() {
        let v: Value = serde_json::from_str(
            r#"{
              "actions": [{
                "updateEngagementPanelAction": {
                  "content": {
                    "transcriptRenderer": {
                      "content": {
                        "transcriptSearchPanelRenderer": {
                          "body": {
                            "transcriptSegmentListRenderer": {
                              "initialSegments": [
                                {
                                  "transcriptSegmentRenderer": {
                                    "startMs": "1200",
                                    "endMs": "3000",
                                    "snippet": {"simpleText": "Hello there"}
                                  }
                                }
                              ]
                            }
                          }
                        }
                      }
                    }
                  }
                }
              }]
            }"#,
        )
        .unwrap();
        let cues = parse_transcript_panel(&v);
        assert_eq!(cues.len(), 1);
        assert_eq!(cues[0].text, "Hello there");
        assert_eq!(cues[0].start_ms, 1200);
        assert_eq!(cues[0].duration_ms, 1800);
    }

    #[test]
    fn json_after_skips_false_marker() {
        let html = r#"var hint = "ytInitialPlayerResponse is injected"; ytInitialPlayerResponse = {"videoDetails":{"title":"Hi"},"captions":{}};"#;
        let found = json_after_all(html, "ytInitialPlayerResponse");
        assert!(found.iter().any(|v| v.pointer("/videoDetails/title").and_then(|x| x.as_str()) == Some("Hi")));
    }

    #[test]
    fn ingest_client_doc_caches() {
        let doc = ingest_client_doc(
            "dQw4w9WgXcQ".into(),
            "Song".into(),
            "Rick".into(),
            213,
            "en".into(),
            "".into(),
            "".into(),
            vec![Cue { start_ms: 0, duration_ms: 1000, text: "Hello".into() }],
        )
        .unwrap();
        assert_eq!(doc.cues.len(), 1);
        assert!(cache_get("dQw4w9WgXcQ||").is_some());
    }

    #[test]
    fn parse_gtx_concatenates_chunks() {
        let body = r#"[[["Hello ","Hola",null,null,3],["world","mundo",null,null,3]],null,"es"]"#;
        assert_eq!(parse_gtx_body(body).as_deref(), Some("Hello world"));
    }

    #[test]
    fn cache_get_source_uses_lang_key() {
        let _ = ingest_client_doc(
            "dQw4w9WgXcQ".into(),
            "Song".into(),
            "Rick".into(),
            213,
            "es".into(),
            "asr".into(),
            "".into(),
            vec![Cue {
                start_ms: 0,
                duration_ms: 1000,
                text: "Hola".into(),
            }],
        )
        .unwrap();
        assert!(cache_get_source("dQw4w9WgXcQ", "es:asr").is_some());
        assert!(cache_get_source("dQw4w9WgXcQ", "").is_some());
    }
}
