use resuma::prelude::*;

use crate::family::canonical_url;
use crate::site;

pub fn page(_req: FlowRequest) -> View {
    set_page_title("Chrome extension | YouTubeForge");
    set_page_description(
        "Open a YouTube watch page and send the transcript to YouTubeForge without pasting the URL.",
    );
    set_page_canonical(canonical_url("/extension"));
    let store = site::chrome_store_url();
    let store_block = if let Some(url) = store {
        view! {
            <p>
                <a class="btn btn-primary" href={url} rel="noopener">"Add from Chrome Web Store"</a>
            </p>
        }
    } else {
        view! {
            <p class="hint">
                "The Web Store listing is not public yet. Load the unpacked folder from the GitHub repo (extension/) in chrome://extensions with Developer mode on."
            </p>
            <p>
                <a class="btn btn-primary" href="https://github.com/GoldevLab/youtubetotext/tree/main/extension" rel="noopener">
                    "extension/ on GitHub"
                </a>
            </p>
        }
    };
    view! {
        <main class="content-section privacy-page">
            <p class="eyebrow">"Browser"</p>
            <h1>"YouTubeForge extension"</h1>
            <p class="hero-lead">
                "On a YouTube watch page, grab captions in the tab and open /?v= on YouTubeForge. Useful when this server’s IP is rate-limited and your browser is not."
            </p>
            {store_block}
            <h2>"What it does"</h2>
            <ol class="howto-grid">
                <li>
                    <h3>"Watch page button"</h3>
                    <p>"Get transcript fetches captions in your browser and posts them to /api/ingest, then opens the result."</p>
                </li>
                <li>
                    <h3>"Toolbar click"</h3>
                    <p>"If the tab is a YouTube video, same flow. Otherwise it opens the home paste box."</p>
                </li>
            </ol>
            <p>
                <NavLink href="/" class="btn btn-ghost">"Paste a link instead"</NavLink>
            </p>
        </main>
    }
}
