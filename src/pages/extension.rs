use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <main class="content-section">
            <p class="eyebrow">"Browser helper"</p>
            <h1>"Optional backup if YouTube blocks our servers"</h1>
            <p class="hero-lead">
                "You do not need this to use YouTubeForge. Paste a YouTube link on the site — same as "
                <a href="https://youtubetotranscript.com/" rel="noreferrer noopener">"YouTubeToTranscript"</a>
                ", without a cookie wall. This extension only helps if YouTube rate-limits our Fly IP."
            </p>
            {crate::ads::slot("extension-mid", "infeed")}
            <ol class="steps">
                <li>"Chrome → Extensions → Enable Developer mode → Load unpacked → pick the " <code>"extension"</code> " folder in this project."</li>
                <li>"On any YouTube watch page, click " <strong>"Get transcript"</strong> " (or the toolbar icon)."</li>
                <li>"If the YouTubeForge server is blocked, open the failed page anyway — the extension fetches captions and reloads automatically."</li>
            </ol>
            <p class="hint">
                "After deploy, files are also at "
                <a href="/ytt-extension/manifest.json">"/ytt-extension/"</a>
                ". Firefox: load a temporary add-on from the same folder. The extension only talks to YouTube and YouTubeForge."
            </p>
            <p class="error-actions">
                <NavLink href="/" class="btn btn-primary">"Paste a YouTube link"</NavLink>
                <NavLink href="/api" class="btn btn-ghost">"Transcript API"</NavLink>
            </p>
            {crate::ads::slot("extension-bottom", "infeed")}
        </main>
    }
}
