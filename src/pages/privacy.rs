use resuma::prelude::*;

use crate::family::canonical_url;
use crate::site;

pub fn contact_line() -> View {
    match site::contact_email() {
        Some(email) => {
            let href = format!("mailto:{email}");
            view! {
                <p>
                    "Questions: "
                    <a href={href}>{email}</a>
                    " or "
                    <a href="https://github.com/GoldevLab/youtubetotext/issues" rel="noopener">"GitHub"</a>
                    ". "
                    <NavLink href="/terms">"Terms"</NavLink>
                    " · "
                    <NavLink href="/pricing">"API"</NavLink>
                    "."
                </p>
            }
        }
        None => view! {
            <p>
                "Questions: "
                <a href="https://github.com/GoldevLab/youtubetotext/issues" rel="noopener">"open an issue on GitHub"</a>
                " (set CONTACT_EMAIL on the server to show a mailbox). "
                <NavLink href="/terms">"Terms"</NavLink>
                " · "
                <NavLink href="/pricing">"API"</NavLink>
                "."
            </p>
        },
    }
}

pub fn page(_req: FlowRequest) -> View {
    set_page_title("Privacy | YouTubeForge");
    set_page_description(
        "How YouTubeForge handles links you paste, ads, and the data we do not collect.",
    );
    set_page_canonical(canonical_url("/privacy"));
    view! {
        <main class="content-section privacy-page">
            <p class="eyebrow">"Legal"</p>
            <h1>"Privacy"</h1>
            <p class="hero-lead">
                "YouTubeForge is a no-account tool. We do not create user profiles, and we do not sell a list of the videos you paste."
            </p>

            <h2>"What we process"</h2>
            <p>
                "When you paste a YouTube link we send that video id to YouTube’s public player and caption endpoints so we can show the transcript, audio, or subtitle file. Rate limits use your IP for a short window so scrapers cannot drain captions through this app."
            </p>
            <p>
                "Optional Cloudflare Turnstile may run on the paste form to block bots. Transcripts you edit stay in the browser until you copy or download them."
            </p>

            <h2>"What we do not do"</h2>
            <p>
                "No sign-up, no mailing list, no stored library of other people’s videos. We are not affiliated with YouTube or Google. Use downloads only for videos you are allowed to keep a personal copy of."
            </p>

            <h2>"Advertising"</h2>
            <p>
                "If Google AdSense is enabled, Google and its partners may set cookies or use similar identifiers to show ads and measure them. See "
                <a href="https://policies.google.com/technologies/ads" rel="noopener">"Google’s advertising policies"</a>
                " and "
                <a href="https://adssettings.google.com/" rel="noopener">"Ad Settings"</a>
                ". Publisher ads.txt is served at "
                <a href="/ads.txt">"/ads.txt"</a>
                " when a publisher id is configured."
            </p>

            <h2>"Analytics"</h2>
            <p>
                "If GA4_ID or PLAUSIBLE_DOMAIN is set on the server, page views are measured so we can see which landings work. No account on this site either way."
            </p>
            <h2>"Contact"</h2>
            {contact_line()}
        </main>
    }
}
