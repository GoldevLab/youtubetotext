//! Public JSON/text transcript API.

use axum::extract::Query;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

use crate::export::{as_markdown, as_srt, as_txt, as_vtt};
use crate::parse::parse_video_id;
use crate::youtube::{ingest_client_doc, load_transcript, Cue};

#[derive(Debug, Deserialize)]
pub struct ApiQuery {
    pub v: Option<String>,
    pub url: Option<String>,
    pub lang: Option<String>,
    pub tlang: Option<String>,
    pub fmt: Option<String>,
}

pub async fn transcript(Query(q): Query<ApiQuery>) -> impl IntoResponse {
    let raw = q
        .v
        .as_deref()
        .or(q.url.as_deref())
        .unwrap_or("")
        .trim()
        .to_string();
    let Some(id) = parse_video_id(&raw) else {
        return json_error(
            StatusCode::BAD_REQUEST,
            "Pass v=VIDEO_ID or a YouTube url=…",
        );
    };
    let lang = q.lang.as_deref().filter(|s| !s.is_empty());
    let tlang = q.tlang.as_deref().filter(|s| !s.is_empty());
    let fmt = q.fmt.as_deref().unwrap_or("json").to_ascii_lowercase();

    let doc = match load_transcript(&id, lang, tlang).await {
        Ok(d) => d,
        Err(e) => {
            let status = StatusCode::from_u16(e.status).unwrap_or(StatusCode::BAD_GATEWAY);
            return json_error(status, &e.message);
        }
    };

    match fmt.as_str() {
        "json" => {
            let body = serde_json::to_string_pretty(doc.as_ref()).unwrap_or_else(|_| "{}".into());
            file_response(
                StatusCode::OK,
                "application/json; charset=utf-8",
                None,
                body,
                true,
            )
        }
        "txt" | "text" => file_response(
            StatusCode::OK,
            "text/plain; charset=utf-8",
            Some(&format!("{id}.txt")),
            as_txt(&doc, false),
            true,
        ),
        "srt" => file_response(
            StatusCode::OK,
            "application/x-subrip; charset=utf-8",
            Some(&format!("{id}.srt")),
            as_srt(&doc.cues),
            true,
        ),
        "vtt" => file_response(
            StatusCode::OK,
            "text/vtt; charset=utf-8",
            Some(&format!("{id}.vtt")),
            as_vtt(&doc.cues),
            true,
        ),
        "md" | "markdown" => file_response(
            StatusCode::OK,
            "text/markdown; charset=utf-8",
            Some(&format!("{id}.md")),
            as_markdown(&doc),
            true,
        ),
        "timed" | "txt-timed" => file_response(
            StatusCode::OK,
            "text/plain; charset=utf-8",
            Some(&format!("{id}-timed.txt")),
            as_txt(&doc, true),
            true,
        ),
        _ => json_error(
            StatusCode::BAD_REQUEST,
            "fmt must be json, txt, srt, vtt, md, or timed.",
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct IngestBody {
    pub video_id: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub duration_secs: Option<u64>,
    pub lang: Option<String>,
    pub kind: Option<String>,
    pub tlang: Option<String>,
    pub cues: Vec<Cue>,
}

/// Captions fetched in the user's browser (extension) — bypasses server IP blocks.
pub async fn ingest(Json(body): Json<IngestBody>) -> impl IntoResponse {
    match ingest_client_doc(
        body.video_id,
        body.title.unwrap_or_default(),
        body.author.unwrap_or_default(),
        body.duration_secs.unwrap_or(0),
        body.lang.unwrap_or_default(),
        body.kind.unwrap_or_default(),
        body.tlang.unwrap_or_default(),
        body.cues,
    ) {
        Ok(doc) => {
            let payload = serde_json::json!({ "ok": true, "video_id": doc.video_id, "cues": doc.cues.len() })
                .to_string();
            file_response(
                StatusCode::OK,
                "application/json; charset=utf-8",
                None,
                payload,
                false,
            )
        }
        Err(e) => {
            let status = StatusCode::from_u16(e.status).unwrap_or(StatusCode::BAD_REQUEST);
            json_error(status, &e.message)
        }
    }
}

pub async fn preflight() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    cors_headers(&mut headers);
    (StatusCode::NO_CONTENT, headers)
}

fn json_error(status: StatusCode, message: &str) -> (StatusCode, HeaderMap, String) {
    let body = serde_json::json!({ "error": message }).to_string();
    file_response(status, "application/json; charset=utf-8", None, body, false)
}

fn file_response(
    status: StatusCode,
    content_type: &str,
    filename: Option<&str>,
    body: String,
    cacheable: bool,
) -> (StatusCode, HeaderMap, String) {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(content_type).unwrap_or(HeaderValue::from_static("text/plain")),
    );
    cors_headers(&mut headers);
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(if cacheable {
            "public, max-age=120"
        } else {
            "no-store"
        }),
    );
    if let Some(name) = filename {
        if let Ok(v) = HeaderValue::from_str(&format!("attachment; filename=\"{name}\"")) {
            headers.insert(header::CONTENT_DISPOSITION, v);
        }
    }
    (status, headers, body)
}

fn cors_headers(headers: &mut HeaderMap) {
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("GET, POST, OPTIONS"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("content-type"),
    );
}
