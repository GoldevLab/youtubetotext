use resuma::prelude::*;

use crate::family::canonical_url;
use crate::site;

pub fn page(_req: FlowRequest) -> View {
    set_page_title("API and limits | YouTubeForge");
    set_page_description(
        "Free YouTube transcript API with a per-IP cap. Request a key for higher limits on /api/transcript, audio, and video.",
    );
    set_page_canonical(canonical_url("/pricing"));
    let contact = site::contact_email();
    let mail = contact
        .as_ref()
        .map(|e| format!("mailto:{e}?subject=YouTubeForge%20API%20key"))
        .unwrap_or_else(|| "https://github.com/GoldevLab/youtubetotext/issues".into());
    let mail_label = if contact.is_some() {
        "Email for an API key"
    } else {
        "Request a key on GitHub"
    };
    view! {
        <main class="content-section privacy-page">
            <p class="eyebrow">"Developers"</p>
            <h1>"API and higher limits"</h1>
            <p class="hero-lead">
                "The website stays free. Scripts that hammer captions through this host need a key so we can raise the cap without draining YouTube from one IP."
            </p>
            <h2>"Free (no key)"</h2>
            <p>"Per IP, per minute: about 24 transcript calls, 10 audio, 6 video. Enough to try. Not enough to crawl a channel."</p>
            <h2>"Key (FORGE_API_KEYS)"</h2>
            <p>"Send X-Api-Key or Authorization: Bearer. Limits rise to about 240 / 80 / 40 per minute. We issue keys by hand — no card form on this page yet."</p>
            <pre class="recap-body">{"GET /api/transcript?v=VIDEO_ID&fmt=srt\nAuthorization: Bearer YOUR_KEY"}</pre>
            <p>
                <a class="btn btn-primary" href={mail}>{mail_label}</a>
                " "
                <NavLink href="/api" class="btn btn-ghost">"API docs"</NavLink>
            </p>
            <p class="hint">
                "A key is not permission to ignore copyright or YouTube’s terms. See "
                <NavLink href="/terms">"Terms"</NavLink>
                "."
            </p>
        </main>
    }
}
