use resuma::prelude::*;

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
