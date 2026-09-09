use resuma::prelude::*;

use crate::family::{app_href, canonical_url, Mode};
use crate::parse::parse_video_id;
use crate::tool::home_search;
use crate::workspace::workspace;

pub fn page(req: FlowRequest) -> View {
    let mode = Mode::parse(req.query_param("mode").unwrap_or("text"));
    let raw = req.query_param("v").unwrap_or("");
    let video_id = parse_video_id(raw).unwrap_or_default();
    if video_id.is_empty() {
        return idle();
    }
    set_page_title(format!(
        "{} | YouTubeForge",
        mode.nav_label()
    ));
    set_page_description(
        "Transcript loaded. Download audio, translate captions, summarize, or export SRT — same video.",
    );
    // `/?v=…&mode=…` is an unbounded, per-video URL space: keep it out of the
    // index (the canonical already points at `/`).
    set_page_robots("noindex, follow");
    transcript_workspace(req, video_id, mode)
}

fn idle() -> View {
    set_page_title(crate::family::HOME_TITLE);
    set_page_description(crate::family::HOME_DESCRIPTION);
    set_page_canonical(canonical_url("/"));
    view! {
        <main class="home-page">
            <div class="hero-wrap">
                <div class="hero-particles" data-hero-particles="" aria-hidden="true"></div>
                <section class="hero">
                    <div class="hero-copy">
                        <p class="eyebrow">"YouTubeForge"</p>
                        <h1>"All the YouTube tools you need"</h1>
                        <p class="hero-lead">
                            "Paste a public link once. Pull the transcript, download SRT or audio, translate captions, or grab a short recap — same video, no account."
                        </p>
                        {home_search(Mode::Text)}
                    </div>
                </section>
            </div>
            <section class="howto" aria-labelledby="howto-title">
                <h2 id="howto-title">"How it works"</h2>
                <ol class="howto-grid">
                    <li>
                        <h3>"Paste the link"</h3>
                        <p>"A watch URL, Shorts, youtu.be, or the video id. No account. We read YouTube’s public caption tracks — we do not invent speech from the audio."</p>
                    </li>
                    <li>
                        <h3>"Read and export"</h3>
                        <p>"Search lines, skip intros, copy the text, or download TXT, SRT, VTT, or Markdown. MP3 is the default audio download."</p>
                    </li>
                    <li>
                        <h3>"Stay on this video"</h3>
                        <p>"The result URL uses ?v= and mode=text (not indexed). Audio, translation, a recap, and SRT stay on that same page."</p>
                    </li>
                </ol>
            </section>
            <section class="features" aria-labelledby="jobs-title">
                <h2 id="jobs-title">"Choose what you need from the video"</h2>
                <p class="hint">
                    "Transcript, audio, translation, summary, or subtitles — same paste box, different outcome."
                </p>
                <ul class="feature-grid">
                    <li>
                        <h3><NavLink href="/youtube-to-text">"YouTube to text"</NavLink></h3>
                        <p>"Searchable transcript from public captions. Quotes, notes, timestamps."</p>
                    </li>
                    <li>
                        <h3><NavLink href="/youtube-to-audio">"YouTube to MP3"</NavLink></h3>
                        <p>"Soundtrack only. MP3 by default, or M4A, Opus, WAV."</p>
                    </li>
                    <li>
                        <h3><NavLink href="/youtube-translator">"Translate captions"</NavLink></h3>
                        <p>"YouTube tlang keeps cue times. Not a paragraph dump into a chat."</p>
                    </li>
                    <li>
                        <h3><NavLink href="/youtube-summary">"Chapter summary"</NavLink></h3>
                        <p>"Extractive recap from the captions, plus a prompt for your own model."</p>
                    </li>
                    <li>
                        <h3><NavLink href="/youtube-to-srt">"SRT / VTT"</NavLink></h3>
                        <p>"Timed subtitle files for VLC, editors, and HTML5 tracks."</p>
                    </li>
                </ul>
            </section>
            <section class="faq" aria-labelledby="faq-title">
                <h2 id="faq-title">"FAQ"</h2>
                <div class="faq-list">
                    <details>
                        <summary>"Is YouTubeForge free to use?"</summary>
                        <p>"Yes. No account, no sign-up. Public YouTube captions are extracted as-is. Ads may appear around the tool."</p>
                    </details>
                    <details>
                        <summary>"How do I access the transcript after generating it?"</summary>
                        <p>"Every result lives on the home URL with v= and mode=text. Optional query params: lang, tlang, and mode (audio, translate, summary, srt). Those URLs are noindex."</p>
                    </details>
                    <details>
                        <summary>"Can I download the transcript?"</summary>
                        <p>"Yes. Download TXT, SRT, VTT, Markdown with timestamp links, or JSON. Copy and Copy Markdown are also available."</p>
                    </details>
                    <details>
                        <summary>"Is there a limit to the length of the video?"</summary>
                        <p>"If YouTube has captions for a public video, YouTubeForge can load them. There is no extra length cap on our side."</p>
                    </details>
                </div>
            </section>
            {crate::ads::slot("home-faq", "infeed")}
            {crate::cross_sell::sister_apps()}
        </main>
    }
}

fn transcript_workspace(req: FlowRequest, video_id: String, mode: Mode) -> View {
    let lang = req.query_param("lang").unwrap_or("").to_string();
    let tlang = req.query_param("tlang").unwrap_or("").to_string();
    let retry = app_href(&video_id, mode);
    let mode_s = mode.slug().to_string();
    load_boundary(
        crate::actions::use_video_doc_load(),
        {
            let retry = retry.clone();
            let vid = video_id.clone();
            let lang_ok = lang.clone();
            let tlang_ok = tlang.clone();
            let mode_ok = mode_s.clone();
            move |res| match res {
                Ok(doc) => view! {
                    <main class="workspace-main">
                        {workspace(doc, lang_ok, tlang_ok, mode_ok)}
                    </main>
                },
                Err(e) => fail_view(e.message, retry, vid),
            }
        },
        {
            let retry = retry.clone();
            let vid = video_id.clone();
            move |err| fail_view(err.message, retry, vid)
        },
        || pending(),
    )
}

fn pending() -> View {
    view! {
        <main class="content-section page-pending">
            <p class="eyebrow">"Working"</p>
            <h1>"Talking to YouTube…"</h1>
            <p class="hero-lead">"This usually takes a second."</p>
        </main>
    }
}

fn fail_view(message: String, retry: String, vid: String) -> View {
    view! {
        <main class="content-section">
            <h1>"Could not load that transcript"</h1>
            <p class="hero-lead">{message}</p>
            <p class="error-actions">
                <NavLink href={retry} class="btn btn-primary">"Try again"</NavLink>
                <NavLink href="/" class="btn btn-ghost">"Another link"</NavLink>
                <button type="button" class="btn btn-ghost" data-fail-audio="" data-vid={vid}>"Download audio anyway"</button>
            </p>
        </main>
    }
}
