//! Rate limits, same-site gate, honeypot, proof-of-work, and download tickets.
//!
//! Shareable `/?v=` links stay challenge-free. `/api/video` and `/api/audio`
//! need a short-lived HMAC ticket issued after a SHA-256 PoW (no Cloudflare).

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::http::HeaderMap;
use hmac::{Hmac, KeyInit, Mac};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use resuma::prelude::FlowRequest;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

const MAX_KEYS: usize = 24_000;
pub const TICKET_TTL_SECS: u64 = 600;
const BAN_SECS: u64 = 30 * 60;
const STRIKE_BAN_AFTER: u32 = 8;
pub const TICKET_COOKIE: &str = "forge_gate";

const POW_MEDIA_MAX: u32 = 80_000;
const POW_HD_MAX: u32 = 400_000;
const POW_TTL_SECS: u64 = 120;

static STATE: Lazy<Mutex<GuardState>> = Lazy::new(|| Mutex::new(GuardState::load()));
static SECRET: Lazy<[u8; 32]> = Lazy::new(load_or_create_secret);

#[derive(Clone, Copy)]
pub struct Limit {
    pub bucket: &'static str,
    pub max: usize,
    pub pro: usize,
    pub window_secs: u64,
}

pub const PAGE: Limit = Limit {
    bucket: "page",
    max: 40,
    pro: 40,
    window_secs: 60,
};
pub const API: Limit = Limit {
    bucket: "api",
    max: 24,
    pro: 240,
    window_secs: 60,
};
pub const AUDIO: Limit = Limit {
    bucket: "audio",
    max: 8,
    pro: 80,
    window_secs: 60,
};
pub const AUDIO_DAY: Limit = Limit {
    bucket: "audio_day",
    max: 30,
    pro: 400,
    window_secs: 86_400,
};
pub const VIDEO_SD: Limit = Limit {
    bucket: "video_sd",
    max: 3,
    pro: 40,
    window_secs: 60,
};
pub const VIDEO_SD_DAY: Limit = Limit {
    bucket: "video_sd_day",
    max: 40,
    pro: 400,
    window_secs: 86_400,
};
pub const VIDEO_720: Limit = Limit {
    bucket: "video720",
    max: 1,
    pro: 8,
    window_secs: 30 * 60,
};
pub const VIDEO_720_DAY: Limit = Limit {
    bucket: "video720_day",
    max: 4,
    pro: 40,
    window_secs: 86_400,
};
pub const VIDEO_HD: Limit = Limit {
    bucket: "videohd",
    max: 1,
    pro: 8,
    window_secs: 86_400,
};
pub const TRANSLATE: Limit = Limit {
    bucket: "translate",
    max: 200,
    pro: 800,
    window_secs: 60,
};
pub const INGEST: Limit = Limit {
    bucket: "ingest",
    max: 12,
    pro: 60,
    window_secs: 60,
};
pub const CHALLENGE: Limit = Limit {
    bucket: "challenge",
    max: 30,
    pro: 120,
    window_secs: 60,
};

#[derive(Debug, Clone)]
pub struct Deny {
    pub hidden: bool,
    pub message: String,
    pub retry_after: Option<u64>,
}

impl Deny {
    fn hidden() -> Self {
        Self {
            hidden: true,
            message: hidden_message(),
            retry_after: None,
        }
    }
    fn busy(message: impl Into<String>, retry_after: Option<u64>) -> Self {
        Self {
            hidden: false,
            message: message.into(),
            retry_after,
        }
    }
    fn forbidden(message: impl Into<String>) -> Self {
        Self {
            hidden: false,
            message: message.into(),
            retry_after: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TicketCap {
    Media,
    Hd,
}

impl TicketCap {
    fn as_str(self) -> &'static str {
        match self {
            TicketCap::Media => "m",
            TicketCap::Hd => "h",
        }
    }
    fn parse(s: &str) -> Option<Self> {
        match s {
            "m" => Some(TicketCap::Media),
            "h" => Some(TicketCap::Hd),
            _ => None,
        }
    }
    pub fn allows_hd(self) -> bool {
        matches!(self, TicketCap::Hd)
    }
}

#[derive(Serialize, Deserialize, Default)]
struct GuardState {
    buckets: HashMap<String, Vec<u64>>,
    bans: HashMap<String, u64>,
    nonces: HashMap<String, u64>,
    strikes: HashMap<String, u32>,
    #[serde(skip)]
    dirty: u32,
}

impl GuardState {
    fn load() -> Self {
        let path = state_path();
        let mut st = fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(GuardState::default);
        st.gc(now_ms());
        st.dirty = 0;
        st
    }

    fn gc(&mut self, now: u64) {
        self.bans.retain(|_, until| *until > now);
        self.nonces.retain(|_, exp| *exp > now);
        for hits in self.buckets.values_mut() {
            hits.retain(|t| now.saturating_sub(*t) < 86_400_000);
        }
        self.buckets.retain(|_, hits| !hits.is_empty());
        let n = self.buckets.len() + self.bans.len() + self.nonces.len();
        if n > MAX_KEYS {
            self.evict_oldest(n.saturating_sub(MAX_KEYS * 3 / 4).max(1));
        }
        if self.strikes.len() > 8_000 {
            self.strikes.retain(|ip, _| self.bans.contains_key(ip));
        }
    }

    fn evict_oldest(&mut self, drop_at_least: usize) {
        let mut keys: Vec<(u64, String)> = self
            .buckets
            .iter()
            .map(|(k, hits)| (hits.first().copied().unwrap_or(0), k.clone()))
            .collect();
        keys.sort_unstable_by_key(|(t, _)| *t);
        let drop_n = drop_at_least.max(keys.len() / 4).min(keys.len());
        for (_, k) in keys.into_iter().take(drop_n) {
            self.buckets.remove(&k);
        }
        if self.nonces.len() > MAX_KEYS / 4 {
            let mut ns: Vec<(u64, String)> = self
                .nonces
                .iter()
                .map(|(k, exp)| (*exp, k.clone()))
                .collect();
            ns.sort_unstable_by_key(|(exp, _)| *exp);
            let drop_n = (ns.len() / 4).max(1);
            for (_, k) in ns.into_iter().take(drop_n) {
                self.nonces.remove(&k);
            }
        }
    }

    fn persist_if_needed(&mut self) {
        self.dirty = self.dirty.saturating_add(1);
        if self.dirty < 16 {
            return;
        }
        self.persist();
    }

    fn persist(&mut self) {
        self.dirty = 0;
        let path = state_path();
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let tmp = path.with_extension("json.tmp");
        if let Ok(body) = serde_json::to_string(self) {
            if fs::write(&tmp, body).is_ok() {
                let _ = fs::rename(&tmp, &path);
            }
        }
    }
}

fn data_dir() -> PathBuf {
    PathBuf::from(std::env::var("RESUMA_DATA_DIR").unwrap_or_else(|_| ".resuma".into()))
}

fn state_path() -> PathBuf {
    data_dir().join("guard-state.json")
}

fn secret_path() -> PathBuf {
    data_dir().join("gate.secret")
}

fn load_or_create_secret() -> [u8; 32] {
    if let Ok(raw) = std::env::var("GATE_SECRET") {
        let t = raw.trim();
        if let Some(bytes) = decode_hex32(t) {
            return bytes;
        }
        if !t.is_empty() {
            return sha256_32(t.as_bytes());
        }
    }
    let path = secret_path();
    if let Ok(s) = fs::read_to_string(&path) {
        if let Some(bytes) = decode_hex32(s.trim()) {
            return bytes;
        }
    }
    let mut bytes = [0u8; 32];
    let _ = getrandom::getrandom(&mut bytes);
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let _ = fs::write(&path, hex_encode(&bytes));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
    }
    bytes
}

fn sha256_32(data: &[u8]) -> [u8; 32] {
    let d = Sha256::digest(data);
    let mut out = [0u8; 32];
    out.copy_from_slice(&d);
    out
}

fn hmac_hex(data: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(&*SECRET).expect("hmac key");
    mac.update(data);
    hex_encode(&mac.finalize().into_bytes())
}

fn hmac_ok(data: &[u8], sig_hex: &str) -> bool {
    let mut mac = HmacSha256::new_from_slice(&*SECRET).expect("hmac key");
    mac.update(data);
    let got = mac.finalize().into_bytes();
    let Some(want) = decode_hex(sig_hex) else {
        return false;
    };
    if got.len() != want.len() {
        return false;
    }
    got.iter()
        .zip(want.iter())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0xf) as usize] as char);
    }
    s
}

fn decode_hex(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) || s.is_empty() {
        return None;
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    for i in (0..bytes.len()).step_by(2) {
        let hi = from_hex(bytes[i])?;
        let lo = from_hex(bytes[i + 1])?;
        out.push((hi << 4) | lo);
    }
    Some(out)
}

fn decode_hex32(s: &str) -> Option<[u8; 32]> {
    let v = decode_hex(s)?;
    if v.len() != 32 {
        return None;
    }
    let mut a = [0u8; 32];
    a.copy_from_slice(&v);
    Some(a)
}

fn from_hex(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}

fn now_secs() -> u64 {
    now_ms() / 1000
}

fn rand_bytes<const N: usize>() -> [u8; N] {
    let mut b = [0u8; N];
    let _ = getrandom::getrandom(&mut b);
    b
}

fn rand_u32(max: u32) -> u32 {
    let mut b = [0u8; 4];
    let _ = getrandom::getrandom(&mut b);
    if max == 0 {
        return 0;
    }
    u32::from_le_bytes(b) % max
}

pub fn configured_api_keys() -> Vec<String> {
    std::env::var("FORGE_API_KEYS")
        .ok()
        .or_else(|| std::env::var("API_KEY").ok())
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| s.len() >= 16)
        .collect()
}

pub fn api_key_from_headers(headers: &HeaderMap) -> Option<String> {
    header_str(headers, "x-api-key")
        .or_else(|| {
            header_str(headers, "authorization").and_then(|v| {
                let v = v.trim();
                v.strip_prefix("Bearer ")
                    .or_else(|| v.strip_prefix("bearer "))
                    .map(|s| s.trim().to_string())
            })
        })
        .filter(|s| !s.is_empty())
}

/// Ops escape-hatch keys from `FORGE_API_KEYS` (not Polar subscribers).
pub fn is_ops_key(headers: &HeaderMap) -> bool {
    let Some(provided) = api_key_from_headers(headers) else {
        return false;
    };
    configured_api_keys().iter().any(|k| k == &provided)
}

/// Ops key or Polar subscriber key — skips same-site / PoW gates.
pub fn is_pro_headers(headers: &HeaderMap) -> bool {
    is_ops_key(headers) || crate::billing::is_paid_headers(headers)
}

pub fn allow(ip: &str, limit: Limit) -> bool {
    take_n(ip, limit.bucket, limit.max, limit.window_secs)
}

fn take_n(ip: &str, bucket: &str, max: usize, window_secs: u64) -> bool {
    quota(ip, bucket, max, window_secs, true).is_ok()
}

fn quota(ip: &str, bucket: &str, max: usize, window_secs: u64, commit: bool) -> Result<(), u64> {
    let now = now_ms();
    let window_ms = window_secs.saturating_mul(1000);
    let key = format!("{bucket}|{ip}");
    let mut st = STATE.lock();
    st.gc(now);
    let hits = st.buckets.entry(key).or_default();
    hits.retain(|t| now.saturating_sub(*t) < window_ms);
    if hits.len() >= max {
        let oldest = hits.first().copied().unwrap_or(now);
        let retry = window_ms.saturating_sub(now.saturating_sub(oldest)) / 1000;
        return Err(retry.max(1));
    }
    if commit {
        hits.push(now);
        st.persist_if_needed();
    }
    Ok(())
}

pub fn client_ip_from_headers(headers: &HeaderMap) -> String {
    header_str(headers, "fly-client-ip")
        .or_else(|| header_str(headers, "x-real-ip"))
        .or_else(|| {
            header_str(headers, "x-forwarded-for").and_then(|v| {
                v.split(',').next().map(|s| s.trim().to_string())
            })
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".into())
}

pub fn client_ip(req: &FlowRequest) -> String {
    req.header("fly-client-ip")
        .or_else(|| req.header("x-real-ip"))
        .or_else(|| {
            req.header("x-forwarded-for")
                .and_then(|v| v.split(',').next())
        })
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".into())
}

pub fn check_req(req: &FlowRequest, limit: Limit) -> Result<(), String> {
    let ip = client_ip(req);
    if banned(&ip) {
        return Err(ban_message());
    }
    if allow(&ip, limit) {
        Ok(())
    } else {
        Err(busy_message())
    }
}

pub fn check_headers(headers: &HeaderMap, limit: Limit) -> Result<(), String> {
    check_headers_deny(headers, limit).map_err(|d| d.message)
}

fn check_headers_deny(headers: &HeaderMap, limit: Limit) -> Result<(), Deny> {
    let ip = client_ip_from_headers(headers);
    if banned(&ip) {
        return Err(Deny::busy(ban_message(), Some(BAN_SECS)));
    }
    let (bucket_key, max) = quota_identity(headers, &ip, limit);
    match quota(&bucket_key, limit.bucket, max, limit.window_secs, true) {
        Ok(()) => Ok(()),
        Err(retry) => Err(Deny::busy(busy_for(limit.bucket), Some(retry))),
    }
}

fn peek_headers(headers: &HeaderMap, limit: Limit) -> Result<(), Deny> {
    let ip = client_ip_from_headers(headers);
    if banned(&ip) {
        return Err(Deny::busy(ban_message(), Some(BAN_SECS)));
    }
    let (bucket_key, max) = quota_identity(headers, &ip, limit);
    quota(&bucket_key, limit.bucket, max, limit.window_secs, false)
        .map_err(|retry| Deny::busy(busy_for(limit.bucket), Some(retry)))
}

/// Paid keys rate-limit by key id (plan rate); ops use elevated IP caps; free by IP.
fn quota_identity(headers: &HeaderMap, ip: &str, limit: Limit) -> (String, usize) {
    if let Some(sub) = crate::billing::subscriber_from_headers(headers) {
        let max = match limit.bucket {
            "api" | "page" | "challenge" | "translate" | "ingest" => {
                sub.plan.rate_per_min() as usize
            }
            _ => limit.pro,
        };
        (format!("key:{}", sub.id), max.max(1))
    } else if is_ops_key(headers) {
        (ip.to_string(), limit.pro)
    } else {
        (ip.to_string(), limit.max)
    }
}

/// Site UI + optional server key. Rejects anonymous third-party scrapers.
pub fn check_app_access(headers: &HeaderMap, limit: Limit) -> Result<(), String> {
    check_app_access_deny(headers, limit).map_err(|d| d.message)
}

fn check_app_access_deny(headers: &HeaderMap, limit: Limit) -> Result<(), Deny> {
    if !(is_same_site_request(headers) || is_pro_headers(headers)) {
        return Err(Deny::hidden());
    }
    check_headers_deny(headers, limit)
}

/// Extension ingest from YouTube tabs (cross-origin) or same-site.
pub fn check_ingest_access(headers: &HeaderMap) -> Result<(), String> {
    if !(is_same_site_request(headers)
        || is_extension_or_youtube_origin(headers)
        || is_pro_headers(headers))
    {
        return Err(hidden_message());
    }
    check_headers(headers, INGEST)
}

pub fn hidden_message() -> String {
    "Not found.".into()
}

fn site_origin() -> String {
    crate::family::public_origin()
        .trim_end_matches('/')
        .to_string()
}

fn origin_host(origin: &str) -> Option<String> {
    let u = url::Url::parse(origin).ok()?;
    Some(u.host_str()?.to_ascii_lowercase())
}

fn is_our_origin(origin: &str) -> bool {
    let site = site_origin();
    if origin.eq_ignore_ascii_case(&site) {
        return true;
    }
    matches!(
        origin_host(origin).as_deref(),
        Some("localhost") | Some("127.0.0.1")
    ) && site.contains("localhost")
}

pub fn is_same_site_request(headers: &HeaderMap) -> bool {
    let Some(site) = header_str(headers, "sec-fetch-site") else {
        return false;
    };
    let s = site.to_ascii_lowercase();
    if s == "same-origin" || s == "same-site" {
        return true;
    }
    if s == "cross-site" {
        return false;
    }
    if s == "none" {
        if let Some(origin) = header_str(headers, "origin") {
            return is_our_origin(&origin);
        }
        if let Some(referer) = header_str(headers, "referer") {
            return referer_is_ours(&referer);
        }
        return false;
    }
    false
}

fn referer_is_ours(referer: &str) -> bool {
    let Ok(u) = url::Url::parse(referer) else {
        return false;
    };
    let Some(host) = u.host_str() else {
        return false;
    };
    let host = host.to_ascii_lowercase();
    if let Some(site_host) = origin_host(&site_origin()) {
        if host == site_host {
            return true;
        }
    }
    site_origin().contains("localhost") && matches!(host.as_str(), "localhost" | "127.0.0.1")
}

fn is_extension_or_youtube_origin(headers: &HeaderMap) -> bool {
    let Some(origin) = header_str(headers, "origin") else {
        return false;
    };
    let lower = origin.to_ascii_lowercase();
    if lower.starts_with("chrome-extension://") || lower.starts_with("moz-extension://") {
        return true;
    }
    matches!(
        origin_host(&origin).as_deref(),
        Some("youtube.com")
            | Some("www.youtube.com")
            | Some("m.youtube.com")
            | Some("music.youtube.com")
            | Some("youtu.be")
    )
}

/// CORS allowlist for preflight / ingest responses.
pub fn cors_allow_origin(headers: &HeaderMap) -> Option<String> {
    let origin = header_str(headers, "origin")?;
    if is_our_origin(&origin) || is_extension_or_youtube_origin(headers) {
        Some(origin)
    } else {
        None
    }
}

pub fn honeypot_tripped(value: Option<&str>) -> bool {
    value.is_some_and(|s| !s.trim().is_empty())
}

pub fn busy_message() -> String {
    "Too many transcript requests from this network. Wait a minute and try again.".into()
}

fn busy_for(bucket: &str) -> String {
    match bucket {
        "video720" | "video720_day" => {
            "720p is limited to one download every 30 minutes from this network. Try 480p, or wait."
                .into()
        }
        "videohd" => {
            "1080p and 4K are limited to one download per day from this network. Try 480p or 720p."
                .into()
        }
        "video_sd" | "video_sd_day" => {
            "Too many video downloads from this network. Wait a minute and try again.".into()
        }
        "audio" | "audio_day" => {
            "Too many audio downloads from this network. Wait a minute and try again.".into()
        }
        _ => busy_message(),
    }
}

fn ban_message() -> String {
    "This network is briefly paused after too many failed downloads. Try again later.".into()
}

fn junk_ua(headers: &HeaderMap) -> bool {
    let ua = header_str(headers, "user-agent").unwrap_or_default();
    let u = ua.to_ascii_lowercase();
    if u.is_empty() {
        return true;
    }
    u.starts_with("curl/")
        || u.starts_with("wget/")
        || u.contains("python-requests")
        || u.contains("python-urllib")
        || u.contains("go-http-client")
        || u.contains("scrapy")
        || u.contains("libwww-perl")
        || u.contains("apache-httpclient")
        || u.contains("java/")
        || u == "-"
}

fn banned(ip: &str) -> bool {
    let now = now_ms();
    let st = STATE.lock();
    st.bans.get(ip).is_some_and(|until| *until > now)
}

#[allow(dead_code)]
fn strike(ip: &str) {
    let now = now_ms();
    let mut st = STATE.lock();
    let n = {
        let e = st.strikes.entry(ip.to_string()).or_insert(0);
        *e = e.saturating_add(1);
        *e
    };
    if n >= STRIKE_BAN_AFTER {
        st.bans
            .insert(ip.to_string(), now + BAN_SECS * 1000);
        st.strikes.insert(ip.to_string(), 0);
    }
    st.persist();
}

fn clear_strikes(ip: &str) {
    let mut st = STATE.lock();
    if st.strikes.remove(ip).is_some() {
        st.persist_if_needed();
    }
}

fn ip_hash(ip: &str) -> String {
    hex_encode(&sha256_32(
        &[ip.as_bytes(), b"|", SECRET.as_slice()].concat(),
    ))[..16]
        .to_string()
}

pub fn issue_ticket(ip: &str, cap: TicketCap) -> String {
    let iat = now_secs();
    let exp = iat + TICKET_TTL_SECS;
    let ip8 = ip_hash(ip);
    let cap_s = cap.as_str();
    let mac = hmac_hex(format!("v1|{iat}|{exp}|{cap_s}|{ip8}").as_bytes());
    format!("v1.{iat}.{exp}.{cap_s}.{ip8}.{mac}")
}

pub fn ticket_cookie_header(ticket: &str) -> String {
    let secure = crate::family::public_origin().starts_with("https://");
    let mut c = format!(
        "{TICKET_COOKIE}={ticket}; Path=/; Max-Age={TICKET_TTL_SECS}; HttpOnly; SameSite=Lax"
    );
    if secure {
        c.push_str("; Secure");
    }
    c
}

fn parse_ticket(raw: &str) -> Option<(u64, u64, TicketCap, String, String)> {
    let mut parts = raw.split('.');
    let v = parts.next()?;
    let iat = parts.next()?.parse().ok()?;
    let exp = parts.next()?.parse().ok()?;
    let cap = TicketCap::parse(parts.next()?)?;
    let ip8 = parts.next()?.to_string();
    let mac = parts.next()?.to_string();
    if parts.next().is_some() || v != "v1" {
        return None;
    }
    Some((iat, exp, cap, ip8, mac))
}

pub fn read_ticket(headers: &HeaderMap, ip: &str, need_hd: bool) -> Result<TicketCap, Deny> {
    if is_pro_headers(headers) {
        return Ok(TicketCap::Hd);
    }
    let raw = header_str(headers, "x-forge-ticket")
        .or_else(|| cookie_named(headers, TICKET_COOKIE))
        .ok_or_else(|| {
            Deny::forbidden("Confirm you are not a bot, then try the download again.".to_string())
        })?;
    let (iat, exp, cap, ip8, mac) = parse_ticket(&raw).ok_or_else(|| {
        Deny::forbidden("Confirm you are not a bot, then try the download again.".to_string())
    })?;
    let now = now_secs();
    if exp < now || iat > now + 60 {
        return Err(Deny::forbidden(
            "That download check expired. Try again.".to_string(),
        ));
    }
    let cap_s = cap.as_str();
    if !hmac_ok(format!("v1|{iat}|{exp}|{cap_s}|{ip8}").as_bytes(), &mac) {
        return Err(Deny::forbidden(
            "Confirm you are not a bot, then try the download again.".to_string(),
        ));
    }
    if ip8 != ip_hash(ip) {
        return Err(Deny::forbidden(
            "Confirm you are not a bot, then try the download again.".to_string(),
        ));
    }
    if need_hd && !cap.allows_hd() {
        return Err(Deny::forbidden(
            "1080p and 4K need an extra check. Confirm in the page, then try again.".to_string(),
        ));
    }
    Ok(cap)
}

#[derive(Debug, Clone, Serialize)]
pub struct PowChallenge {
    pub algorithm: &'static str,
    pub salt: String,
    pub challenge: String,
    pub maxnumber: u32,
    pub signature: String,
    pub exp: u64,
    pub kind: String,
}

pub fn pow_kind(raw: Option<&str>) -> &'static str {
    match raw.unwrap_or("media").trim().to_ascii_lowercase().as_str() {
        "hd" | "1080" | "4k" => "hd",
        _ => "media",
    }
}

fn pow_max_for(ip: &str, kind: &str) -> u32 {
    let base = if kind == "hd" {
        POW_HD_MAX
    } else {
        POW_MEDIA_MAX
    };
    let strikes = STATE.lock().strikes.get(ip).copied().unwrap_or(0);
    if strikes >= 3 {
        base.saturating_mul(4).min(1_200_000)
    } else {
        base
    }
}

pub fn issue_pow(headers: &HeaderMap, kind: &str) -> Result<PowChallenge, Deny> {
    check_app_access_deny(headers, CHALLENGE)?;
    let ip = client_ip_from_headers(headers);
    if banned(&ip) {
        return Err(Deny::busy(ban_message(), Some(BAN_SECS)));
    }
    let kind = pow_kind(Some(kind));
    let maxnumber = pow_max_for(&ip, kind);
    let salt = hex_encode(&rand_bytes::<16>());
    let n = rand_u32(maxnumber.max(1));
    let challenge = hex_encode(&Sha256::digest(format!("{salt}{n}").as_bytes()));
    let exp = now_secs() + POW_TTL_SECS;
    let ip8 = ip_hash(&ip);
    let signature = hmac_hex(format!("{salt}:{challenge}:{maxnumber}:{exp}:{kind}:{ip8}").as_bytes());
    Ok(PowChallenge {
        algorithm: "SHA-256",
        salt,
        challenge,
        maxnumber,
        signature,
        exp,
        kind: kind.into(),
    })
}

pub fn verify_pow(
    salt: &str,
    challenge: &str,
    number: u32,
    maxnumber: u32,
    signature: &str,
    exp: u64,
    kind: &str,
    ip: &str,
) -> Result<TicketCap, Deny> {
    let kind = pow_kind(Some(kind));
    let now = now_secs();
    if exp < now {
        return Err(Deny::forbidden("That check expired. Try again.".to_string()));
    }
    if salt.len() != 32
        || challenge.len() != 64
        || !salt.chars().all(|c| c.is_ascii_hexdigit())
        || !challenge.chars().all(|c| c.is_ascii_hexdigit())
    {
        return Err(Deny::forbidden(
            "Confirm you are not a bot, then try again.".to_string(),
        ));
    }
    if maxnumber == 0 || maxnumber > 1_200_000 || number > maxnumber {
        return Err(Deny::forbidden(
            "Confirm you are not a bot, then try again.".to_string(),
        ));
    }
    let ip8 = ip_hash(ip);
    if !hmac_ok(
        format!("{salt}:{challenge}:{maxnumber}:{exp}:{kind}:{ip8}").as_bytes(),
        signature,
    ) {
        return Err(Deny::forbidden(
            "Confirm you are not a bot, then try again.".to_string(),
        ));
    }
    let got = hex_encode(&Sha256::digest(format!("{salt}{number}").as_bytes()));
    if !got.eq_ignore_ascii_case(challenge) {
        return Err(Deny::forbidden(
            "Confirm you are not a bot, then try again.".to_string(),
        ));
    }
    {
        let mut st = STATE.lock();
        if st.nonces.contains_key(challenge) {
            return Err(Deny::forbidden(
                "That check was already used. Try again.".to_string(),
            ));
        }
        st.nonces.insert(challenge.to_string(), exp * 1000);
        st.persist_if_needed();
    }
    Ok(if kind == "hd" {
        TicketCap::Hd
    } else {
        TicketCap::Media
    })
}

pub fn video_quality_tier(quality: &str) -> &'static str {
    match quality {
        "360" | "480" => "sd",
        "720" => "720",
        _ => "hd",
    }
}

pub fn needs_hd_ticket(quality: &str) -> bool {
    video_quality_tier(quality) == "hd"
}

pub fn max_duration_secs(quality: &str) -> u64 {
    match video_quality_tier(quality) {
        "sd" => 3 * 3600,
        "720" => 90 * 60,
        _ => 60 * 60,
    }
}

pub fn max_audio_duration_secs() -> u64 {
    max_duration_secs("360")
}

pub fn max_filesize_arg(quality: &str) -> &'static str {
    match quality {
        "360" | "480" => "1G",
        "720" => "1500M",
        "1080" => "2G",
        _ => "1500M",
    }
}

pub fn quality_label(quality: &str) -> String {
    match quality {
        "best" => "best quality".into(),
        other => format!("{other}p"),
    }
}

fn media_limits(kind: MediaKind, quality: &str) -> Vec<Limit> {
    match kind {
        MediaKind::Audio => vec![AUDIO, AUDIO_DAY],
        MediaKind::Video => match video_quality_tier(quality) {
            "sd" => vec![VIDEO_SD, VIDEO_SD_DAY],
            "720" => vec![VIDEO_720, VIDEO_720_DAY],
            _ => vec![VIDEO_HD],
        },
    }
}

/// Same-site (or API key) + ticket + per-quality quotas.
/// Peek every bucket first; commit all only when `commit` is true (no half-taken 720 slot).
pub fn check_media_access(
    headers: &HeaderMap,
    kind: MediaKind,
    quality: &str,
    commit: bool,
) -> Result<(), Deny> {
    if !(is_same_site_request(headers) || is_pro_headers(headers)) {
        return Err(Deny::hidden());
    }
    if !is_pro_headers(headers) && junk_ua(headers) {
        return Err(Deny::hidden());
    }
    let ip = client_ip_from_headers(headers);
    if banned(&ip) {
        return Err(Deny::busy(ban_message(), Some(BAN_SECS)));
    }

    // Polar subscribers: monthly plan meters (no PoW ticket).
    if let Some(sub) = crate::billing::subscriber_from_headers(headers) {
        let meter = media_meter(kind, quality)?;
        if !commit {
            let cap = sub.plan.meter_cap(meter);
            if cap == 0 {
                return Err(Deny::forbidden(format!(
                    "Your {} plan does not include this download. Upgrade at /pricing.",
                    sub.plan.label()
                )));
            }
            return Ok(());
        }
        crate::billing::take_meter(&sub.id, meter).map_err(Deny::forbidden)?;
        clear_strikes(&ip);
        return Ok(());
    }

    let need_hd = kind == MediaKind::Video && needs_hd_ticket(quality);
    if !is_ops_key(headers) {
        read_ticket(headers, &ip, need_hd)?;
    }
    let limits = media_limits(kind, quality);
    for limit in &limits {
        peek_headers(headers, *limit)?;
    }
    if !commit {
        return Ok(());
    }
    for limit in &limits {
        let (bucket_key, max) = quota_identity(headers, &ip, *limit);
        if let Err(retry) = quota(&bucket_key, limit.bucket, max, limit.window_secs, true) {
            return Err(Deny::busy(busy_for(limit.bucket), Some(retry)));
        }
    }
    clear_strikes(&ip);
    Ok(())
}

fn media_meter(kind: MediaKind, quality: &str) -> Result<crate::billing::Meter, Deny> {
    use crate::billing::Meter;
    match kind {
        MediaKind::Audio => Ok(Meter::Audio),
        MediaKind::Video => match video_quality_tier(quality) {
            "sd" => Ok(Meter::VideoSd),
            "720" => Ok(Meter::Video720),
            _ => Ok(Meter::VideoHd),
        },
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    Audio,
    Video,
}

pub fn deny_status(deny: &Deny) -> axum::http::StatusCode {
    if deny.hidden {
        axum::http::StatusCode::NOT_FOUND
    } else if deny.retry_after.is_some() {
        axum::http::StatusCode::TOO_MANY_REQUESTS
    } else {
        axum::http::StatusCode::FORBIDDEN
    }
}

fn cookie_named(headers: &HeaderMap, name: &str) -> Option<String> {
    let raw = header_str(headers, "cookie")?;
    for part in raw.split(';') {
        let part = part.trim();
        if let Some(v) = part.strip_prefix(&format!("{name}=")) {
            let v = v.trim();
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

fn header_str(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn honeypot_empty_is_ok() {
        assert!(!honeypot_tripped(None));
        assert!(!honeypot_tripped(Some("")));
        assert!(!honeypot_tripped(Some("  ")));
        assert!(honeypot_tripped(Some("http://spam.test")));
    }

    #[test]
    fn rate_limit_trips() {
        let ip = format!("test-{}", now_ms());
        let lim = Limit {
            bucket: "test",
            max: 3,
            pro: 3,
            window_secs: 60,
        };
        assert!(allow(&ip, lim));
        assert!(allow(&ip, lim));
        assert!(allow(&ip, lim));
        assert!(!allow(&ip, lim));
    }

    #[test]
    fn same_site_sec_fetch() {
        let mut h = HeaderMap::new();
        h.insert("sec-fetch-site", "same-origin".parse().unwrap());
        assert!(is_same_site_request(&h));
    }

    #[test]
    fn same_site_rejects_spoofed_referer() {
        let mut h = HeaderMap::new();
        h.insert("sec-fetch-site", "cross-site".parse().unwrap());
        h.insert("referer", "http://localhost/".parse().unwrap());
        assert!(!is_same_site_request(&h));

        let mut h = HeaderMap::new();
        h.insert("referer", "https://forgeyt.com.evil.test/".parse().unwrap());
        assert!(!is_same_site_request(&h));

        let mut h = HeaderMap::new();
        h.insert("origin", "https://forgeyt.com".parse().unwrap());
        assert!(!is_same_site_request(&h));

        let mut h = HeaderMap::new();
        h.insert("referer", "https://forgeyt.com/".parse().unwrap());
        assert!(!is_same_site_request(&h));
    }

    #[test]
    fn rejects_bare_curl() {
        let h = HeaderMap::new();
        assert!(!is_same_site_request(&h));
        assert!(check_app_access(&h, API).is_err());
    }

    #[test]
    fn pow_roundtrip_and_ticket() {
        let salt = hex_encode(&rand_bytes::<16>());
        let n = 42u32;
        let challenge = hex_encode(&Sha256::digest(format!("{salt}{n}").as_bytes()));
        let exp = now_secs() + 60;
        let kind = "media";
        let maxnumber = POW_MEDIA_MAX;
        let ip = "203.0.113.9";
        let ip8 = ip_hash(ip);
        let signature = hmac_hex(format!("{salt}:{challenge}:{maxnumber}:{exp}:{kind}:{ip8}").as_bytes());
        let cap = verify_pow(&salt, &challenge, n, maxnumber, &signature, exp, kind, ip).expect("pow");
        assert_eq!(cap.as_str(), "m");
        let ticket = issue_ticket("203.0.113.9", cap);
        let mut h = HeaderMap::new();
        h.insert(
            "cookie",
            format!("{TICKET_COOKIE}={ticket}").parse().unwrap(),
        );
        let got = read_ticket(&h, "203.0.113.9", false).expect("ticket");
        assert_eq!(got.as_str(), "m");
        assert!(read_ticket(&h, "203.0.113.9", true).is_err());
        assert!(read_ticket(&h, "198.51.100.2", false).is_err());
        assert!(verify_pow(&salt, &challenge, n, maxnumber, &signature, exp, kind, "198.51.100.2").is_err());
        assert_eq!(quality_label("best"), "best quality");
        assert_eq!(quality_label("720"), "720p");
    }

    #[test]
    fn video_tiers() {
        assert_eq!(video_quality_tier("480"), "sd");
        assert_eq!(video_quality_tier("720"), "720");
        assert_eq!(video_quality_tier("1080"), "hd");
        assert_eq!(video_quality_tier("best"), "hd");
        assert_eq!(max_duration_secs("720"), 90 * 60);
    }
}
