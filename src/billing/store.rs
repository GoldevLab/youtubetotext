//! Persistent API keys + monthly usage. Keys are hashed at rest; plaintext
//! is held only until the welcome page reveals it once.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::plans::{Meter, Plan};

static STORE: Lazy<Mutex<KeyStore>> = Lazy::new(|| Mutex::new(KeyStore::load()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyRecord {
    pub id: String,
    pub key_hash: String,
    pub prefix: String,
    pub plan: Plan,
    pub active: bool,
    pub customer_id: String,
    pub subscription_id: String,
    pub email: String,
    pub product_id: String,
    pub period_end: u64,
    pub usage: UsageCounters,
    pub created_at: u64,
    pub updated_at: u64,
    /// Plaintext key until first reveal (never logged).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending_reveal: Option<String>,
    #[serde(default)]
    pub revealed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkout_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageCounters {
    pub transcripts: u32,
    pub translates: u32,
    pub audio: u32,
    pub video_sd: u32,
    pub video_720: u32,
    pub video_hd: u32,
}

impl UsageCounters {
    fn get(&self, meter: Meter) -> u32 {
        match meter {
            Meter::Transcript => self.transcripts,
            Meter::Translate => self.translates,
            Meter::Audio => self.audio,
            Meter::VideoSd => self.video_sd,
            Meter::Video720 => self.video_720,
            Meter::VideoHd => self.video_hd,
        }
    }

    fn bump(&mut self, meter: Meter) {
        match meter {
            Meter::Transcript => self.transcripts = self.transcripts.saturating_add(1),
            Meter::Translate => self.translates = self.translates.saturating_add(1),
            Meter::Audio => self.audio = self.audio.saturating_add(1),
            Meter::VideoSd => self.video_sd = self.video_sd.saturating_add(1),
            Meter::Video720 => self.video_720 = self.video_720.saturating_add(1),
            Meter::VideoHd => self.video_hd = self.video_hd.saturating_add(1),
        }
    }

    fn reset(&mut self) {
        *self = UsageCounters::default();
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct KeyStore {
    keys: HashMap<String, ApiKeyRecord>,
    /// checkout_id / forge_token → key id
    by_checkout: HashMap<String, String>,
    /// customer_id → key id
    by_customer: HashMap<String, String>,
    /// key_hash → key id
    by_hash: HashMap<String, String>,
    /// forge_token → plan (awaiting Lemon webhook)
    #[serde(default)]
    pending: HashMap<String, PendingCheckout>,
    #[serde(skip)]
    dirty: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PendingCheckout {
    plan: Plan,
    created_at: u64,
}

impl KeyStore {
    fn load() -> Self {
        let path = store_path();
        let mut st = fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(KeyStore::default);
        st.rebuild_indexes();
        st.dirty = 0;
        st
    }

    fn rebuild_indexes(&mut self) {
        self.by_checkout.clear();
        self.by_customer.clear();
        self.by_hash.clear();
        for (id, rec) in &self.keys {
            if let Some(cid) = &rec.checkout_id {
                self.by_checkout.insert(cid.clone(), id.clone());
            }
            if !rec.customer_id.is_empty() {
                self.by_customer.insert(rec.customer_id.clone(), id.clone());
            }
            self.by_hash.insert(rec.key_hash.clone(), id.clone());
        }
    }

    fn persist_if_needed(&mut self) {
        self.dirty = self.dirty.saturating_add(1);
        if self.dirty < 4 {
            return;
        }
        self.persist();
    }

    fn persist(&mut self) {
        self.dirty = 0;
        let path = store_path();
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let tmp = path.with_extension("json.tmp");
        if let Ok(body) = serde_json::to_string_pretty(self) {
            if fs::write(&tmp, body).is_ok() {
                let _ = fs::rename(&tmp, &path);
            }
        }
    }
}

fn data_dir() -> PathBuf {
    PathBuf::from(std::env::var("RESUMA_DATA_DIR").unwrap_or_else(|_| ".resuma".into()))
}

fn store_path() -> PathBuf {
    data_dir().join("api-keys.json")
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn hash_key(raw: &str) -> String {
    let mut h = Sha256::new();
    h.update(raw.as_bytes());
    h.update(b"|forge-api-key-v1");
    hex::encode(h.finalize())
}

mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        const H: &[u8; 16] = b"0123456789abcdef";
        let mut s = String::with_capacity(bytes.as_ref().len() * 2);
        for b in bytes.as_ref() {
            s.push(H[(b >> 4) as usize] as char);
            s.push(H[(b & 0xf) as usize] as char);
        }
        s
    }
}

fn generate_key() -> (String, String, String) {
    let mut raw = [0u8; 24];
    let _ = getrandom::getrandom(&mut raw);
    let secret = hex::encode(raw);
    let full = format!("forge_sk_live_{secret}");
    let prefix = format!("forge_sk_live_{}", &secret[..8]);
    let id = {
        let mut b = [0u8; 8];
        let _ = getrandom::getrandom(&mut b);
        format!("key_{}", hex::encode(b))
    };
    (id, full, prefix)
}

pub fn remember_pending(forge_token: &str, plan: Plan) {
    let mut st = STORE.lock();
    st.pending.insert(
        forge_token.to_string(),
        PendingCheckout {
            plan,
            created_at: now_secs(),
        },
    );
    // Drop stale pendings (>48h).
    let cut = now_secs().saturating_sub(48 * 86_400);
    st.pending.retain(|_, p| p.created_at >= cut);
    st.persist();
}

pub fn pending_plan(forge_token: &str) -> Option<Plan> {
    let st = STORE.lock();
    st.pending.get(forge_token).map(|p| p.plan)
}

/// Upsert an active subscription key for a Polar customer.
pub fn upsert_subscription(
    plan: Plan,
    customer_id: &str,
    subscription_id: &str,
    email: &str,
    product_id: &str,
    period_end: u64,
    checkout_id: Option<&str>,
) -> ApiKeyRecord {
    let now = now_secs();
    let mut st = STORE.lock();
    if let Some(id) = st.by_customer.get(customer_id).cloned() {
        if let Some(rec) = st.keys.get_mut(&id) {
            let plan_changed = rec.plan != plan;
            rec.plan = plan;
            rec.active = true;
            rec.subscription_id = subscription_id.to_string();
            rec.email = email.to_string();
            rec.product_id = product_id.to_string();
            rec.period_end = period_end.max(now + 86_400);
            rec.updated_at = now;
            if plan_changed {
                rec.usage.reset();
            }
            if let Some(cid) = checkout_id {
                rec.checkout_id = Some(cid.to_string());
            }
            let out = rec.clone();
            if let Some(cid) = checkout_id {
                st.by_checkout.insert(cid.to_string(), id.clone());
            }
            st.persist();
            return out;
        }
    }

    let (id, full, prefix) = generate_key();
    let rec = ApiKeyRecord {
        id: id.clone(),
        key_hash: hash_key(&full),
        prefix,
        plan,
        active: true,
        customer_id: customer_id.to_string(),
        subscription_id: subscription_id.to_string(),
        email: email.to_string(),
        product_id: product_id.to_string(),
        period_end: period_end.max(now + 30 * 86_400),
        usage: UsageCounters::default(),
        created_at: now,
        updated_at: now,
        pending_reveal: Some(full),
        revealed: false,
        checkout_id: checkout_id.map(|s| s.to_string()),
    };
    st.by_hash.insert(rec.key_hash.clone(), id.clone());
    st.by_customer.insert(customer_id.to_string(), id.clone());
    if let Some(cid) = checkout_id {
        st.by_checkout.insert(cid.to_string(), id.clone());
    }
    st.keys.insert(id, rec.clone());
    st.persist();
    rec
}

pub fn revoke_customer(customer_id: &str) {
    let mut st = STORE.lock();
    if let Some(id) = st.by_customer.get(customer_id).cloned() {
        if let Some(rec) = st.keys.get_mut(&id) {
            rec.active = false;
            rec.updated_at = now_secs();
            rec.pending_reveal = None;
            st.persist();
        }
    }
}

pub fn revoke_subscription(subscription_id: &str) {
    let mut st = STORE.lock();
    let ids: Vec<String> = st
        .keys
        .iter()
        .filter(|(_, r)| r.subscription_id == subscription_id)
        .map(|(id, _)| id.clone())
        .collect();
    for id in ids {
        if let Some(rec) = st.keys.get_mut(&id) {
            rec.active = false;
            rec.updated_at = now_secs();
            rec.pending_reveal = None;
        }
    }
    st.persist();
}

fn roll_period_if_needed(rec: &mut ApiKeyRecord) {
    let now = now_secs();
    if rec.period_end > 0 && now >= rec.period_end {
        rec.usage.reset();
        // Soft roll +30d until Polar webhook refreshes period_end.
        rec.period_end = now + 30 * 86_400;
        rec.updated_at = now;
    }
}

pub fn lookup_raw_key(raw: &str) -> Option<ApiKeyRecord> {
    if raw.len() < 24 || !raw.starts_with("forge_sk_") {
        return None;
    }
    let hash = hash_key(raw);
    let mut st = STORE.lock();
    let id = st.by_hash.get(&hash).cloned()?;
    let rec = st.keys.get_mut(&id)?;
    if !rec.active {
        return None;
    }
    roll_period_if_needed(rec);
    let out = rec.clone();
    st.persist_if_needed();
    Some(out)
}

pub fn take_meter(key_id: &str, meter: Meter) -> Result<ApiKeyRecord, String> {
    let mut st = STORE.lock();
    let rec = st
        .keys
        .get_mut(key_id)
        .ok_or_else(|| "API key not found.".to_string())?;
    if !rec.active {
        return Err("This API key is inactive. Renew your subscription.".into());
    }
    roll_period_if_needed(rec);
    let cap = rec.plan.meter_cap(meter);
    if cap == 0 {
        return Err(format!(
            "Your {} plan does not include this endpoint. Upgrade at /pricing.",
            rec.plan.label()
        ));
    }
    if rec.usage.get(meter) >= cap {
        return Err(format!(
            "Monthly {} quota reached for {} ({}). Upgrade or wait for the next billing cycle.",
            meter_name(meter),
            rec.plan.label(),
            cap
        ));
    }
    rec.usage.bump(meter);
    rec.updated_at = now_secs();
    let out = rec.clone();
    st.persist_if_needed();
    Ok(out)
}

fn meter_name(m: Meter) -> &'static str {
    match m {
        Meter::Transcript => "transcript",
        Meter::Translate => "translate",
        Meter::Audio => "audio",
        Meter::VideoSd => "video SD",
        Meter::Video720 => "video 720p",
        Meter::VideoHd => "video HD",
    }
}

/// Reveal plaintext once for the welcome page after a paid checkout.
pub fn reveal_for_checkout(checkout_id: &str) -> Option<(String, ApiKeyRecord)> {
    let mut st = STORE.lock();
    let id = st.by_checkout.get(checkout_id).cloned()?;
    let rec = st.keys.get_mut(&id)?;
    if !rec.active {
        return None;
    }
    let key = rec.pending_reveal.clone()?;
    rec.pending_reveal = None;
    rec.revealed = true;
    rec.updated_at = now_secs();
    let out = rec.clone();
    st.persist();
    Some((key, out))
}

pub fn peek_for_checkout(checkout_id: &str) -> Option<ApiKeyRecord> {
    let st = STORE.lock();
    let id = st.by_checkout.get(checkout_id)?;
    st.keys.get(id).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_stable() {
        assert_eq!(hash_key("abc"), hash_key("abc"));
        assert_ne!(hash_key("abc"), hash_key("abd"));
    }

    #[test]
    fn upsert_and_lookup() {
        let cust = format!("cust_test_{}", now_secs());
        let rec = upsert_subscription(
            Plan::Basic,
            &cust,
            "sub_test",
            "a@b.test",
            "prod_basic",
            now_secs() + 86400,
            Some("chk_test"),
        );
        assert!(rec.pending_reveal.is_some());
        let raw = rec.pending_reveal.clone().unwrap();
        let found = lookup_raw_key(&raw).expect("lookup");
        assert_eq!(found.plan, Plan::Basic);
        assert!(take_meter(&found.id, Meter::Transcript).is_ok());
        revoke_customer(&cust);
        assert!(lookup_raw_key(&raw).is_none());
    }
}
