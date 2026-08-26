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
                <p class="eyebrow">"YouTube transcript, instantly"</p>
                <h1>"Free YouTube transcript from any video"</h1>
                <p class="hero-lead">
                    "Paste a YouTube link. Get the transcript: searchable, downloadable as SRT/VTT/Markdown, translatable, shareable. No ads, no cookie wall, no account — a cleaner YouTube to text tool."
                </p>
                {home_search()}
            </section>

            <section class="features" aria-labelledby="why-title">
                <h2 id="why-title">"Why YouTubeToText instead of the usual transcript sites"</h2>
                <ul class="feature-grid">
                    <li>
                        <h3>"Shareable pages"</h3>
                        <p>"Every YouTube transcript has a clean URL. Search engines and notes apps can read the text, not a blob of ads."</p>
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
                        <p>"Paste a link in the browser, like the other transcript sites — but without ads or a cookie wall. An optional extension exists only if YouTube later blocks our servers."</p>
                    </li>
                    <li>
                        <h3>"A free API"</h3>
                        <p>"curl a JSON/SRT YouTube transcript. No key for light use. Built in Rust — one binary, no Node scrape farm."</p>
                    </li>
                </ul>
            </section>
        </main>
    }
}
