//! Keyword landings that share the paste form but target a different job.
//! Home owns "youtube transcript" / "transcribe youtube video".
//! These pages own download, free generator, and convert-to-text.

use crate::family::{Landing, Mode};

pub struct KeywordPage {
    pub path: &'static str,
    pub mode: Mode,
    pub landing: Landing,
}

pub const DOWNLOAD: KeywordPage = KeywordPage {
    path: "/youtube-transcript-download",
    mode: Mode::Text,
    landing: Landing {
        title: "YouTube Transcript Download — Free TXT, SRT, and VTT | YouTubeForge",
        description: "Download a YouTube transcript from a public video. Save TXT or Markdown to read, or SRT and VTT when you need timestamps. No account.",
        eyebrow: "YouTube transcript download",
        h1: "Download a YouTube transcript",
        lead: "Paste a public YouTube link and save the caption track as a file. TXT and Markdown are for reading and notes. SRT and VTT are for a player or editor. This is a downloader for captions YouTube already published, not a new recording of the audio.",
        howto_title: "How to download a YouTube transcript",
        howto: [
            ("Paste the link", "Watch, Shorts, youtu.be, or the 11-character video id. The transcript opens beside the player."),
            ("Check the lines", "Search a phrase, trim the intro, or switch caption language before you save. The file matches what you see."),
            ("Save the file", "Download TXT, Markdown, JSON, SRT, or VTT. The browser saves it. We do not keep a copy."),
        ],
        why_title: "What a transcript download is for",
        why: [
            ("A file you can keep", "Copy-to-clipboard disappears. A download sits in your notes, a repo, or an editor project."),
            ("Pick the format", "Plain TXT for reading. Markdown when you want timestamp links. SRT or VTT when a player needs cues."),
            ("Same public captions", "The file is YouTube’s caption track. Auto-captions can mis-hear names; edit on the page, then download."),
            ("No account", "The downloader does not ask you to sign up or pay per minute."),
        ],
        examples_title: "Downloads people actually need",
        examples: [
            ("Notes archive", "Download TXT from a lecture and drop it next to the slides."),
            ("Article quotes", "Download Markdown so each quote still links to the moment in the video."),
            ("Editor sidecar", "Download SRT when the transcript has to play as subtitles, not sit in a doc."),
        ],
        limits: "No public captions means nothing to download. Private, age-restricted, and live-only videos often fail. We do not invent a transcript from the audio on this page.",
        faq: [
            ("Is this a YouTube transcript downloader?", "Yes. Paste a public link and save TXT, Markdown, JSON, SRT, or VTT. No account."),
            ("TXT or SRT?", "TXT or Markdown to read and quote. SRT or VTT if the next step is a player or editor. Both come from the same cues."),
            ("Where does the file go?", "Your browser downloads it. YouTubeForge does not store the transcript on a server for you."),
            ("Can I download just part of the video?", "Trim the lines on the result page first. The download uses the cues you kept."),
            ("What if the video has no captions?", "There is no file to save. YouTube never published a track, so we cannot invent one."),
        ],
    },
};

pub const GENERATOR: KeywordPage = KeywordPage {
    path: "/youtube-transcript-generator",
    mode: Mode::Text,
    landing: Landing {
        title: "YouTube Transcript Generator Free — No Account | YouTubeForge",
        description: "Free YouTube transcript generator. Paste a public link and get the caption text in the browser. No sign-up and no per-minute charge.",
        eyebrow: "Free transcript generator",
        h1: "Free YouTube transcript generator",
        lead: "Generate a transcript from a public YouTube video without creating an account. The generator reads the caption track YouTube already published and shows it as searchable lines. It does not run a paid speech-to-text job, and it does not invent words that were never captioned.",
        howto_title: "How the free generator works",
        howto: [
            ("Paste a public link", "Any watch URL, Short, or youtu.be link. No login on YouTubeForge."),
            ("Generate the lines", "We fetch the public caption track and lay it out with timestamps next to the player."),
            ("Use the text", "Search, copy, or download. The result URL stays noindex so one-off videos do not flood search."),
        ],
        why_title: "Why this generator is free",
        why: [
            ("No speech-to-text bill", "Paid generators charge because they run a model on the audio. This one uses captions YouTube already made."),
            ("No account wall", "You paste a link and get the text. Ads may show around the tool. The transcript is not locked."),
            ("You can check every line", "Each row is a caption cue. Click it to hear that moment before you trust a quote."),
            ("Same video, next job", "After the text is on screen you can download subtitles, translate the cues, or save audio."),
        ],
        examples_title: "When a free generator is the right tool",
        examples: [
            ("One video, right now", "A podcast upload or a lecture you do not want to rewatch."),
            ("Check auto-captions", "See what YouTube heard before you cite it. Edit a name on the page if it is wrong."),
            ("Hand the text to your own model", "Copy the transcript into the AI you already use, instead of paying a wrapper site."),
        ],
        limits: "If the video has no public captions, the generator has nothing to show. It will not whisper the audio for you. Very new livestreams often have no track yet.",
        faq: [
            ("Is the YouTube transcript generator free?", "Yes. No account and no minute pack. Ads may appear around the page."),
            ("Do you generate text with AI?", "No. We load YouTube’s caption track, human or auto. No captions means no transcript."),
            ("How is this different from downloading a file?", "Generating shows the lines in the browser so you can search and jump. Download is the separate step when you want TXT, SRT, or VTT on disk."),
            ("Which languages?", "Whatever caption tracks that video publishes. Translation of those cues is a separate tool."),
            ("Can I share the result?", "Yes. The working link is /?v={id}&mode=text. Those result URLs are noindex."),
        ],
    },
};

pub const CONVERT: KeywordPage = KeywordPage {
    path: "/convert-youtube-video-to-text",
    mode: Mode::Text,
    landing: Landing {
        title: "Convert YouTube Video to Text — Free | YouTubeForge",
        description: "Convert a YouTube video to text. Paste the link and get the spoken words as plain text for notes, quotes, or another tool. No account.",
        eyebrow: "YouTube video → text",
        h1: "Convert a YouTube video to text",
        lead: "Conversion here means the spoken track becomes plain text you can read, search, and paste. You do not get a new video file, and you do not get burned-in captions. The source is the public caption track on that video.",
        howto_title: "How to convert a YouTube video to text",
        howto: [
            ("Copy the video link", "The watch URL is enough. Shorts and youtu.be links work the same way."),
            ("Paste and convert", "Submit the form. The text opens as lines with timestamps, next to the player."),
            ("Take the text with you", "Copy plain text into notes, or download TXT or Markdown. Use SRT only if you needed a subtitle file instead."),
        ],
        why_title: "Text, not another video file",
        why: [
            ("A document", "Lectures, interviews, and explainers are easier to scan as text when you already know the claim you need."),
            ("Search inside the video", "Find a name or a phrase, then click the line to hear it."),
            ("Paste into what you already use", "Notes, a doc, or a model you pay for. The conversion step stops at the caption text."),
            ("Honest about the source", "We convert captions to text. We do not listen to the audio and write a fresh transcript on our servers."),
        ],
        examples_title: "Conversions this page is for",
        examples: [
            ("Meeting-style recap", "Turn a long upload into text, skip the intro, copy the rest."),
            ("Quote with a timestamp", "Convert, search the phrase, copy Markdown so the quote still points at the video."),
            ("Feed another tool", "Some apps want a transcript pasted in. This page is that paste."),
        ],
        limits: "Videos with no public captions cannot be converted here. On-screen text that was never spoken is not included. Private and age-restricted videos usually fail.",
        faq: [
            ("Will this convert the video file itself?", "No. You get the caption text. Audio and subtitle downloads are separate tools."),
            ("Is the text the full script?", "It is the caption track, which usually follows the speech. Auto-captions can miss names and jargon."),
            ("Can I convert a Short?", "Yes, when that Short has public captions."),
            ("Do timestamps survive the conversion?", "On the page, yes. Copy with timestamps or download Markdown if you want them in the file. Plain copy is text only."),
            ("Is it free?", "Yes. No account. Ads may appear around the tool."),
        ],
    },
};

pub const ALL: &[KeywordPage] = &[DOWNLOAD, GENERATOR, CONVERT];
