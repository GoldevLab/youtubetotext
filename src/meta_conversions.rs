//! Meta Conversions API (CAPI) — server-side twin of the browser Pixel.
//!
//! Env: `META_PIXEL_ID` + `PRIVATE_PIXEL_TOKEN` (optional `META_TEST_EVENT_CODE`).
//! Payload shape matches Meta’s Payload Helper / ACUPATAS CAPI module.
//! https://developers.facebook.com/docs/marketing-api/conversions-api/payload-helper

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::http::HeaderMap;
use serde::Deserialize;
use serde_json::{json, Value};

const API_VERSION: &str = "v21.0";

#[derive(Debug, Clone, Default)]
pub struct MetaBrowserHints {
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub fbp: Option<String>,
    pub fbc: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ViewContentBody {
    pub event_id: String,
    pub v: Option<String>,
    pub event_source_url: Option<String>,
    pub fbp: Option<String>,
    pub fbc: Option<String>,
}

pub fn pixel_config() -> Option<(String, String)> {
    let pixel_id = std::env::var("META_PIXEL_ID")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|id| !id.is_empty() && id.len() <= 20 && id.bytes().all(|b| b.is_ascii_digit()))?;
    let token = std::env::var("PRIVATE_PIXEL_TOKEN")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|t| t.len() >= 20)?;
    Some((pixel_id, token))
}

pub fn is_configured() -> bool {
    pixel_config().is_some()
}

fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    let cookie = headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())?;
    for part in cookie.split(';') {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix(name) {
            if let Some(val) = rest.strip_prefix('=') {
                let v = val.trim();
                if !v.is_empty() {
                    return Some(v.to_string());
                }
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

fn pick_fbc(headers: &HeaderMap, body_fbc: Option<&str>, source_url: &str) -> Option<String> {
    if let Some(v) = body_fbc.map(str::trim).filter(|s| !s.is_empty()) {
        return Some(v.to_string());
    }
    if let Some(v) = cookie_value(headers, "_fbc") {
        return Some(v);
    }
    let fbclid = url::Url::parse(source_url)
        .ok()
        .and_then(|u| {
            u.query_pairs()
                .find(|(k, _)| k == "fbclid")
                .map(|(_, v)| v.into_owned())
        })
        .filter(|s| !s.is_empty())?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    Some(format!("fb.1.{ts}.{fbclid}"))
}

pub fn hints_from_headers(headers: &HeaderMap, body: &ViewContentBody) -> MetaBrowserHints {
    let source = body
        .event_source_url
        .as_deref()
        .unwrap_or("")
        .trim();
    MetaBrowserHints {
        client_ip: {
            let ip = crate::guard::client_ip_from_headers(headers);
            if ip == "unknown" {
                None
            } else {
                Some(ip)
            }
        },
        user_agent: header_str(headers, "user-agent"),
        fbp: body
            .fbp
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .or_else(|| cookie_value(headers, "_fbp")),
        fbc: pick_fbc(headers, body.fbc.as_deref(), source),
    }
}

pub fn valid_event_id(raw: &str) -> Option<&str> {
    let s = raw.trim();
    if s.len() < 8 || s.len() > 64 {
        return None;
    }
    if s
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        Some(s)
    } else {
        None
    }
}

fn build_user_data(hints: &MetaBrowserHints) -> Value {
    let mut user_data = serde_json::Map::new();
    if let Some(ip) = hints.client_ip.as_deref() {
        user_data.insert("client_ip_address".into(), json!(ip));
    }
    if let Some(ua) = hints.user_agent.as_deref() {
        user_data.insert("client_user_agent".into(), json!(ua));
    }
    if let Some(fbp) = hints.fbp.as_deref() {
        user_data.insert("fbp".into(), json!(fbp));
    }
    if let Some(fbc) = hints.fbc.as_deref() {
        user_data.insert("fbc".into(), json!(fbc));
    }
    Value::Object(user_data)
}

fn build_view_content_event(
    event_id: &str,
    source_url: &str,
    video_id: &str,
    hints: &MetaBrowserHints,
) -> Value {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    json!({
        "event_name": "ViewContent",
        "event_time": now,
        "event_id": event_id,
        "action_source": "website",
        "event_source_url": source_url.chars().take(2048).collect::<String>(),
        "user_data": build_user_data(hints),
        "custom_data": {
            "content_name": "transcript",
            "content_type": "product",
            "content_ids": [video_id],
        }
    })
}

async fn post_capi(events: Vec<Value>) {
    let Some((pixel_id, token)) = pixel_config() else {
        return;
    };
    if events.is_empty() {
        return;
    }
    let url = format!("https://graph.facebook.com/{API_VERSION}/{pixel_id}/events");
    let mut body = json!({
        "data": events,
        "access_token": token,
    });
    if let Ok(code) = std::env::var("META_TEST_EVENT_CODE") {
        let code = code.trim();
        if !code.is_empty() && code.len() < 64 {
            body
                .as_object_mut()
                .expect("object")
                .insert("test_event_code".into(), json!(code));
        }
    }
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .connect_timeout(Duration::from_secs(4))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[meta-conversions] client: {e}");
            return;
        }
    };
    match client.post(&url).json(&body).send().await {
        Ok(res) if res.status().is_success() => {}
        Ok(res) => {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            eprintln!(
                "[meta-conversions] CAPI failed: {status} {}",
                text.chars().take(500).collect::<String>()
            );
        }
        Err(e) => eprintln!("[meta-conversions] CAPI error: {e}"),
    }
}

/// Fire ViewContent to Meta CAPI (dedupe with Pixel via shared `event_id`).
pub async fn track_view_content(
    headers: &HeaderMap,
    body: ViewContentBody,
) -> Result<(), &'static str> {
    if pixel_config().is_none() {
        return Ok(());
    }
    let event_id = valid_event_id(&body.event_id).ok_or("invalid event_id")?;
    let raw_v = body.v.as_deref().unwrap_or("").trim();
    let video_id = crate::parse::parse_video_id(raw_v).ok_or("invalid video id")?;
    let origin = crate::family::public_origin().trim_end_matches('/').to_string();
    let source_url = body
        .event_source_url
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && s.len() <= 2048)
        .filter(|s| {
            s.starts_with(&origin)
                || s.starts_with("http://localhost")
                || s.starts_with("http://127.0.0.1")
        })
        .map(str::to_string)
        .unwrap_or_else(|| format!("{origin}/?v={video_id}"));
    let hints = hints_from_headers(headers, &body);
    let event = build_view_content_event(event_id, &source_url, &video_id, &hints);
    post_capi(vec![event]).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn event_id_rules() {
        assert!(valid_event_id("short").is_none());
        assert_eq!(
            valid_event_id("vc_abcd-1234-5678"),
            Some("vc_abcd-1234-5678")
        );
        assert!(valid_event_id("bad id!").is_none());
    }

    #[test]
    fn reads_fbp_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::COOKIE,
            HeaderValue::from_static("_ga=1; _fbp=fb.1.123.456; other=x"),
        );
        let body = ViewContentBody {
            event_id: "vc_test_event_01".into(),
            v: Some("dQw4w9WgXcQ".into()),
            event_source_url: None,
            fbp: None,
            fbc: None,
        };
        let hints = hints_from_headers(&headers, &body);
        assert_eq!(hints.fbp.as_deref(), Some("fb.1.123.456"));
    }
}
