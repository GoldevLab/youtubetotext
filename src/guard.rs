//! Rate limits and a hidden honeypot — cheaper than a captcha on every paste.
//!
//! Shareable `/?v=` links stay captcha-free. Scrapers that hammer YouTube through
//! this app get 429. Optional Cloudflare Turnstile on the home form when
//! `TURNSTILE_SITE_KEY` + `TURNSTILE_SECRET` are set.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use axum::http::HeaderMap;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use resuma::prelude::FlowRequest;

const WINDOW: Duration = Duration::from_secs(60);
const MAX_KEYS: usize = 24_000;

static BUCKETS: Lazy<Mutex<HashMap<String, Vec<Instant>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Clone, Copy)]
pub struct Limit {
    pub bucket: &'static str,
    pub max: usize,
    pub pro: usize,
}

pub const PAGE: Limit = Limit {
    bucket: "page",
    max: 40,
    pro: 40,
};
pub const API: Limit = Limit {
    bucket: "api",
    max: 24,
    pro: 240,
};
pub const AUDIO: Limit = Limit {
    bucket: "audio",
    max: 10,
    pro: 80,
};
pub const VIDEO: Limit = Limit {
    bucket: "video",
    max: 6,
    pro: 40,
};
pub const TRANSLATE: Limit = Limit {
    bucket: "translate",
    max: 200,
    pro: 800,
};
pub const INGEST: Limit = Limit {
    bucket: "ingest",
    max: 12,
    pro: 60,
};

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

pub fn is_pro_headers(headers: &HeaderMap) -> bool {
    let Some(provided) = api_key_from_headers(headers) else {
        return false;
    };
    configured_api_keys().iter().any(|k| k == &provided)
}

pub fn allow(ip: &str, limit: Limit) -> bool {
    allow_n(ip, limit.bucket, limit.max)
}

fn allow_n(ip: &str, bucket: &str, max: usize) -> bool {
    let now = Instant::now();
    let key = format!("{bucket}|{ip}");
    let mut map = BUCKETS.lock();
    if map.len() >= MAX_KEYS {
        map.retain(|_, hits| hits.last().is_some_and(|t| now.duration_since(*t) < WINDOW));
        if map.len() >= MAX_KEYS {
            map.clear();
        }
    }
    let hits = map.entry(key).or_default();
    hits.retain(|t| now.duration_since(*t) < WINDOW);
    if hits.len() >= max {
        return false;
    }
    hits.push(now);
    true
}

pub fn client_ip_from_headers(headers: &HeaderMap) -> String {
    header_str(headers, "fly-client-ip")
        .or_else(|| header_str(headers, "cf-connecting-ip"))
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
        .or_else(|| req.header("cf-connecting-ip"))
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
    if allow(&client_ip(req), limit) {
        Ok(())
    } else {
        Err(busy_message())
    }
}

pub fn check_headers(headers: &HeaderMap, limit: Limit) -> Result<(), String> {
    let max = if is_pro_headers(headers) {
        limit.pro
    } else {
        limit.max
    };
    if allow_n(&client_ip_from_headers(headers), limit.bucket, max) {
        Ok(())
    } else {
        Err(busy_message())
    }
}

pub fn honeypot_tripped(value: Option<&str>) -> bool {
    value.is_some_and(|s| !s.trim().is_empty())
}

pub fn busy_message() -> String {
    "Too many transcript requests from this network. Wait a minute and try again."
        .into()
}

pub fn turnstile_site_key() -> Option<String> {
    std::env::var("TURNSTILE_SITE_KEY")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub async fn verify_turnstile(token: Option<&str>) -> Result<(), String> {
    if turnstile_site_key().is_none() {
        return Ok(());
    }
    let Ok(secret) = std::env::var("TURNSTILE_SECRET") else {
        return Ok(());
    };
    let secret = secret.trim();
    if secret.is_empty() {
        return Ok(());
    }
    let token = token
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "Confirm you are not a bot, then try again.".to_string())?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
        .map_err(|_| "Captcha check failed.".to_string())?;
    let resp = client
        .post("https://challenges.cloudflare.com/turnstile/v0/siteverify")
        .form(&[("secret", secret), ("response", token)])
        .send()
        .await
        .map_err(|_| "Captcha check failed.".to_string())?;
    let v: serde_json::Value = resp
        .json()
        .await
        .map_err(|_| "Captcha check failed.".to_string())?;
    if v.get("success").and_then(|x| x.as_bool()) == Some(true) {
        Ok(())
    } else {
        Err("Confirm you are not a bot, then try again.".into())
    }
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
        let ip = format!("test-{}", Instant::now().elapsed().as_nanos());
        let lim = Limit {
            bucket: "test",
            max: 3,
            pro: 3,
        };
        assert!(allow(&ip, lim));
        assert!(allow(&ip, lim));
        assert!(allow(&ip, lim));
        assert!(!allow(&ip, lim));
    }
}
