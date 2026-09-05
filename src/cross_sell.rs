use resuma::prelude::*;

use crate::family::Mode;

pub fn related(current: Mode) -> View {
    let cards = Mode::all()
        .into_iter()
        .filter(|m| *m != current)
        .map(|m| {
            let href = m.landing_path().to_string();
            let label = match m {
                Mode::Text => "YouTube to text",
                Mode::Audio => "YouTube to MP3",
                Mode::Translate => "Translate captions",
                Mode::Summary => "Chapter summary",
                Mode::Srt => "Download SRT / VTT",
            };
            let hint = match m {
                Mode::Text => "Searchable transcript from public captions.",
                Mode::Audio => "Save the soundtrack, then stay on the same video.",
                Mode::Translate => "Keep timestamps. YouTube tlang, not a chat paste.",
                Mode::Summary => "Extractive recap plus a prompt for your own model.",
                Mode::Srt => "Timed subtitle files for players and editors.",
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
            <NavLink href="/youtube-a-texto">"ES"</NavLink>
            <span aria-hidden="true">" · "</span>
            <NavLink href="/privacy">"Privacy"</NavLink>
            <span aria-hidden="true">" · "</span>
            <NavLink href="/terms">"Terms"</NavLink>
            <span aria-hidden="true">" · "</span>
            <NavLink href="/pricing">"API"</NavLink>
            <span aria-hidden="true">" · "</span>
            <NavLink href="/extension">"Extension"</NavLink>
        </nav>
    }
}

/// Sister products (same pattern as YouTubeToTranscript’s footer apps).
pub fn sister_apps() -> View {
    let cards = [
        (
            "PlacaQR",
            "3D-printable QR — stand, tile, keychain, or plaque.",
            "https://placaqr.fly.dev",
        ),
        (
            "UnderKb",
            "Compress images to a real KB target. JPG, WebP, PNG.",
            "https://underkb.fly.dev",
        ),
        (
            "PDFForge",
            "Merge, split, compress PDFs. JPG ↔ PDF and extract text.",
            "https://pdfforge.fly.dev",
        ),
        (
            "Billloom",
            "Invoice, quote, and receipt PDFs. No account, no watermark.",
            "https://billloom.fly.dev",
        ),
    ]
    .into_iter()
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
                "No account. Paste, convert, download. QR in 3D, images under a KB cap, PDFs, and invoices."
            </p>
            <ul class="related-grid">{cards}</ul>
        </nav>
    }
}

/// Compact footer strip so workspace pages also point at the family.
pub fn sister_apps_links() -> View {
    view! {
        <p class="sister-apps-links">
            <span>"Also from us:"</span>
            " "
            <a href="https://placaqr.fly.dev" rel="noopener">"PlacaQR"</a>
            <span aria-hidden="true">" · "</span>
            <a href="https://underkb.fly.dev" rel="noopener">"UnderKb"</a>
            <span aria-hidden="true">" · "</span>
            <a href="https://pdfforge.fly.dev" rel="noopener">"PDFForge"</a>
            <span aria-hidden="true">" · "</span>
            <a href="https://billloom.fly.dev" rel="noopener">"Billloom"</a>
        </p>
    }
}
