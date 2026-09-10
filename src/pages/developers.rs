use resuma::prelude::*;

use crate::billing::billing_ready;
use crate::family::canonical_url;

pub fn page(_req: FlowRequest) -> View {
    set_page_title("Developers | YouTubeForge API");
    set_page_description(
        "YouTubeForge developer API: transcripts, SRT, translation, and optional audio with an x-api-key from Lemon Squeezy.",
    );
    set_page_canonical(canonical_url("/developers"));

    let sub_link = if billing_ready() {
        view! { <NavLink href="/pricing" class="btn btn-primary">"Get an API key"</NavLink> }
    } else {
        view! { <NavLink href="/pricing" class="btn btn-primary">"See pricing"</NavLink> }
    };

    view! {
        <main class="content-section docs-page">
            <p class="eyebrow">"API"</p>
            <h1>"YouTubeForge for developers"</h1>
            <p class="hero-lead">
                "Same caption engine as the website. Authenticate with "
                <code>"x-api-key"</code>
                " (or "
                <code>"Authorization: Bearer …"</code>
                "). Billing via Lemon Squeezy."
            </p>
            <p class="error-actions">{sub_link} <NavLink href="/pricing" class="btn btn-ghost">"Plans"</NavLink></p>

            <h2>"Auth"</h2>
            <pre class="code-block">{"curl -sS 'https://forgeyt.com/api/transcript?v=VIDEO_ID&fmt=json' \\\n  -H 'x-api-key: forge_sk_live_…' \\\n  -H 'Accept: application/json'"}</pre>

            <h2>"Endpoints"</h2>
            <ul class="docs-list">
                <li>
                    <strong>"GET /api/transcript"</strong>
                    " — "
                    <code>"v"</code>
                    " or "
                    <code>"url"</code>
                    ", optional "
                    <code>"lang"</code>
                    ", "
                    <code>"tlang"</code>
                    ", "
                    <code>"fmt=json|txt|srt|vtt|md|timed"</code>
                    ". Counts as one transcript credit."
                </li>
                <li>
                    <strong>"POST /api/translate"</strong>
                    " — JSON body with cues + "
                    <code>"tlang"</code>
                    ". Counts as one translate credit."
                </li>
                <li>
                    <strong>"GET /api/audio"</strong>
                    " — "
                    <code>"v"</code>
                    " + "
                    <code>"fmt=mp3|m4a|opus|wav"</code>
                    ". Basic/Pro monthly audio caps."
                </li>
                <li>
                    <strong>"GET /api/video"</strong>
                    " — Pro only (SD / 720). Pass "
                    <code>"q=480|720"</code>
                    ". HD not sold on the API."
                </li>
                <li>
                    <strong>"GET /api/v1/me"</strong>
                    " — plan + usage counters for the calling key."
                </li>
            </ul>

            <h2>"Limits"</h2>
            <p>
                "Monthly quotas reset with your Lemon billing period. Burst rate is per key (Basic 120/min, Pro 300/min). Free website traffic still uses network limits and proof-of-work for downloads — paid keys skip the bot gate."
            </p>

            <h2>"Errors"</h2>
            <p>
                "JSON "
                <code>"{\"error\":\"…\"}"</code>
                " with "
                <code>"429"</code>
                " when a monthly meter is exhausted, "
                <code>"401"</code>
                " on "
                <code>"/api/v1/me"</code>
                " without a key, and "
                <code>"404"</code>
                " for anonymous cross-site scrapers (same-site UI still works)."
            </p>
        </main>
    }
}
