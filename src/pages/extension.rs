use resuma::prelude::*;

use crate::family::canonical_url;
use crate::site;

pub fn page(_req: FlowRequest) -> View {
    set_page_title("YouTube Transcript Chrome Extension — Free | YouTubeForge");
    set_page_description(
        "Get a YouTube transcript from any watch page in Chrome. One click opens searchable captions on YouTubeForge — free, no account.",
    );
    set_page_canonical(canonical_url("/extension"));
    let store = site::chrome_store_url();
    let store_block = if let Some(url) = store {
        view! {
            <p class="error-actions">
                <a class="btn btn-primary" href={url} rel="noopener">"Add to Chrome — free"</a>
                <NavLink href="/guides/youtube-transcript" class="btn btn-ghost">"How transcripts work"</NavLink>
            </p>
        }
    } else {
        view! {
            <p class="hint">
                "The Web Store listing is not public yet. Load the unpacked folder from the GitHub repo (extension/) in chrome://extensions with Developer mode on."
            </p>
            <p class="error-actions">
                <a class="btn btn-primary" href="https://github.com/GoldevLab/youtubetotext/tree/main/extension" rel="noopener">
                    "extension/ on GitHub"
                </a>
            </p>
        }
    };
    view! {
        <main class="content-section privacy-page">
            <p class="eyebrow">"Chrome"</p>
            <h1>"YouTube transcript extension for Chrome"</h1>
            <p class="hero-lead">
                "On a YouTube watch page, grab public captions in your browser and open them on YouTubeForge — searchable text, SRT, or audio next. Useful when the website’s server is rate-limited and your laptop is not."
            </p>
            {store_block}
            <h2>"What you get"</h2>
            <ol class="howto-grid">
                <li>
                    <h3>"Watch page button"</h3>
                    <p>"Get transcript fetches captions in the tab, sends them to YouTubeForge, then opens /?v= with the text ready."</p>
                </li>
                <li>
                    <h3>"Toolbar click"</h3>
                    <p>"If the tab is a YouTube video, same flow. Otherwise it opens the home paste box."</p>
                </li>
                <li>
                    <h3>"Same tools after"</h3>
                    <p>"From the result: copy, download SRT/VTT, translate captions, or save MP3 — no second paste."</p>
                </li>
            </ol>
            <h2>"Install tips"</h2>
            <p>
                "Pin the extension after install. Use it on public videos that already have captions (human or auto). Private or age-gated videos usually cannot expose caption tracks."
            </p>
            <p class="error-actions">
                <NavLink href="/" class="btn btn-ghost">"Paste a link on the web"</NavLink>
                <NavLink href="/youtube-to-text" class="btn btn-ghost">"YouTube to text"</NavLink>
                <NavLink href="/guides">"Guides"</NavLink>
            </p>
        </main>
    }
}
