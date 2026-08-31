//! SEO articles. The paste form is only on `/`.

use resuma::prelude::*;
use serde_json::{json, Value};

use crate::family::{landing_for, Mode};

pub fn seo_landing(mode: Mode) -> View {
    let landing = landing_for(mode);
    set_page_title(landing.title);
    set_page_description(landing.description);

    let faq_ld = faq_json_ld(&landing.faq);
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
        <main class="home-page landing-page">
            {View::raw(format!(
                r#"<script type="application/ld+json">{}</script>"#,
                faq_ld
            ))}
            <section class="hero">
                <div class="hero-copy">
                    <p class="eyebrow">{landing.eyebrow}</p>
                    <h1>{landing.h1}</h1>
                    <p class="hero-lead">{landing.lead}</p>
                    <p class="error-actions">
                        <NavLink href="/" class="btn btn-primary">
                            "Paste a YouTube link"
                        </NavLink>
                    </p>
                    <p class="hint">
                        "Paste the link on the home page. After the transcript loads, audio, translation, a recap, and SRT are on that result."
                    </p>
                </div>
            </section>

            {crate::ads::slot("landing-hero", "infeed")}

            <section class="howto" aria-labelledby="howto-title">
                <h2 id="howto-title">{landing.howto_title}</h2>
                <ol class="howto-grid">{howto}</ol>
            </section>

            {crate::ads::slot("landing-mid", "infeed")}

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

            {crate::ads::slot("landing-faq", "infeed")}

            <section class="faq" aria-labelledby="faq-title">
                <h2 id="faq-title">"FAQ"</h2>
                <div class="faq-list">{faq}</div>
            </section>

            {crate::cross_sell::related(mode, "")}
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
