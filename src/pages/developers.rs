use resuma::prelude::*;

use crate::family::canonical_url;

pub fn page(_req: FlowRequest) -> View {
    set_page_title("Developers | YouTubeForge");
    set_page_description("YouTubeForge developer API is temporarily unavailable. Use the free web tool at forgeyt.com.");
    set_page_canonical(canonical_url("/developers"));
    set_page_robots("noindex, follow");
    view! {
        <main class="content-section">
            <p class="eyebrow">"API"</p>
            <h1>"Developer API is paused"</h1>
            <p class="hero-lead">
                "Public API keys and docs are temporarily offline. Paste a YouTube link on the homepage for transcripts, audio, translation, summary, and SRT."
            </p>
            <p class="error-actions">
                <NavLink href="/" class="btn btn-primary">"Open YouTubeForge"</NavLink>
            </p>
        </main>
    }
}
