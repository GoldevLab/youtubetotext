use resuma::prelude::*;

use crate::family::Mode;

pub fn related(current: Mode, es: bool) -> View {
    let cards = Mode::all()
        .into_iter()
        .filter(|m| *m != current)
        .map(|m| {
            let href = if es {
                m.es_path().to_string()
            } else {
                m.landing_path().to_string()
            };
            let (label, hint) = if es {
                match m {
                    Mode::Text => (
                        "YouTube a texto",
                        "Transcripción buscable desde subtítulos públicos.",
                    ),
                    Mode::Audio => (
                        "YouTube a MP3",
                        "Guarda la banda sonora y sigue en el mismo video.",
                    ),
                    Mode::Translate => (
                        "Traducir subtítulos",
                        "Conserva tiempos. YouTube tlang, no un chat.",
                    ),
                    Mode::Summary => (
                        "Resumen por capítulos",
                        "Recap extractivo más un prompt para tu modelo.",
                    ),
                    Mode::Srt => (
                        "Descargar SRT / VTT",
                        "Subtítulos temporizados para reproductores.",
                    ),
                }
            } else {
                match m {
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
                }
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

    let title = if es {
        "Otras herramientas YouTubeForge"
    } else {
        "Other YouTubeForge tools"
    };
    let hint = if es {
        "Misma caja de pegado. Trabajo distinto. Texto, audio, traducción, resumen y SRT tienen página propia para que la búsqueda las encuentre."
    } else {
        "Same paste box. Different job. Transcript, audio, translation, summary, and SRT each have their own page so search can find them."
    };

    view! {
        <nav class="cross-sell" aria-label={if es { "Herramientas YouTube relacionadas" } else { "Related YouTube tools" }}>
            <h2>{title}</h2>
            <p class="hint">{hint}</p>
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
            <NavLink href="/youtube-a-texto">"ES texto"</NavLink>
            <span aria-hidden="true">" · "</span>
            <NavLink href="/youtube-a-mp3">"ES MP3"</NavLink>
            <span aria-hidden="true">" · "</span>
            <NavLink href="/privacy">"Privacy"</NavLink>
            <span aria-hidden="true">" · "</span>
            <NavLink href="/terms">"Terms"</NavLink>
            <span aria-hidden="true">" · "</span>
            <NavLink href="/extension">"Extension"</NavLink>
            <span aria-hidden="true">" · "</span>
            <NavLink href="/pricing">"Pricing"</NavLink>
            <span aria-hidden="true">" · "</span>
            <NavLink href="/developers">"API"</NavLink>
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
        "3D-printable QR — stand, tile, keychain, or plaque.",
        "https://placaqr.fly.dev",
    )
];

const SELF: &str = "YouTubeForge";

/// Sister products (same pattern as YouTubeToTranscript’s footer apps).
pub fn sister_apps() -> View {
    let cards = FAMILY
        .iter()
        .copied()
        .filter(|(name, _, _)| *name != SELF)
        .map(|(name, blurb, href)| {
            let name = name.to_string();
            let blurb = blurb.to_string();
            let href = href.to_string();
            view! {
                <li>
                    <a href={href} class="related-card sister-card" rel="noopener">
                        <strong>{name}</strong>
                        <span>{blurb}</span>
                    </a>
                </li>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <nav class="sister-apps" aria-label="Other apps from us">
            <p class="eyebrow">"Also from us"</p>
            <h2>"Free tools, same idea"</h2>
            <p class="hint">
                "No account. Paste, convert, download. Images under a KB cap, PDFs, and 3D QR."
            </p>
            <ul class="related-grid">{cards}</ul>
        </nav>
    }
}

/// Compact footer strip so workspace pages also point at the family.
pub fn sister_apps_links() -> View {
    let items = FAMILY
        .iter()
        .copied()
        .filter(|(name, _, _)| *name != SELF)
        .enumerate()
        .map(|(i, (name, _, href))| {
            let name = name.to_string();
            let href = href.to_string();
            if i == 0 {
                view! { <a href={href} rel="noopener">{name}</a> }
            } else {
                view! {
                    <span aria-hidden="true">" · "</span>
                    <a href={href} rel="noopener">{name}</a>
                }
            }
        })
        .collect::<Vec<_>>();

    view! {
        <nav class="sister-apps-links" aria-label="Also from us">
            <span>"Also from us:"</span>
            " "
            {items}
        </nav>
    }
}
