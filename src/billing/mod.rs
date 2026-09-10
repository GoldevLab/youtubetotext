//! Lemon Squeezy billing: plans, API keys, checkout, webhooks.

mod lemon;
mod plans;
mod store;

pub use plans::{Meter, Plan};
pub use store::{lookup_raw_key, take_meter, ApiKeyRecord};

use axum::body::Bytes;
use axum::extract::Query;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::Json;
use serde::Deserialize;

use crate::guard;

/// Resolve a paid subscriber from `x-api-key` / Bearer (not ops `FORGE_API_KEYS`).
pub fn subscriber_from_headers(headers: &HeaderMap) -> Option<ApiKeyRecord> {
    let raw = guard::api_key_from_headers(headers)?;
    lookup_raw_key(&raw)
}

pub fn is_paid_headers(headers: &HeaderMap) -> bool {
    subscriber_from_headers(headers).is_some()
}

pub fn billing_ready() -> bool {
    lemon::configured()
}

#[derive(Debug, Deserialize)]
pub struct CheckoutQuery {
    pub plan: Option<String>,
}

pub async fn checkout(Query(q): Query<CheckoutQuery>, _headers: HeaderMap) -> Response {
    if !lemon::configured() {
        return json_err(
            StatusCode::SERVICE_UNAVAILABLE,
            "Subscriptions are not configured yet. Set LEMON_* secrets and try again.",
        );
    }
    let Some(plan) = q.plan.as_deref().and_then(Plan::parse) else {
        return json_err(StatusCode::BAD_REQUEST, "Pass plan=basic or plan=pro.");
    };
    let forge_token = lemon::new_forge_token();
    store::remember_pending(&forge_token, plan);
    match lemon::create_checkout(plan, &forge_token).await {
        Ok(url) => Redirect::temporary(&url).into_response(),
        Err(m) => json_err(StatusCode::BAD_GATEWAY, &m),
    }
}

#[derive(Debug, Deserialize)]
pub struct WelcomeQuery {
    /// forge_token from checkout custom data / redirect
    pub t: Option<String>,
}

/// After Lemon redirect: reveal API key once when webhook has provisioned it.
pub async fn reveal(Query(q): Query<WelcomeQuery>) -> Response {
    let Some(token) = q.t.as_deref().map(str::trim).filter(|s| {
        s.starts_with("ft_") && s.len() >= 12 && s.len() <= 80
            && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    }) else {
        return json_err(StatusCode::BAD_REQUEST, "Missing or invalid t= token.");
    };

    if let Some((api_key, rec)) = store::reveal_for_checkout(token) {
        return Json(serde_json::json!({
            "ok": true,
            "api_key": api_key,
            "prefix": rec.prefix,
            "plan": rec.plan.as_str(),
            "email": rec.email,
            "message": "Copy this key now. It will not be shown again.",
            "usage": usage_json(&rec),
        }))
        .into_response();
    }

    if let Some(rec) = store::peek_for_checkout(token) {
        return Json(serde_json::json!({
            "ok": true,
            "api_key": null,
            "prefix": rec.prefix,
            "plan": rec.plan.as_str(),
            "email": rec.email,
            "revealed": rec.revealed,
            "message": if rec.revealed {
                "This key was already shown. Manage billing in the Lemon Squeezy customer portal (email receipt)."
            } else {
                "Key is provisioning. Refresh in a moment."
            },
            "pending": rec.pending_reveal.is_none() && !rec.revealed,
            "usage": usage_json(&rec),
        }))
        .into_response();
    }

    if store::pending_plan(token).is_some() {
        return Json(serde_json::json!({
            "ok": false,
            "pending": true,
            "message": "Payment received — provisioning your API key. Refresh in a few seconds.",
        }))
        .into_response();
    }

    json_err(
        StatusCode::NOT_FOUND,
        "Unknown checkout token. Open the link from your Lemon receipt or start again at /pricing.",
    )
}

pub async fn webhook(headers: HeaderMap, body: Bytes) -> Response {
    if !lemon::verify_webhook(&headers, &body) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let Ok(event) = serde_json::from_slice::<serde_json::Value>(&body) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let etype = event
        .pointer("/meta/event_name")
        .and_then(|t| t.as_str())
        .unwrap_or("");
    let custom = event.get("meta").and_then(|m| m.get("custom_data"));
    let forge_token = custom
        .and_then(|c| c.get("forge_token"))
        .and_then(|s| s.as_str())
        .map(|s| s.to_string());
    let plan_hint = custom
        .and_then(|c| c.get("plan"))
        .and_then(|s| s.as_str())
        .and_then(Plan::parse);
    let data = event.get("data").cloned().unwrap_or(serde_json::json!({}));

    match etype {
        "subscription_created" | "subscription_updated" | "subscription_resumed"
        | "subscription_unpaused" => {
            handle_subscription(&data, forge_token.as_deref(), plan_hint, true);
        }
        "subscription_expired" | "subscription_cancelled" | "subscription_paused" => {
            // Keep access until period end on cancelled; expire/paused → revoke.
            if etype == "subscription_cancelled" {
                handle_subscription(&data, forge_token.as_deref(), plan_hint, true);
            } else {
                revoke_from_payload(&data);
            }
        }
        "order_created" => {
            // One-shot safety if subscription webhook lags.
            if data
                .pointer("/attributes/status")
                .and_then(|s| s.as_str())
                == Some("paid")
            {
                handle_order(&data, forge_token.as_deref(), plan_hint);
            }
        }
        _ => {}
    }

    StatusCode::OK.into_response()
}

fn revoke_from_payload(data: &serde_json::Value) {
    if let Some(cid) = customer_id(data) {
        store::revoke_customer(&cid);
    }
    if let Some(sid) = data.get("id").and_then(|s| s.as_str()) {
        store::revoke_subscription(sid);
    }
}

fn customer_id(data: &serde_json::Value) -> Option<String> {
    data.pointer("/attributes/customer_id")
        .and_then(|v| {
            v.as_i64()
                .map(|n| n.to_string())
                .or_else(|| v.as_str().map(|s| s.to_string()))
        })
        .or_else(|| {
            data.pointer("/relationships/customer/data/id")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string())
        })
}

fn handle_subscription(
    data: &serde_json::Value,
    forge_token: Option<&str>,
    plan_hint: Option<Plan>,
    active: bool,
) {
    let status = data
        .pointer("/attributes/status")
        .and_then(|s| s.as_str())
        .unwrap_or("");
    let Some(cust) = customer_id(data) else {
        return;
    };
    if !active || matches!(status, "expired" | "unpaid") {
        store::revoke_customer(&cust);
        return;
    }

    let variant = data
        .pointer("/attributes/variant_id")
        .and_then(|v| {
            v.as_i64()
                .map(|n| n.to_string())
                .or_else(|| v.as_str().map(|s| s.to_string()))
        })
        .unwrap_or_default();
    let plan = Plan::from_variant_id(&variant)
        .or(plan_hint)
        .or_else(|| forge_token.and_then(store::pending_plan));
    let Some(plan) = plan else {
        return;
    };

    let email = data
        .pointer("/attributes/user_email")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let sub_id = data.get("id").and_then(|s| s.as_str()).unwrap_or("");
    let period_end = data
        .pointer("/attributes/renews_at")
        .or_else(|| data.pointer("/attributes/ends_at"))
        .and_then(|v| v.as_str())
        .and_then(parse_rfc3339_approx)
        .unwrap_or_else(|| now_plus_days(32));

    let _ = store::upsert_subscription(
        plan,
        &cust,
        sub_id,
        &email,
        &variant,
        period_end,
        forge_token,
    );
}

fn handle_order(data: &serde_json::Value, forge_token: Option<&str>, plan_hint: Option<Plan>) {
    let Some(cust) = customer_id(data) else {
        return;
    };
    let variant = data
        .pointer("/attributes/first_order_item/variant_id")
        .or_else(|| data.pointer("/attributes/variant_id"))
        .and_then(|v| {
            v.as_i64()
                .map(|n| n.to_string())
                .or_else(|| v.as_str().map(|s| s.to_string()))
        })
        .unwrap_or_default();
    let plan = Plan::from_variant_id(&variant)
        .or(plan_hint)
        .or_else(|| forge_token.and_then(store::pending_plan));
    let Some(plan) = plan else {
        return;
    };
    let email = data
        .pointer("/attributes/user_email")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let order_id = data.get("id").and_then(|s| s.as_str()).unwrap_or("order");
    let _ = store::upsert_subscription(
        plan,
        &cust,
        order_id,
        &email,
        &variant,
        now_plus_days(32),
        forge_token,
    );
}

fn now_plus_days(days: u64) -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
        .saturating_add(days.saturating_mul(86_400))
}

fn parse_rfc3339_approx(s: &str) -> Option<u64> {
    // "2026-10-09T12:00:00.000000Z" → rough unix via date-only if needed.
    if let Ok(n) = s.parse::<u64>() {
        return Some(n);
    }
    // Accept YYYY-MM-DD…
    let y = s.get(0..4)?.parse::<i64>().ok()?;
    let mo = s.get(5..7)?.parse::<u64>().ok()?;
    let d = s.get(8..10)?.parse::<u64>().ok()?;
    if !(1970..=2100).contains(&y) || !(1..=12).contains(&mo) || !(1..=31).contains(&d) {
        return None;
    }
    // Days since epoch (approx, ignores leap seconds; good enough for period_end).
    let mut days = 0i64;
    for yy in 1970..y {
        days += if is_leap(yy) { 366 } else { 365 };
    }
    let mdays = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for m in 1..mo {
        days += mdays[m as usize] as i64;
        if m == 2 && is_leap(y) {
            days += 1;
        }
    }
    days += d as i64 - 1;
    Some((days * 86_400) as u64)
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn usage_json(rec: &ApiKeyRecord) -> serde_json::Value {
    serde_json::json!({
        "transcripts": { "used": rec.usage.transcripts, "cap": rec.plan.transcripts_month() },
        "translates": { "used": rec.usage.translates, "cap": rec.plan.translates_month() },
        "audio": { "used": rec.usage.audio, "cap": rec.plan.audio_month() },
        "video_sd": { "used": rec.usage.video_sd, "cap": rec.plan.video_sd_month() },
        "video_720": { "used": rec.usage.video_720, "cap": rec.plan.video_720_month() },
        "period_end": rec.period_end,
        "rate_per_min": rec.plan.rate_per_min(),
    })
}

/// GET /api/v1/me — usage for the calling key.
pub async fn me(headers: HeaderMap) -> Response {
    if let Some(rec) = subscriber_from_headers(&headers) {
        return Json(serde_json::json!({
            "ok": true,
            "plan": rec.plan.as_str(),
            "prefix": rec.prefix,
            "email": rec.email,
            "active": rec.active,
            "usage": usage_json(&rec),
        }))
        .into_response();
    }
    if guard::is_ops_key(&headers) {
        return Json(serde_json::json!({
            "ok": true,
            "plan": "ops",
            "message": "Ops key — unlimited internal access.",
        }))
        .into_response();
    }
    json_err(StatusCode::UNAUTHORIZED, "Pass a valid x-api-key.")
}

fn json_err(status: StatusCode, message: &str) -> Response {
    let mut out = HeaderMap::new();
    out.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json; charset=utf-8"),
    );
    out.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    let body = serde_json::json!({ "error": message }).to_string();
    (status, out, body).into_response()
}
