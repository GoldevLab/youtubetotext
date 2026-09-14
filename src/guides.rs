//! Long-form SEO guides — crawlable articles that funnel to the paste tools.

use resuma::prelude::*;
use serde_json::json;

use crate::family::canonical_url;
use crate::tool::home_search;
use crate::family::Mode;

pub struct Guide {
    pub path: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub h1: &'static str,
    pub lead: &'static str,
    pub mode: Mode,
    pub sections: &'static [(&'static str, &'static str)],
    pub faq: &'static [(&'static str, &'static str)],
}

pub const ALL: &[Guide] = &[TRANSCRIPT, SRT_VTT, MP3];

pub const TRANSCRIPT: Guide = Guide {
    path: "/guides/youtube-transcript",
    title: "How to Get a YouTube Transcript (Free, No Account) | YouTubeForge",
    description: "Paste a public YouTube link and read searchable captions. Copy quotes, download TXT/Markdown, jump timestamps. Uses YouTube’s public subtitle tracks.",
    h1: "How to get a YouTube transcript for free",
    lead: "You do not need to re-type an interview. If YouTube published captions (human or auto), YouTubeForge turns them into searchable text you can copy, download, or skim with timestamps.",
    mode: Mode::Text,
    sections: &[
        (
            "What “transcript” means here",
            "A transcript on YouTubeForge is the caption track YouTube already exposes — timed lines you can search, trim, and export. We do not run speech-to-text on our servers. If the video has no captions, there is nothing to extract.",
        ),
        (
            "Step 1 — Copy the video link",
            "Use a watch URL, Shorts link, youtu.be short link, or the 11-character video id. Private, age-restricted, or live-only videos usually fail because captions are not publicly available the same way.",
        ),
        (
            "Step 2 — Paste on YouTubeForge",
            "Open the box below (or /youtube-to-text), paste, and submit. The result opens at /?v= with the text beside the player. That result URL stays noindex so it does not flood search.",
        ),
        (
            "Step 3 — Find the line you need",
            "Search for a name or phrase, skip intro/outro with the range tools, click a line to jump the video, then copy plain text or Markdown with timestamps.",
        ),
        (
            "When this beats watching",
            "Hour-long interviews, lectures, and podcasts uploads are faster as text when you only need a quote, a name, or a claim. Export SRT if you need timed subtitles in an editor.",
        ),
        (
            "Limits to expect",
            "No captions published → nothing to show. Auto-captions can mis-hear names; edit on-device before you trust a citation. We are not YouTube or Google.",
        ),
    ],
    faq: &[
        (
            "Is a YouTube transcript free on YouTubeForge?",
            "Yes. No account. Ads may appear around the tool.",
        ),
        (
            "Do you invent speech from the audio?",
            "No. We read public caption tracks. No captions means no transcript.",
        ),
        (
            "Can I download the transcript?",
            "Yes — TXT, Markdown, JSON, plus SRT/VTT when you need timed files.",
        ),
    ],
};

pub const SRT_VTT: Guide = Guide {
    path: "/guides/srt-vs-vtt",
    title: "SRT vs VTT Subtitles: Which to Download from YouTube | YouTubeForge",
    description: "SRT and WebVTT are timed subtitle formats. Learn the difference and download either from public YouTube captions with YouTubeForge.",
    h1: "SRT vs VTT — which subtitle file should you download?",
    lead: "Both formats store timed lines. Players and editors prefer different ones. YouTubeForge builds SRT and VTT from the same public caption track so you can pick what your tool expects.",
    mode: Mode::Srt,
    sections: &[
        (
            "What SRT is",
            "SubRip (.srt) is the common interchange format: cue number, start → end timestamps, then text. Most desktop video editors and many players open it without fuss.",
        ),
        (
            "What VTT is",
            "WebVTT (.vtt) is the web-native format for HTML5 <track>. Browsers and many streaming workflows expect VTT. It can carry slightly richer cue settings than classic SRT.",
        ),
        (
            "Which should you choose?",
            "Editing in Premiere, CapCut, or VLC → start with SRT. Embedding captions on a website → prefer VTT. Unsure? Download both; they come from the same cues.",
        ),
        (
            "How to download from a YouTube video",
            "Paste the link on /youtube-to-srt (or use the box below), open the result, then download SRT or VTT. Timestamps stay aligned with the caption track YouTube published.",
        ),
        (
            "Transcript text vs subtitle file",
            "A plain transcript is for reading and search. SRT/VTT are for players: they need start, end, and cue order. Use TXT/Markdown when you want quotes; use SRT/VTT when you want on-screen captions.",
        ),
    ],
    faq: &[
        (
            "Can I get SRT if YouTube only has auto-captions?",
            "Yes, when that auto track is public. Quality matches YouTube’s auto captions.",
        ),
        (
            "Is VTT the same as SRT?",
            "Same idea (timed text), different file syntax. Convert by downloading the format you need.",
        ),
        (
            "Do you burn captions into the video?",
            "No. You get a sidecar subtitle file to load in a player or editor.",
        ),
    ],
};

pub const MP3: Guide = Guide {
    path: "/guides/youtube-to-mp3",
    title: "YouTube to MP3: Save Audio from a Public Video | YouTubeForge",
    description: "Download the soundtrack from a public YouTube video as MP3 (or M4A, Opus, WAV). Paste a link — no account. Personal-use and rights still apply.",
    h1: "YouTube to MP3 — save the audio track",
    lead: "Sometimes you need the soundtrack, not the transcript: language practice, a talk without captions, or offline listening. YouTubeForge requests an audio-only stream when YouTube exposes one.",
    mode: Mode::Audio,
    sections: &[
        (
            "How it works",
            "Paste a public watch / Shorts / youtu.be link on /youtube-to-audio. Choose MP3 by default (or M4A, Opus, WAV). We resolve an audio stream YouTube already serves to players — we do not host a pirate library.",
        ),
        (
            "Transcript vs audio",
            "Use the transcript tool when you need searchable words. Use audio when captions are missing or you want to listen. Both stay on the same result URL for that video.",
        ),
        (
            "Length and quality limits",
            "Audio downloads are capped around three hours. Very long uploads or restricted videos can fail. Prefer public videos you are allowed to keep a personal copy of.",
        ),
        (
            "Rights — read this",
            "Only download when you may keep a copy (your video, a license, or a use the law permits). Do not use this as a bulk piracy mirror or to bypass login, age, or region walls.",
        ),
        (
            "After the download",
            "The file saves through your browser. Stay on the result page to grab SRT, a transcript, or a translation of the same video without pasting again.",
        ),
    ],
    faq: &[
        (
            "Is YouTube to MP3 free here?",
            "Yes for the web tool. Ads may appear. No account required.",
        ),
        (
            "What format should I pick?",
            "MP3 for maximum compatibility. M4A/Opus when you want smaller files and your player supports them.",
        ),
        (
            "Why did a download fail?",
            "Private, age-gated, live-only, or region-blocked videos often cannot resolve a plain audio URL. Try another public video.",
        ),
    ],
};

fn guide_json_ld(g: &Guide) -> String {
    let origin = crate::family::public_origin();
    let faq_entities: Vec<_> = g
        .faq
        .iter()
        .map(|(q, a)| {
            json!({
                "@type": "Question",
                "name": q,
                "acceptedAnswer": {"@type": "Answer", "text": a}
            })
        })
        .collect();
    let steps: Vec<_> = g
        .sections
        .iter()
        .take(3)
        .enumerate()
        .map(|(i, (name, text))| {
            json!({
                "@type": "HowToStep",
                "position": i + 1,
                "name": name,
                "text": text
            })
        })
        .collect();
    let doc = json!({
        "@context": "https://schema.org",
        "@graph": [
            {
                "@type": "Article",
                "headline": g.h1,
                "description": g.description,
                "mainEntityOfPage": format!("{origin}{}", g.path),
                "author": {"@type": "Organization", "name": "YouTubeForge"},
                "publisher": {"@type": "Organization", "name": "YouTubeForge", "url": origin}
            },
            {
                "@type": "HowTo",
                "name": g.h1,
                "description": g.lead,
                "step": steps
            },
            {
                "@type": "FAQPage",
                "mainEntity": faq_entities
            }
        ]
    });
    serde_json::to_string(&doc).unwrap_or_else(|_| "{}".into())
}

pub fn render(g: &Guide) -> View {
    set_page_title(g.title);
    set_page_description(g.description);
    set_page_canonical(canonical_url(g.path));
    set_page_json_ld(guide_json_ld(g));

    let sections: Vec<View> = g
        .sections
        .iter()
        .map(|(h, p)| {
            view! {
                <section class="guide-section">
                    <h2>{*h}</h2>
                    <p>{*p}</p>
                </section>
            }
        })
        .collect();
    let faq: Vec<View> = g
        .faq
        .iter()
        .map(|(q, a)| {
            view! {
                <details>
                    <summary>{*q}</summary>
                    <p>{*a}</p>
                </details>
            }
        })
        .collect();
    let tool_href = g.mode.landing_path().to_string();
    let tool_label = match g.mode {
        Mode::Text => "Open YouTube to text",
        Mode::Audio => "Open YouTube to MP3",
        Mode::Srt => "Open SRT / VTT download",
        Mode::Translate => "Open translator",
        Mode::Summary => "Open summary",
    };

    view! {
        <main class="home-page landing-page guide-page" lang="en">
            <div class="hero-wrap">
                <div class="hero-particles" data-hero-particles="" aria-hidden="true"></div>
                <section class="hero">
                    <div class="hero-copy">
                        <p class="eyebrow">"Guide"</p>
                        <h1>{g.h1}</h1>
                        <p class="hero-lead">{g.lead}</p>
                        {home_search(g.mode)}
                    </div>
                </section>
            </div>

            <article class="content-section guide-article">
                {sections}
                <p class="error-actions">
                    <NavLink href={tool_href} class="btn btn-primary">{tool_label}</NavLink>
                    <NavLink href="/guides" class="btn btn-ghost">"All guides"</NavLink>
                </p>
            </article>

            {crate::ads::slot("landing-mid", "infeed")}

            <section class="faq" aria-labelledby="guide-faq">
                <h2 id="guide-faq">"FAQ"</h2>
                <div class="faq-list">{faq}</div>
            </section>

            {crate::cross_sell::related(g.mode)}
            {crate::cross_sell::guides_nav(Some(g.path))}
        </main>
    }
}

pub fn index_page() -> View {
    set_page_title("Guides — YouTube Transcript, SRT, MP3 | YouTubeForge");
    set_page_description(
        "Short guides: get a YouTube transcript, choose SRT vs VTT, and save audio as MP3. Free tools on forgeyt.com.",
    );
    set_page_canonical(canonical_url("/guides"));

    let cards: Vec<View> = ALL
        .iter()
        .map(|g| {
            let href = g.path.to_string();
            view! {
                <li>
                    <a href={href} class="related-card" data-r-nav="true">
                        <strong>{g.h1}</strong>
                        <span>{g.description}</span>
                    </a>
                </li>
            }
        })
        .collect();

    view! {
        <main class="content-section guide-page">
            <p class="eyebrow">"Learn"</p>
            <h1>"YouTubeForge guides"</h1>
            <p class="hero-lead">
                "Practical pages for the jobs people search for — then the same paste box to do it."
            </p>
            <ul class="related-grid">{cards}</ul>
            <p class="error-actions">
                <NavLink href="/" class="btn btn-primary">"Paste a YouTube link"</NavLink>
                <NavLink href="/extension" class="btn btn-ghost">"Chrome extension"</NavLink>
            </p>
        </main>
    }
}
