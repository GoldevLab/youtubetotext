//! SEO landings: unique article + the same paste form. Results still open `/?v=&mode=`.

use resuma::prelude::*;
use serde_json::{json, Value};

use crate::family::{canonical_url, landing_for, Mode};
use crate::tool::home_search;

pub fn web_application_json_ld() -> Value {
    json!({
        "@context": "https://schema.org",
        "@type": "WebApplication",
        "name": "YouTubeForge",
        "alternateName": ["YouTube transcript", "YouTube to text"],
        "url": crate::family::public_origin(),
        "applicationCategory": "UtilitiesApplication",
        "operatingSystem": "Web",
        "offers": {"@type": "Offer", "price": "0", "priceCurrency": "USD"},
        "description": "Free YouTube transcript tool: search, download SRT/VTT, translate captions, save audio. No account."
    })
}

pub fn seo_landing(mode: Mode) -> View {
    render_landing(mode, landing_for(mode), false)
}

pub fn seo_landing_es(mode: Mode) -> View {
    render_landing(mode, crate::landing_es::landing_for_es(mode), true)
}

fn render_landing(mode: Mode, landing: crate::family::Landing, es: bool) -> View {
    set_page_title(landing.title);
    set_page_description(landing.description);
    set_page_canonical(canonical_url(if es {
        mode.es_path()
    } else {
        mode.landing_path()
    }));
    set_page_json_ld(faq_json_ld(&landing.faq));
    let alt_href = if es {
        mode.landing_path().to_string()
    } else {
        mode.es_path().to_string()
    };
    let alt_label = if es {
        "This page in English"
    } else {
        "Esta página en español"
    };
    let limits_title = if es { "Límites" } else { "Limits" };
    let faq_title = if es { "Preguntas frecuentes" } else { "FAQ" };

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
        <main class="home-page landing-page" lang={if es { "es" } else { "en" }}>
            <div class="hero-wrap">
                <div class="hero-particles" data-hero-particles="" aria-hidden="true"></div>
                <section class="hero">
                    <div class="hero-copy">
                        <p class="eyebrow">{landing.eyebrow}</p>
                        <h1>{landing.h1}</h1>
                        <p class="hero-lead">{landing.lead}</p>
                        <p class="hint">
                            <NavLink href={alt_href}>{alt_label}</NavLink>
                        </p>
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
                <h2>{limits_title}</h2>
                <p>{landing.limits}</p>
            </section>

            {crate::ads::slot("landing-mid", "infeed")}

            <section class="faq" aria-labelledby="faq-title">
                <h2 id="faq-title">{faq_title}</h2>
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
