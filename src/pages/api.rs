use resuma::prelude::*;

use crate::family::canonical_url;

pub fn page(_req: FlowRequest) -> View {
    set_page_title("Transcript API | YouTubeForge");
    set_page_description(
        "GET /api/transcript?v= for JSON, TXT, SRT, VTT, or Markdown. Optional API key for higher limits.",
    );
    set_page_canonical(canonical_url("/api"));
    view! {
        <main class="content-section privacy-page">
            <p class="eyebrow">"HTTP"</p>
            <h1>"Transcript API"</h1>
            <p class="hero-lead">
                "Public captions only. Rate-limited per IP. Add a key for batch jobs — see "
                <NavLink href="/pricing">"pricing"</NavLink>
                "."
            </p>
            <h2>"Transcript"</h2>
            <pre class="recap-body">{"GET /api/transcript?v=VIDEO_ID&fmt=json|txt|srt|vtt|md&lang=&tlang="}</pre>
            <h2>"Audio / video"</h2>
            <pre class="recap-body">{"GET /api/audio?v=VIDEO_ID&fmt=mp3\nGET /api/video?v=VIDEO_ID&q=720"}</pre>
            <h2>"Auth"</h2>
            <p>"Optional header X-Api-Key or Authorization: Bearer. Keys are set on the server as FORGE_API_KEYS."</p>
            <p>
                <NavLink href="/pricing" class="btn btn-primary">"Get a key"</NavLink>
                " "
                <NavLink href="/terms" class="btn btn-ghost">"Terms"</NavLink>
            </p>
        </main>
    }
}
