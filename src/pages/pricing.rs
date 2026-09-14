use resuma::prelude::*;

use crate::family::canonical_url;

pub fn page(_req: FlowRequest) -> View {
    set_page_title("Pricing | YouTubeForge");
    set_page_description("YouTubeForge API pricing is temporarily unavailable. The free web tool stays open.");
    set_page_canonical(canonical_url("/pricing"));
    set_page_robots("noindex, follow");
    view! {
        <main class="content-section pricing-page">
            <p class="eyebrow">"API"</p>
            <h1>"API pricing is paused"</h1>
            <p class="hero-lead">
                "Paid API keys and subscriptions are temporarily unavailable. The free website tool (transcripts, audio, SRT, translate, summary) stays open — no account."
            </p>
            <p class="error-actions">
                <NavLink href="/" class="btn btn-primary">"Back to the tool"</NavLink>
            </p>
        </main>
    }
}
