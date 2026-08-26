use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <main class="content-section">
            <p class="eyebrow">"Browser helper"</p>
            <h1>"Optional backup if YouTube blocks our servers"</h1>
            <p class="hero-lead">
                "You do not need this to use YouTubeToText. Paste a YouTube link on the site — same as "
                <a href="https://youtubetotranscript.com/" rel="noreferrer noopener">"YouTubeToTranscript"</a>
                ", but without ads. This extension only helps if YouTube rate-limits our Fly IP."
            </p>
            <ol class="steps">
                <li>"Chrome → Extensions → Enable Developer mode → Load unpacked → pick the " <code>"extension"</code> " folder in this project."</li>
                <li>"On any YouTube watch page, click " <strong>"Get transcript"</strong> " (or the toolbar icon)."</li>
                <li>"If the YouTubeToText server is blocked, open the failed page anyway — the extension fetches captions and reloads automatically."</li>
            </ol>
            <p class="hint">
                "After deploy, files are also at "
                <a href="/ytt-extension/manifest.json">"/ytt-extension/"</a>
                ". Firefox: load a temporary add-on from the same folder. The extension only talks to YouTube and YouTubeToText. No ads."
            </p>
            <p class="error-actions">
                <a class="btn btn-primary" href="/">"Paste a YouTube link"</a>
                <a class="btn btn-ghost" href="/api">"Transcript API"</a>
            </p>
        </main>
    }
}
