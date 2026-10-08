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
    /// Tool and guide URLs this article should pass readers to.
    pub links: &'static [(&'static str, &'static str)],
}

pub const ALL: &[Guide] = &[TRANSCRIPT, SEE_ON_YOUTUBE, SRT_VTT, MP3];

pub const TRANSCRIPT: Guide = Guide {
    path: "/guides/youtube-transcript",
    title: "How to Get a Transcript of a YouTube Video | YouTubeForge",
    description: "How to get a transcript from any public YouTube video, transcribe it from the captions, and download TXT or SRT. No account.",
    h1: "How to get a transcript from any YouTube video",
    lead: "Paste the link. If YouTube published captions, you get the transcript as searchable text — then you can copy it or download a file. You do not re-type the video, and we do not run speech-to-text on our servers.",
    mode: Mode::Text,
    sections: &[
        (
            "How to get a transcript of a YouTube video",
            "Copy a watch URL, a Shorts link, a youtu.be link, or the 11-character video id. Paste it in the box on this page, or on the YouTube transcript tool at forgeyt.com. The lines open next to the player at /?v= (that result URL stays noindex). Search a name, click a line to jump, and copy the quote.",
        ),
        (
            "How to get the transcript from a YouTube video link",
            "The link is the whole input. Private, age-restricted, members-only, and many livestreams fail because the caption track is not public. A normal public upload with auto-captions or a human track works.",
        ),
        (
            "How to transcribe a YouTube video",
            "Transcribe here means reading the caption track YouTube already made, human or automatic. It does not mean sending the audio through another speech model. If you only need to follow along inside YouTube, use Show transcript on the watch page. Use YouTubeForge when you need search, a copy, or a file.",
        ),
        (
            "How to download a YouTube transcript",
            "Open the video in the tool, then save TXT or Markdown to read, or SRT/VTT when an editor needs timestamps. The download page walks through the formats. The file matches the lines on screen, including any edit you made in the browser.",
        ),
        (
            "When the transcript beats watching",
            "Hour-long interviews, lectures, and podcast uploads are faster as text when you need a quote, a name, or a claim. Download subtitles if the next step is a player, not a doc.",
        ),
        (
            "Limits to expect",
            "No captions published means nothing to show. Auto-captions mis-hear names; fix them on the page before you cite them. We are not YouTube or Google.",
        ),
    ],
    faq: &[
        (
            "How do I get a transcript of a YouTube video for free?",
            "Paste the public link on YouTubeForge. No account. Ads may appear around the tool.",
        ),
        (
            "Can I transcribe a video that has no captions?",
            "Not here. We do not invent speech from the audio. No public caption track means no transcript.",
        ),
        (
            "Can I download the transcript after I generate it?",
            "Yes. TXT, Markdown, JSON, SRT, and VTT. Start from the download page if the file is the whole job.",
        ),
    ],
    links: &[
        ("/", "YouTube transcript tool"),
        ("/youtube-transcript-download", "Download the transcript"),
        ("/youtube-transcript-generator", "Free transcript generator"),
        ("/convert-youtube-video-to-text", "Convert the video to text"),
        ("/youtube-to-srt", "Download subtitles (SRT / VTT)"),
        ("/guides/see-transcript-on-youtube", "See the transcript inside YouTube"),
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
    links: &[
        ("/youtube-to-srt", "Download YouTube subtitles"),
        ("/youtube-transcript-download", "Download the transcript as text"),
        ("/guides/youtube-transcript", "How to get a transcript"),
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
    links: &[
        ("/youtube-to-audio", "YouTube to MP3"),
        ("/", "Get the transcript instead"),
        ("/guides/youtube-transcript", "How to get a transcript"),
    ],
};

pub const SEE_ON_YOUTUBE: Guide = Guide {
    path: "/guides/see-transcript-on-youtube",
    title: "How to See the Transcript on YouTube | YouTubeForge",
    description: "How to see, find, open, and show the transcript on YouTube — desktop and phone — and what to do when Show transcript is missing.",
    h1: "How to see the transcript on YouTube",
    lead: "YouTube can show the caption track on the watch page. Open that panel when you only need to follow along. Use YouTubeForge when you need to search, copy, or download the same words.",
    mode: Mode::Text,
    sections: &[
        (
            "How to see the transcript on YouTube (computer)",
            "Open the video on youtube.com. Under the title, expand the description (…more). Click Show transcript. A panel lists the lines with timestamps. Click a line to jump the video. The button sits in the description area, not in the player gear menu.",
        ),
        (
            "How to find the transcript when you do not see a button",
            "Scroll the description again. Show transcript only appears when that video has a caption track. The CC button in the player turns captions on the picture; it does not open the transcript panel. If neither CC nor Show transcript exists, YouTube never published captions for that upload.",
        ),
        (
            "How to open the transcript on your phone",
            "In the YouTube app or mobile site, open the video, tap the description (or the arrow under the title), then tap Show transcript. The lines replace the description until you close the panel.",
        ),
        (
            "How to view or show the transcript",
            "View, show, and open are the same control: Show transcript, after the description is expanded. There is no separate switch in Settings. If the panel is blank, switch caption language from the panel menu when the video has more than one track.",
        ),
        (
            "How to pull up a transcript when YouTube has none",
            "You cannot. Show transcript is hidden when there is no public caption track. YouTubeForge reads that same track, so a missing button there means no transcript here either. We do not listen to the audio and write a new one.",
        ),
        (
            "When the on-YouTube panel is not enough",
            "The panel does not hand you a TXT, SRT, or VTT file, and it is awkward to search across a long interview. Paste the same link into YouTubeForge to search the lines, copy a quote, or download the file.",
        ),
    ],
    faq: &[
        (
            "Where is Show transcript?",
            "On a computer, expand the description under the video, then click Show transcript. On a phone, open the description first.",
        ),
        (
            "Why can’t I find the transcript?",
            "The video has no caption track, the description is still collapsed, or you are looking in the player settings instead of under the video.",
        ),
        (
            "Can I download the transcript from that panel?",
            "YouTube’s panel is for reading and jumping. To save TXT, SRT, or VTT, paste the link on YouTubeForge.",
        ),
    ],
    links: &[
        ("/", "Get a searchable YouTube transcript"),
        ("/youtube-transcript-download", "Download the transcript"),
        ("/guides/youtube-transcript", "How to get a transcript from any video"),
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
    let related_links: Vec<View> = g
        .links
        .iter()
        .map(|(href, label)| {
            let href = (*href).to_string();
            view! {
                <li>
                    <NavLink href={href}>{*label}</NavLink>
                </li>
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
                <h2>"Use the tool"</h2>
                <ul class="guide-links">{related_links}</ul>
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
        "How to get a YouTube transcript, how to open it on YouTube, SRT vs VTT, and YouTube to MP3. Each guide ends on the free tool.",
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
