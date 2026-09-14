use resuma::prelude::*;

use crate::family::Mode;

pub fn related(current: Mode) -> View {
    let cards = Mode::all()
        .into_iter()
        .filter(|m| *m != current)
        .map(|m| {
            let href = m.landing_path().to_string();
            let (label, hint) = match m {
                Mode::Text => (
                    "YouTube to text",
                    "Searchable transcript from public captions.",
                ),
                Mode::Audio => (
                    "YouTube to MP3",
                    "Save the soundtrack, then stay on the same video.",
                ),
                Mode::Translate => (
                    "Translate captions",
                    "Keep timestamps. YouTube tlang, not a chat paste.",
                ),
                Mode::Summary => (
                    "Chapter summary",
                    "Extractive recap plus a prompt for your own model.",
                ),
                Mode::Srt => (
                    "Download SRT / VTT",
                    "Timed subtitle files for players and editors.",
                ),
            };
            view! {
                <li>
                    <a href={href} class="related-card" data-r-nav="true">
                        <strong>{label}</strong>
                        <span>{hint}</span>
                    </a>
                </li>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <nav class="cross-sell" aria-label="Related YouTube tools">
            <h2>"Other YouTubeForge tools"</h2>
            <p class="hint">
                "Same paste box. Different job. Transcript, audio, translation, summary, and SRT each have their own page so search can find them."
            </p>
            <ul class="related-grid">{cards}</ul>
        </nav>
    }
}

pub fn seo_footer_links() -> View {
    view! {
        <nav class="seo-links" aria-label="YouTubeForge tools">
            <NavLink href="/youtube-to-text">"Transcript"</NavLink>
            <span aria-hidden="true">" · "</span>
            <NavLink href="/youtube-to-audio">"Audio / MP3"</NavLink>
            <span aria-hidden="true">" · "</span>
            <NavLink href="/youtube-translator">"Translate"</NavLink>
            <span aria-hidden="true">" · "</span>
            <NavLink href="/youtube-summary">"Summary"</NavLink>
            <span aria-hidden="true">" · "</span>
            <NavLink href="/youtube-to-srt">"SRT"</NavLink>
            <span aria-hidden="true">" · "</span>
            <NavLink href="/guides">"Guides"</NavLink>
            <span aria-hidden="true">" · "</span>
            <NavLink href="/privacy">"Privacy"</NavLink>
            <span aria-hidden="true">" · "</span>
            <NavLink href="/terms">"Terms"</NavLink>
            <span aria-hidden="true">" · "</span>
            <NavLink href="/extension">"Extension"</NavLink>
        </nav>
    }
}

pub fn guides_nav(current_path: Option<&str>) -> View {
    let cards = crate::guides::ALL
        .iter()
        .filter(|g| current_path != Some(g.path))
        .map(|g| {
            let href = g.path.to_string();
            let blurb = g.lead;
            let label = g.h1;
            view! {
                <li>
                    <a href={href} class="related-card" data-r-nav="true">
                        <strong>{label}</strong>
                        <span>{blurb}</span>
                    </a>
                </li>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <nav class="cross-sell guides-nav" aria-label="Guides">
            <h2>"Guides"</h2>
            <p class="hint">
                "Short explainers for transcript, subtitles, and audio — then the same paste box."
            </p>
            <ul class="related-grid">{cards}</ul>
            <p>
                <NavLink href="/guides">"All guides"</NavLink>
            </p>
        </nav>
    }
}

/// Canonical family order. Each app skips itself in the footer and home cards.
const FAMILY: &[(&str, &str, &str)] = &[
    (
        "YouTubeForge",
        "YouTube transcript, MP3, SRT, and translation.",
        "https://forgeyt.com",
    ),
    (
        "UnderKb",
        "Compress images to a real KB target. JPG, WebP, PNG.",
        "https://under200kb.com",
    ),
    (
        "PDFForge",
        "Merge, split, compress PDFs. JPG ↔ PDF and extract text.",
        "https://pdfforge.fly.dev",
    ),
    (
        "PlacaQR",
        "QR codes for Venezuelan plates and stickers.",
        "https://placaqr.com",
    ),
];

pub fn sister_apps() -> View {
    let cards = FAMILY
        .iter()
        .filter(|(_, _, url)| !url.contains("forgeyt.com"))
        .map(|(name, blurb, url)| {
            let href = (*url).to_string();
            view! {
                <li>
                    <a class="family-card" href={href} rel="noopener">
                        <h3>{*name}</h3>
                        <p>{*blurb}</p>
                    </a>
                </li>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <section class="family" aria-labelledby="family-title">
            <h2 id="family-title">"Also from us"</h2>
            <ul class="family-grid">{cards}</ul>
        </section>
    }
}

pub fn sister_apps_links() -> View {
    let links = FAMILY
        .iter()
        .filter(|(_, _, url)| !url.contains("forgeyt.com"))
        .enumerate()
        .map(|(i, (name, _, url))| {
            let href = (*url).to_string();
            let sep = if i == 0 {
                view! { <span></span> }
            } else {
                view! { <span aria-hidden="true">" · "</span> }
            };
            view! {
                <>
                    {sep}
                    <a href={href} rel="noopener">{*name}</a>
                </>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <p class="family-links">
            "Also from us: "
            {links}
        </p>
    }
}
