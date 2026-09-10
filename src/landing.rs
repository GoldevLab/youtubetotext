//! SEO landings: unique article + the same paste form. Results still open `/?v=&mode=`.

use resuma::prelude::*;
use serde_json::{json, Value};

use crate::family::{canonical_url, landing_for, Mode};
use crate::tool::home_search;

pub fn web_application_json_ld() -> Value {
    let origin = crate::family::public_origin();
    json!({
        "@type": "WebApplication",
        "@id": format!("{origin}/#app"),
        "name": "YouTubeForge",
        "alternateName": ["YouTube transcript", "YouTube to text", "YouTube to SRT"],
        "url": origin,
        "applicationCategory": "UtilitiesApplication",
        "operatingSystem": "Any",
        "browserRequirements": "Requires JavaScript",
        "offers": {"@type": "Offer", "price": "0", "priceCurrency": "USD"},
        "featureList": [
            "Searchable YouTube transcripts",
            "Download SRT and VTT captions",
            "Translate captions",
            "Download audio (MP3/M4A)",
            "Chapter-style summary from captions"
        ],
        "description": "Free YouTube transcript tool: search, download SRT/VTT, translate captions, save audio. No account."
    })
}

/// Single Schema.org document (not a JSON array). Monitor and similar crawlers
/// often miss arrays / nested graphs inside `[...]`.
pub fn home_structured_data() -> Value {
    let origin = crate::family::public_origin();
    json!({
        "@context": "https://schema.org",
        "@graph": [
            {
                "@type": "Organization",
                "@id": format!("{origin}/#org"),
                "name": "YouTubeForge",
                "url": origin
            },
            {
                "@type": "WebSite",
                "@id": format!("{origin}/#website"),
                "name": "YouTubeForge",
                "url": origin,
                "publisher": {"@id": format!("{origin}/#org")},
                "potentialAction": {
                    "@type": "SearchAction",
                    "target": format!("{origin}/?v={{search_term_string}}"),
                    "query-input": "required name=search_term_string"
                }
            },
            web_application_json_ld(),
            {
                "@type": "HowTo",
                "name": "Get a YouTube transcript with YouTubeForge",
                "description": "Paste a public YouTube URL and read, search, or download the captions.",
                "totalTime": "PT1M",
                "step": [
                    {
                        "@type": "HowToStep",
                        "position": 1,
                        "name": "Paste the link",
                        "text": "Paste a YouTube watch URL, Shorts link, youtu.be link, or video id into the box on forgeyt.com."
                    },
                    {
                        "@type": "HowToStep",
                        "position": 2,
                        "name": "Open the transcript",
                        "text": "Submit the form to load public captions as searchable text with timestamps."
                    },
                    {
                        "@type": "HowToStep",
                        "position": 3,
                        "name": "Copy or download",
                        "text": "Copy the text, or download TXT, SRT, VTT, Markdown, or JSON. Optional: translate, summarize, or save audio."
                    }
                ]
            },
            {
                "@type": "FAQPage",
                "@id": format!("{origin}/#faq"),
                "mainEntity": [
                    {
                        "@type": "Question",
                        "name": "Is YouTubeForge free to use?",
                        "acceptedAnswer": {
                            "@type": "Answer",
                            "text": "Yes. No account, no sign-up. Public YouTube captions are extracted as-is. Ads may appear around the tool."
                        }
                    },
                    {
                        "@type": "Question",
                        "name": "How do I access the transcript after generating it?",
                        "acceptedAnswer": {
                            "@type": "Answer",
                            "text": "Every result lives on the home URL with v= and mode=text. Optional query params: lang, tlang, and mode (audio, translate, summary, srt)."
                        }
                    },
                    {
                        "@type": "Question",
                        "name": "Can I download the transcript?",
                        "acceptedAnswer": {
                            "@type": "Answer",
                            "text": "Yes. Download TXT, SRT, VTT, Markdown with timestamp links, or JSON. Copy and Copy Markdown are also available."
                        }
                    },
                    {
                        "@type": "Question",
                        "name": "Is there a limit to the length of the video?",
                        "acceptedAnswer": {
                            "@type": "Answer",
                            "text": "Captions have no extra length cap. Video: about 3 hours at 360p/480p, 90 min at 720p, 60 min at 1080p+. Audio: about 3 hours."
                        }
                    }
                ]
            }
        ]
    })
}

pub fn home_structured_data_str() -> String {
    serde_json::to_string(&home_structured_data()).unwrap_or_else(|_| "{}".into())
}

/// Plain JSON-LD script (no CSP nonce). Monitor and similar crawlers often miss
/// `type="application/ld+json" nonce="…"` because their regex expects `">` right after the type.
pub fn home_json_ld_script() -> String {
    let safe = home_structured_data_str()
        .replace('<', "\\u003c")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029");
    format!(r#"<script type="application/ld+json">{safe}</script>"#)
}

pub fn seo_landing(mode: Mode) -> View {
    render_landing(mode, landing_for(mode))
}

fn render_landing(mode: Mode, landing: crate::family::Landing) -> View {
    set_page_title(landing.title);
    set_page_description(landing.description);
    set_page_canonical(canonical_url(mode.landing_path()));
    set_page_json_ld(faq_json_ld(&landing.faq));

    let howto: Vec<View> = landing
        .howto
        .iter()
        .map(|(h, p)| {
            view! {
                <li>
                    <h3>{*h}</h3>
                    <p>{*p}</p>
                </li>
            }
        })
        .collect();
    let why: Vec<View> = landing
        .why
        .iter()
        .map(|(h, p)| {
            view! {
                <li>
                    <h3>{*h}</h3>
                    <p>{*p}</p>
                </li>
            }
        })
        .collect();
    let examples: Vec<View> = landing
        .examples
        .iter()
        .map(|(h, p)| {
            view! {
                <li>
                    <h3>{*h}</h3>
                    <p>{*p}</p>
                </li>
            }
        })
        .collect();
    let faq: Vec<View> = landing
        .faq
        .iter()
        .map(|(q, a)| {
            view! {
                <details>
                    <summary>{*q}</summary>
                    <p>{*a}</p>
                </details>
            }
        })
        .collect();

    view! {
        <main class="home-page landing-page" lang="en">
            <div class="hero-wrap">
                <div class="hero-particles" data-hero-particles="" aria-hidden="true"></div>
                <section class="hero">
                    <div class="hero-copy">
                        <p class="eyebrow">{landing.eyebrow}</p>
                        <h1>{landing.h1}</h1>
                        <p class="hero-lead">{landing.lead}</p>
                        {home_search(mode)}
                    </div>
                </section>
            </div>

            <section class="howto" aria-labelledby="howto-title">
                <h2 id="howto-title">{landing.howto_title}</h2>
                <ol class="howto-grid">{howto}</ol>
            </section>

            <section class="features" aria-labelledby="why-title">
                <h2 id="why-title">{landing.why_title}</h2>
                <ul class="feature-grid">{why}</ul>
            </section>

            <section class="features" aria-labelledby="ex-title">
                <h2 id="ex-title">{landing.examples_title}</h2>
                <ul class="feature-grid">{examples}</ul>
            </section>

            <section class="content-section limits">
                <h2>"Limits"</h2>
                <p>{landing.limits}</p>
            </section>

            {crate::ads::slot("landing-mid", "infeed")}

            <section class="faq" aria-labelledby="faq-title">
                <h2 id="faq-title">"FAQ"</h2>
                <div class="faq-list">{faq}</div>
            </section>

            {crate::cross_sell::related(mode)}
        </main>
    }
}

fn faq_json_ld(faq: &[(&str, &str); 5]) -> String {
    let entities: Vec<Value> = faq
        .iter()
        .map(|(q, a)| {
            json!({
                "@type": "Question",
                "name": q,
                "acceptedAnswer": { "@type": "Answer", "text": a }
            })
        })
        .collect();
    serde_json::to_string(&json!({
        "@context": "https://schema.org",
        "@type": "FAQPage",
        "mainEntity": entities
    }))
    .unwrap_or_else(|_| "{}".into())
}
