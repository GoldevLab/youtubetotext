//! Google AdSense. Layout is always reserved; live `<ins>` units render when
//! `ADSENSE_CLIENT` plus a slot id are set.
//!
//! Slot lookup (first match): `ADSENSE_SLOT_{PLACEMENT}` (hyphens → `_`),
//! then `ADSENSE_SLOT_{SIZE}` (`leaderboard` | `infeed` | `rectangle`),
//! then `ADSENSE_SLOT` (one unit for every placement).

use resuma::prelude::*;
use resuma::server::CspConfig;

const CLIENT_ENV: &str = "ADSENSE_CLIENT";

/// Scripts, pixels, XHR, and ad frames Google AdSense loads.
const ADSENSE_ORIGINS: &[&str] = &[
    "https://pagead2.googlesyndication.com",
    "https://googleads.g.doubleclick.net",
    "https://tpc.googlesyndication.com",
    "https://www.google.com",
    "https://www.gstatic.com",
    "https://www.googleadservices.com",
    "https://adservice.google.com",
    "https://www.googletagservices.com",
    "https://partner.googleadservices.com",
    "https://ep1.adtrafficquality.google",
    "https://ep2.adtrafficquality.google",
    "https://fundingchoicesmessages.google.com",
];

/// Thumbnails, IFrame API, and nocookie embed host.
const YOUTUBE_ORIGINS: &[&str] = &[
    "https://www.youtube.com",
    "https://youtube.com",
    "https://www.youtube-nocookie.com",
    "https://i.ytimg.com",
    "https://i9.ytimg.com",
    "https://s.ytimg.com",
];

pub fn client_id() -> Option<String> {
    std::env::var(CLIENT_ENV)
        .ok()
        .as_deref()
        .and_then(sanitize_client)
}

fn sanitize_client(raw: &str) -> Option<String> {
    let s = raw.trim();
    let digits = s.strip_prefix("ca-pub-")?;
    if digits.len() >= 10 && digits.bytes().all(|b| b.is_ascii_digit()) {
        Some(s.to_string())
    } else {
        None
    }
}

fn sanitize_slot(raw: &str) -> Option<String> {
    let s = raw.trim();
    if !s.is_empty() && s.len() <= 22 && s.bytes().all(|b| b.is_ascii_digit()) {
        Some(s.to_string())
    } else {
        None
    }
}

fn env_slot(name: &str) -> Option<String> {
    std::env::var(name).ok().as_deref().and_then(sanitize_slot)
}

fn slot_id(placement: &str, size: &str) -> Option<String> {
    let specific = format!(
        "ADSENSE_SLOT_{}",
        placement.replace('-', "_").to_ascii_uppercase()
    );
    env_slot(&specific)
        .or_else(|| env_slot(&format!("ADSENSE_SLOT_{}", size.trim().to_ascii_uppercase())))
        .or_else(|| env_slot("ADSENSE_SLOT"))
}

pub fn head_snippet() -> String {
    match client_id() {
        Some(id) => format!(
            r#"<link rel="preconnect" href="https://pagead2.googlesyndication.com" crossorigin="anonymous" />
<link rel="preconnect" href="https://googleads.g.doubleclick.net" crossorigin="anonymous" />
<script async src="https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js?client={id}" crossorigin="anonymous"></script>
<script type="module" src="/js/youtubetotext-ads.js"></script>"#
        ),
        None => r#"<script type="module" src="/js/youtubetotext-ads.js"></script>"#.into(),
    }
}

pub fn ads_txt() -> Option<String> {
    let client = client_id()?;
    let pub_id = client.strip_prefix("ca-")?;
    Some(format!("google.com, {pub_id}, DIRECT, f08c47fec0942fa0\n"))
}

/// Allow YouTube embeds + AdSense. Resuma 1.3.1 has no `frame-src` knob, so the
/// policy stays report-only: enforcing `default-src 'self'` would blank the
/// player and the ad iframes. Host allowlists are still emitted so a later
/// Resuma with `frame-src` can enforce without another app change.
pub fn apply_csp(csp: &mut CspConfig) {
    // `'strict-dynamic'` ignores host allowlists; adsbygoogle.js in `<head>`
    // is not nonce'd by Resuma, so it would never run.
    csp.strict_dynamic = false;
    for origin in YOUTUBE_ORIGINS.iter().chain(ADSENSE_ORIGINS) {
        push_unique(&mut csp.script_src, origin);
        push_unique(&mut csp.img_src, origin);
        push_unique(&mut csp.connect_src, origin);
        push_unique(&mut csp.style_src, origin);
    }
    csp.report_only = true;
}

fn push_unique(list: &mut Vec<String>, origin: &str) {
    if !list.iter().any(|s| s == origin) {
        list.push(origin.to_string());
    }
}

/// Reserved slot. Live AdSense when publisher + unit IDs are set.
pub fn slot(placement: &'static str, size: &'static str) -> View {
    let class = format!("ad-slot ad-slot-{size}");
    let live = client_id().zip(slot_id(placement, size));
    let lazy = size == "rectangle";
    match live {
        Some((client, unit)) if lazy => {
            let class = format!("{class} is-live");
            view! {
                <aside class={class} data-ad={placement} aria-label="Advertisement" data-ad-lazy="">
                    <div class="ad-slot-frame">
                        <ins
                            class="adsbygoogle"
                            style="display:block"
                            data-ad-client={client}
                            data-ad-slot={unit}
                            data-ad-format="auto"
                            data-full-width-responsive="true"
                        ></ins>
                    </div>
                </aside>
            }
        }
        Some((client, unit)) => {
            let class = format!("{class} is-live");
            view! {
                <aside class={class} data-ad={placement} aria-label="Advertisement">
                    <div class="ad-slot-frame">
                        <ins
                            class="adsbygoogle"
                            style="display:block"
                            data-ad-client={client}
                            data-ad-slot={unit}
                            data-ad-format="auto"
                            data-full-width-responsive="true"
                        ></ins>
                    </div>
                </aside>
            }
        }
        None => view! {
            <aside class={class} data-ad={placement} aria-label="Advertisement">
                <div class="ad-slot-frame">
                    <span class="ad-slot-label">"Ad"</span>
                </div>
            </aside>
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_ca_pub() {
        assert!(sanitize_client("ca-pub-1234567890123456").is_some());
        assert!(sanitize_client("pub-123").is_none());
        assert!(sanitize_client("ca-pub-abc").is_none());
    }

    #[test]
    fn accepts_numeric_slot() {
        assert_eq!(sanitize_slot("1234567890").as_deref(), Some("1234567890"));
        assert!(sanitize_slot("12ab").is_none());
        assert!(sanitize_slot("").is_none());
    }
}
