//! Lemon Squeezy checkout + HMAC webhook verification.

use axum::http::HeaderMap;
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

use super::plans::Plan;

type HmacSha256 = Hmac<Sha256>;

const LEMON_API: &str = "https://api.lemonsqueezy.com/v1";

pub fn configured() -> bool {
    api_key().is_some()
        && store_id().is_some()
        && variant_id(Plan::Basic).is_some()
        && variant_id(Plan::Pro).is_some()
}

fn api_key() -> Option<String> {
    env_nonempty("LEMON_API_KEY")
}

fn store_id() -> Option<String> {
    env_nonempty("LEMON_STORE_ID")
}

fn webhook_secret() -> Option<String> {
    env_nonempty("LEMON_WEBHOOK_SECRET")
}

pub fn variant_id(plan: Plan) -> Option<String> {
    env_nonempty(plan.lemon_variant_env())
}

fn env_nonempty(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Lemon signs with HMAC-SHA256 hex of the raw body → `X-Signature`.
pub fn verify_webhook(headers: &HeaderMap, body: &[u8]) -> bool {
    let Some(secret) = webhook_secret() else {
        return false;
    };
    let Some(sig) = headers
        .get("x-signature")
        .or_else(|| headers.get("X-Signature"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
    else {
        return false;
    };
    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(body);
    let digest = hex_encode(&mac.finalize().into_bytes());
    // Constant-time-ish compare (same length required).
    if digest.len() != sig.len() {
        return false;
    }
    digest
        .as_bytes()
        .iter()
        .zip(sig.as_bytes())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

fn hex_encode(bytes: &[u8]) -> String {
    const H: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(H[(b >> 4) as usize] as char);
        s.push(H[(b & 0xf) as usize] as char);
    }
    s
}

pub fn new_forge_token() -> String {
    let mut b = [0u8; 16];
    let _ = getrandom::getrandom(&mut b);
    format!("ft_{}", hex_encode(&b))
}

pub async fn create_checkout(plan: Plan, forge_token: &str) -> Result<String, String> {
    let token = api_key().ok_or("Billing is not configured.")?;
    let store = store_id().ok_or("LEMON_STORE_ID missing.")?;
    let variant = variant_id(plan).ok_or("That plan variant is not configured.")?;
    let origin = crate::family::public_origin().trim_end_matches('/').to_string();
    let redirect = format!("{origin}/developers/welcome?t={forge_token}");

    let variant_num: Option<u64> = variant.parse().ok();
    let mut attributes = serde_json::json!({
        "checkout_options": {
            "embed": false,
            "media": false,
            "logo": true,
            "desc": true,
            "discount": true,
            "button_color": "#e85d04"
        },
        "checkout_data": {
            "custom": {
                "forge_token": forge_token,
                "plan": plan.as_str(),
                "app": "forgeyt"
            }
        },
        "product_options": {
            "redirect_url": redirect,
            "receipt_button_text": "Open API docs",
            "receipt_link_url": format!("{origin}/developers")
        }
    });
    if let Some(n) = variant_num {
        attributes["product_options"]["enabled_variants"] = serde_json::json!([n]);
    }

    let body = serde_json::json!({
        "data": {
            "type": "checkouts",
            "attributes": attributes,
            "relationships": {
                "store": { "data": { "type": "stores", "id": store } },
                "variant": { "data": { "type": "variants", "id": variant } }
            }
        }
    });

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{LEMON_API}/checkouts"))
        .bearer_auth(&token)
        .header("Accept", "application/vnd.api+json")
        .header("Content-Type", "application/vnd.api+json")
        .header("User-Agent", "YouTubeForge/1.0")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Could not reach Lemon Squeezy: {e}"))?;
    let status = res.status();
    let text = res.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!("Lemon checkout failed ({status})."));
    }
    let v: serde_json::Value =
        serde_json::from_str(&text).map_err(|_| "Lemon returned bad JSON.".to_string())?;
    v.pointer("/data/attributes/url")
        .and_then(|u| u.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Lemon checkout missing url.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hmac_hex_matches_known() {
        let secret = "test_secret";
        let body = b"{\"meta\":{}}";
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        let dig = hex_encode(&mac.finalize().into_bytes());
        assert_eq!(dig.len(), 64);
    }
}
