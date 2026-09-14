use resuma::prelude::*;

use crate::family::canonical_url;

pub fn page(_req: FlowRequest) -> View {
    set_page_title("API key | YouTubeForge");
    set_page_description("YouTubeForge API subscriptions are temporarily unavailable.");
    set_page_canonical(canonical_url("/developers/welcome"));
    set_page_robots("noindex, follow");
    view! {
        <main class="content-section">
            <p class="eyebrow">"API"</p>
            <h1>"API keys are paused"</h1>
            <p class="hero-lead">
                "Checkout and key reveal are temporarily offline. The free web tool remains available."
            </p>
            <p class="error-actions">
                <NavLink href="/" class="btn btn-primary">"Back to the tool"</NavLink>
            </p>
        </main>
    }
}
