//! Site-wide JSON-LD (landings were folded into `/`).

use serde_json::{json, Value};

pub fn web_application_json_ld() -> Value {
    json!({
        "@context": "https://schema.org",
        "@type": "WebApplication",
        "name": "YouTubeForge",
        "alternateName": ["YouTube transcript", "YouTube to text"],
        "url": "https://youtubetotext.fly.dev",
        "applicationCategory": "UtilitiesApplication",
        "operatingSystem": "Web",
        "offers": {"@type": "Offer", "price": "0", "priceCurrency": "USD"},
        "description": "Free YouTube transcript tool: search, download SRT/VTT, translate captions, save audio. No account."
    })
}
