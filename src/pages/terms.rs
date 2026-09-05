use resuma::prelude::*;

use crate::family::canonical_url;

pub fn page(_req: FlowRequest) -> View {
    set_page_title("Terms | YouTubeForge");
    set_page_description(
        "What YouTubeForge extracts, what you may download, and what we do not host.",
    );
    set_page_canonical(canonical_url("/terms"));
    view! {
        <main class="content-section privacy-page">
            <p class="eyebrow">"Legal"</p>
            <h1>"Terms of use"</h1>
            <p class="hero-lead">
                "YouTubeForge reads public YouTube caption tracks and, when YouTube exposes a plain stream, lets you save audio or video. We are not YouTube or Google."
            </p>
            <h2>"What you get"</h2>
            <p>
                "Transcripts come from captions YouTube already published (human or auto). We do not invent speech with our own speech-to-text. Audio and video downloads are the streams YouTube already serves to a player, when we can resolve a URL."
            </p>
            <h2>"Your responsibility"</h2>
            <p>
                "Only paste public videos. Only download a personal copy when you are allowed to keep one (your video, a license, or a use the law permits). Do not use this app as a piracy mirror, a bulk scraper, or a way to bypass age, region, or login walls."
            </p>
            <h2>"API"</h2>
            <p>
                "GET /api/transcript is free with a per-IP cap. A key in FORGE_API_KEYS (header X-Api-Key or Authorization: Bearer) raises that cap for scripts. Keys are not a license to ignore YouTube’s rules or copyright."
            </p>
            <h2>"Ads and availability"</h2>
            <p>
                "The tool is free. Ads may appear. We do not promise uptime, completeness, or that every public video will resolve. Results at /?v= are working URLs and stay noindex."
            </p>
            <p>
                <NavLink href="/privacy">"Privacy"</NavLink>
                " · "
                <NavLink href="/pricing">"API pricing"</NavLink>
            </p>
        </main>
    }
}
