use resuma::prelude::*;

use crate::tool::home_search;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <main class="home-page">
            <div class="app-orbs" aria-hidden="true">
                <div class="orb orb-a"></div>
                <div class="orb orb-b"></div>
            </div>
            <section class="hero">
                <div class="hero-copy">
                    <p class="eyebrow">"YouTube transcript, instantly"</p>
                    <h1>"Free YouTube transcript from any video"</h1>
                    <p class="hero-lead">
                        "Paste a YouTube link. Get the transcript: searchable, downloadable as SRT/VTT/Markdown, translatable, shareable. No cookie wall, no account — a cleaner YouTube to text tool."
                    </p>
                    {home_search()}
                </div>
                <aside class="hero-aside">
                    {crate::ads::slot("home-hero", "rectangle")}
                </aside>
            </section>

            <section class="howto" aria-labelledby="howto-title">
                <h2 id="howto-title">"How to get a YouTube transcript"</h2>
                <ol class="howto-grid">
                    <li>
                        <h3>"1. Paste the link"</h3>
                        <p>"Watch, Shorts, youtu.be, or a bare video id. You land on a shareable page at " <code>"/v/{id}"</code> "."</p>
                    </li>
                    <li>
                        <h3>"2. Trim and pick language"</h3>
                        <p>"Skip intros and outros, click two lines to set a range, or switch captions. The language you pick stays in the URL."</p>
                    </li>
                    <li>
                        <h3>"3. Copy, download, or prompt"</h3>
                        <p>"Fix typos in Edit, download SRT/VTT/Markdown, or copy a summary, notes, quiz, or quotes prompt."</p>
                    </li>
                </ol>
            </section>

            {crate::ads::slot("home-mid", "rectangle")}

            <section class="features" aria-labelledby="why-title">
                <h2 id="why-title">"Why YouTubeToText instead of the usual transcript sites"</h2>
                <ul class="feature-grid">
                    <li>
                        <h3>"Shareable pages"</h3>
                        <p>"Every YouTube transcript has a clean URL. Search engines and notes apps can read the text."</p>
                    </li>
                    <li>
                        <h3>"Real downloads"</h3>
                        <p>"TXT, SRT, VTT, Markdown with timestamp links, and JSON. One click — not copy-paste into a doc."</p>
                    </li>
                    <li>
                        <h3>"Find the line"</h3>
                        <p>"Filter as you type. Trim intros and outros. Click a line to jump the video. Press / to search."</p>
                    </li>
                    <li>
                        <h3>"Translate without a round trip"</h3>
                        <p>"Use YouTube’s own caption tracks and auto-translate. The language you pick is in the URL, so you can share it."</p>
                    </li>
                    <li>
                        <h3>"AI prompts, not a paywall"</h3>
                        <p>"Copy a ready-made summary, notes, quiz, or quotes prompt with the transcript. Paste it into the model you already use."</p>
                    </li>
                    <li>
                        <h3>"No extension required"</h3>
                        <p>"Paste a link in the browser. An optional extension exists only if YouTube later blocks our servers."</p>
                    </li>
                    <li>
                        <h3>"A free API"</h3>
                        <p>"curl a JSON/SRT YouTube transcript. No key for light use. Built in Rust — one binary, no Node scrape farm."</p>
                    </li>
                </ul>
            </section>

            {crate::ads::slot("home-faq", "infeed")}

            <section class="faq" aria-labelledby="faq-title">
                <h2 id="faq-title">"FAQ"</h2>
                <div class="faq-list">
                    <details>
                        <summary>"Is YouTubeToText free to use?"</summary>
                        <p>"Yes. No account, no sign-up. If YouTube published captions for a public video, you can read them here."</p>
                    </details>
                    <details>
                        <summary>"How do I access the transcript after generating it?"</summary>
                        <p>"Every transcript lives at a clean URL: " <code>"/v/{videoId}"</code> ". Optional query params: " <code>"lang"</code> " and " <code>"tlang"</code> ". Bookmark it, share it, or paste it into notes — the text is in the page, not trapped in a widget."</p>
                    </details>
                    <details>
                        <summary>"Can I download the transcript?"</summary>
                        <p>"Yes. TXT, SRT, VTT, Markdown with timestamp links, and JSON. Copy and Copy Markdown are one click. Edits you make in Edit mode go into those files."</p>
                    </details>
                    <details>
                        <summary>"Is there a limit to the length of the video?"</summary>
                        <p>"If the video is public and has captions, we load them. We do not add an extra duration cap."</p>
                    </details>
                    <details>
                        <summary>"How long does it take to generate the transcript?"</summary>
                        <p>"Usually about a second. We read YouTube’s existing captions — we do not re-transcribe the audio."</p>
                    </details>
                    <details>
                        <summary>"Is there a limit to the number of transcripts I can generate?"</summary>
                        <p>"No account means no quota UI. Use it reasonably. Heavy bulk jobs should use the API and back off on errors."</p>
                    </details>
                    <details>
                        <summary>"Do you offer a YouTube transcript API?"</summary>
                        <p>"Yes. " <NavLink href="/api">"Free HTTP API"</NavLink> " — " <code>"GET /api/transcript?v=VIDEO_ID&fmt=json"</code> ". Formats: json, txt, srt, vtt, md. No key for light use."</p>
                    </details>
                </div>
            </section>
        </main>
    }
}
