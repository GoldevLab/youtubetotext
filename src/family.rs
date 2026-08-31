//! YouTubeForge: one working URL (`/`). SEO articles stay on their own paths.

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Text,
    Audio,
    Translate,
    Summary,
    Srt,
}

impl Mode {
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "audio" => Self::Audio,
            "translate" | "translator" => Self::Translate,
            "summary" => Self::Summary,
            "srt" | "vtt" | "subtitles" => Self::Srt,
            _ => Self::Text,
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Audio => "audio",
            Self::Translate => "translate",
            Self::Summary => "summary",
            Self::Srt => "srt",
        }
    }

    pub fn landing_path(self) -> &'static str {
        match self {
            Self::Text => "/youtube-to-text",
            Self::Audio => "/youtube-to-audio",
            Self::Translate => "/youtube-translator",
            Self::Summary => "/youtube-summary",
            Self::Srt => "/youtube-to-srt",
        }
    }

    pub fn nav_label(self) -> &'static str {
        match self {
            Self::Text => "Transcript",
            Self::Audio => "Audio",
            Self::Translate => "Translate",
            Self::Summary => "Summary",
            Self::Srt => "SRT",
        }
    }

    pub fn cta(self) -> &'static str {
        match self {
            Self::Text => "Get transcript",
            Self::Audio => "Get audio",
            Self::Translate => "Translate captions",
            Self::Summary => "Summarize video",
            Self::Srt => "Download SRT",
        }
    }

    pub fn all() -> [Mode; 5] {
        [
            Self::Text,
            Self::Audio,
            Self::Translate,
            Self::Summary,
            Self::Srt,
        ]
    }
}

pub fn app_href(video_id: &str, mode: Mode) -> String {
    if video_id.is_empty() {
        "/".into()
    } else {
        format!("/?v={video_id}&mode={}", mode.slug())
    }
}

pub struct Landing {
    pub title: &'static str,
    pub description: &'static str,
    pub eyebrow: &'static str,
    pub h1: &'static str,
    pub lead: &'static str,
    pub howto_title: &'static str,
    pub howto: [(&'static str, &'static str); 3],
    pub why_title: &'static str,
    pub why: [(&'static str, &'static str); 4],
    pub examples_title: &'static str,
    pub examples: [(&'static str, &'static str); 3],
    pub limits: &'static str,
    pub faq: [(&'static str, &'static str); 5],
}

pub fn landing_for(mode: Mode) -> Landing {
    match mode {
        Mode::Text => TEXT,
        Mode::Audio => AUDIO,
        Mode::Translate => TRANSLATE,
        Mode::Summary => SUMMARY,
        Mode::Srt => SRT,
    }
}

const TEXT: Landing = Landing {
    title: "YouTube to Text — Free Transcript from Any Public Video | YouTubeForge",
    description: "Paste a YouTube URL and get a searchable transcript. Copy, download TXT/Markdown, jump timestamps. No account.",
    eyebrow: "YouTube → text",
    h1: "Turn a YouTube video into a searchable transcript",
    lead: "This page is for people who want the spoken words as text — notes, quotes, search, or paste into another tool. Captions come from YouTube’s public tracks. We do not re-transcribe audio on our servers.",
    howto_title: "How to get a YouTube transcript",
    howto: [
        ("Paste the link", "Watch, Shorts, youtu.be, or an 11-character id. You land in the app with the transcript beside a privacy-friendly player."),
        ("Find the line", "Search, skip intro/outro, or click two lines to keep a range. Click a cue to jump the video."),
        ("Copy or share", "Copy plain text or Markdown with timestamp links. Every video also has a shareable page at /v/{id}."),
    ],
    why_title: "What this transcript tool is for",
    why: [
        ("Reading, not watching", "Lectures, interviews, and long explainers are faster as text when you already know you need a quote or a definition."),
        ("Shareable URLs", "The transcript lives in HTML at /v/{id}. Notes apps and search engines can read it — it is not trapped in a canvas."),
        ("Edits stay on your device", "Fix an auto-caption typo in Edit mode. Copies and downloads use the text you see."),
        ("No cookie wall", "Public captions only. No account. We are not affiliated with YouTube or Google."),
    ],
    examples_title: "Typical transcript jobs",
    examples: [
        ("Interview quote", "Search a guest’s name, copy three lines, paste into an article with a timestamp link."),
        ("Lecture notes", "Skip the 45-second intro, copy the rest as Markdown into your notes."),
        ("Accessibility check", "Read auto-captions before you rely on them; Edit if a term is wrong."),
    ],
    limits: "If YouTube never published captions (human or auto), there is nothing to extract. Age-restricted, private, and live-only videos often fail. We do not invent speech from the audio file on this page — use Audio if you need the soundtrack, or SRT if you need a subtitle file.",
    faq: [
        ("Is YouTubeForge free?", "Yes. No sign-up. Ads may appear around the page; the transcript itself is not paywalled."),
        ("Do you transcribe the audio with speech-to-text?", "No. We load YouTube’s existing caption tracks. If the video has no captions, this tool cannot invent them."),
        ("Where does the transcript live after I generate it?", "On the home page with ?v={videoId}, and as a shareable document at /v/{videoId}. Optional query: lang, tlang, and mode (audio, translate, summary, srt)."),
        ("Is there a length cap?", "We do not add one. If YouTube has captions for a public video, we load them."),
        ("Can I keep timestamps?", "Yes. Copy with timestamps, Copy Markdown (YouTube time links), or download timed TXT."),
    ],
};

const AUDIO: Landing = Landing {
    title: "YouTube to Audio — Download Public Video Audio (M4A/WebM) | YouTubeForge",
    description: "Extract the audio stream from a public YouTube video for personal playback. M4A or WebM when YouTube exposes a direct URL. Then transcribe or subtitle in the same app.",
    eyebrow: "YouTube → audio",
    h1: "Download audio from a public YouTube video",
    lead: "This page is for the soundtrack, not the transcript. We ask YouTube’s player API for an audio-only stream (usually M4A/AAC or WebM/Opus) and stream that file to you. It is a different job from captions: talks without subtitles can still have audio; videos with captions are not required.",
    howto_title: "How to save YouTube audio",
    howto: [
        ("Paste the watch URL", "Same links as the transcript tool. The app requests an audio-only adaptive stream, not a muxed 1080p file."),
        ("Download the file", "You get an attachment (M4A or WebM) through /api/audio. Play it in a local player. We do not convert to MP3 in the browser."),
        ("Do the next job", "When the file is ready, jump to transcript, SRT, translation, or a chapter summary in the same app."),
    ],
    why_title: "Why this is not a generic “YouTube MP3” clone",
    why: [
        ("Audio-only streams", "We prefer adaptive audio (itag 140/251 class) so you are not downloading video pixels."),
        ("Same family as captions", "After audio, one click to text, SRT, or a summary — one session, several intents."),
        ("No ffmpeg.wasm on the landing", "The SEO page stays light. Heavy work is the app + a streaming API."),
        ("Honest failures", "If YouTube only returns a signature-cipher URL we cannot decode, we say so. We do not pretend every video is downloadable."),
    ],
    examples_title: "When audio-only is the right intent",
    examples: [
        ("Podcast-style watch later", "Save the talk for a phone player that does not need the video."),
        ("Language listening", "Play the original audio while you read a translated transcript in another tab."),
        ("No captions", "The video has speech but YouTube never shipped a timedtext track — audio still may work."),
    ],
    limits: "Only public videos we can resolve through InnerTube. Ciphered streams, DRM-ish formats, and region/age walls fail. Use this for content you are allowed to keep a personal copy of. We are not a piracy mirror and we do not host a library of files.",
    faq: [
        ("Is this YouTube to MP3?", "We download the audio stream YouTube already serves (often M4A or WebM), not a re-encoded MP3. Many players open M4A directly."),
        ("Why did download fail?", "The player JSON had no plain audio URL (signature/n-param), or the video is blocked for this network. Try another public video."),
        ("How large are the files?", "Roughly the audio bitrate times duration (for example ~1 MB per minute at 128 kbps AAC). We stream; we do not buffer the whole file in RAM."),
        ("Can I clip a range?", "Not on the download yet. Trim exists on the transcript side. For audio, download the full stream."),
        ("Is it free?", "Yes, with the same ads-around-the-tool model as the rest of YouTubeForge."),
    ],
};

const TRANSLATE: Landing = Landing {
    title: "YouTube Translator — Translate Captions with YouTube tlang | YouTubeForge",
    description: "Translate YouTube captions into another language using YouTube’s own caption translation (tlang). Timed lines stay timed. Share the language in the URL.",
    eyebrow: "YouTube captions → another language",
    h1: "Translate a YouTube transcript without leaving the timestamps",
    lead: "This is not a generic paragraph translator. We load a caption track, then ask YouTube to auto-translate it (`tlang`) so each line keeps its start time. That is what you want for subtitles, study, and jumping the player — not a blob of translated prose with no cues.",
    howto_title: "How caption translation works here",
    howto: [
        ("Open the video in the app", "Pick the source track (English auto vs human, Spanish, Japanese, …)."),
        ("Choose Translate", "The tlang list is YouTube’s translation catalog. The URL stores lang and tlang so you can share the same view."),
        ("Export if you need a file", "Download SRT/VTT in the translated lines, or copy Markdown with timestamps."),
    ],
    why_title: "Why use YouTube’s translator instead of pasting into a chat",
    why: [
        ("Line alignment", "A chat model will merge sentences and drop times. tlang keeps cue boundaries."),
        ("Shareable language pair", "/v/{id}?lang=en&tlang=es is a document, not a one-off paste."),
        ("Same downloads", "Translated cues still export as SRT, VTT, TXT, Markdown, JSON."),
        ("No extra API key", "We do not send the transcript to a third-party MT API."),
    ],
    examples_title: "Translation jobs this page is built for",
    examples: [
        ("Study a talk in your language", "Keep English audio, read Spanish cues, click a line to hear the original."),
        ("Bilingual subtitles file", "Translate then download SRT for a player that needs a .srt."),
        ("Channel not in your language", "Use auto-captions as source, then tlang — quality follows YouTube’s MT."),
    ],
    limits: "Translation quality is YouTube’s, including names and jargon mistakes. Tracks marked non-translatable cannot use tlang. We do not run a separate neural MT model.",
    faq: [
        ("Is this a full YouTube video translator?", "We translate captions, not burned-in video pixels or the audio track."),
        ("Which languages?", "Whatever YouTube lists on that video’s translatable track. The app shows the catalog."),
        ("Does the share link remember the language?", "Yes: lang for the source track, tlang for the translation."),
        ("Can I edit a bad translation?", "Edit mode changes the text on your device, then copy/download."),
        ("What if there are no captions?", "Nothing to translate. Try Audio, or pick a video with auto-captions on."),
    ],
};

const SUMMARY: Landing = Landing {
    title: "YouTube Summary — Chapter Recap + Prompt from the Transcript | YouTubeForge",
    description: "Summarize a YouTube video from its captions: chapter-aware recap on the server, plus a copy-paste prompt for the model you already use. No extra AI bill.",
    eyebrow: "YouTube → summary",
    h1: "Summarize a YouTube video from its transcript, not from a black-box recap",
    lead: "Search “YouTube summary” and you often get a model that watched nothing and billed you. Here the source of truth is the caption file. We build a chapter-aware recap from those lines, and we also copy a tight prompt so you can paste the transcript into the LLM you already pay for.",
    howto_title: "How to summarize a video here",
    howto: [
        ("Load captions", "Same paste-a-link flow. Chapters come from YouTube when the creator set them."),
        ("Read the recap", "Each chapter (or the whole talk) gets the opening sentences of that span — extractive, not hallucinated."),
        ("Optional: your model", "Copy the Summary prompt (transcript included) into ChatGPT, Claude, or a local model for a tighter rewrite."),
    ],
    why_title: "Why this beats a one-shot “AI summary” site",
    why: [
        ("You can check the cues", "Every claim in the recap is a slice of the transcript you can open underneath."),
        ("Chapters matter", "A 2-hour lecture is not one paragraph. We split on creator chapters when they exist."),
        ("You keep the model", "We do not lock summaries behind our API meter. Prompts are local copy."),
        ("Same tool family", "After the recap: download SRT, translate, or grab audio."),
    ],
    examples_title: "Summary intents that are actually different",
    examples: [
        ("Conference talk", "Use chapters as an outline; skim the recap before committing 40 minutes."),
        ("Study notes", "Copy the Notes prompt instead of Summary — headings and glossary, still from captions."),
        ("Quote hunting", "Use Quotes prompt, then click timestamps in the transcript."),
    ],
    limits: "Garbage captions in, garbage recap out. We do not watch the pixels, so visuals-only jokes and slides without speech will not appear. No chapters means one recap for the whole file.",
    faq: [
        ("Do you use GPT on our servers?", "No. The on-page recap is extractive (sentences from the captions). The Summary button copies a prompt for your own model."),
        ("How long does it take?", "Caption fetch is usually about a second. The recap is instant after that."),
        ("Can I summarize a clipped range?", "Trim the cues first, then copy the Summary prompt — it uses visible lines."),
        ("What if auto-captions are messy?", "Edit obvious errors, then recap/copy. We do not clean ASR for you."),
        ("Is it free?", "Yes. Same AdSense-around-the-flow model."),
    ],
};

const SRT: Landing = Landing {
    title: "YouTube to SRT / VTT — Download Subtitles from Captions | YouTubeForge",
    description: "Download YouTube captions as SRT or VTT for players, editors, and burns. Timed cues, language tracks, optional translation. Not a screenshot of the transcript.",
    eyebrow: "YouTube → SRT / VTT",
    h1: "Download YouTube subtitles as SRT or VTT",
    lead: "This page is for a subtitle file you can drop into VLC, Premiere, or a website <track>. That is a different artifact from “copy the transcript”: SRT/VTT need cue index, start, end, and text. We build those from YouTube’s timed captions.",
    howto_title: "How to get an SRT from YouTube",
    howto: [
        ("Paste the video", "Open it in the app so cues have start and duration."),
        ("Pick the track", "Human captions usually beat auto for names. Translate first if you need another language in the file."),
        ("Download SRT or VTT", "SRT uses comma milliseconds; VTT is WEBVTT with dots. Same cues, different wrapping."),
    ],
    why_title: "Why a dedicated SRT landing",
    why: [
        ("Players want files", "Copy-paste into a doc is not a subtitle. SRT is."),
        ("VTT for the web", "If you are adding captions to HTML5 video, download VTT, not TXT."),
        ("Trim then export", "Skip intro/outro or click two lines; the file only includes the range you kept."),
        ("API too", "GET /api/transcript?v=…&fmt=srt for scripts."),
    ],
    examples_title: "Subtitle jobs this page targets",
    examples: [
        ("Local playback", "Download SRT, sit it next to the file in VLC."),
        ("Course clip", "Trim to the demo section, export VTT for the lesson page."),
        ("Translation file", "tlang then SRT — a translated subtitle, still timed."),
    ],
    limits: "No captions means no SRT. We do not OCR burned-in hardsubs. Cue timing follows YouTube; overlapping auto-captions are collapsed when they repeat.",
    faq: [
        ("SRT vs VTT?", "SRT is the classic editor/player format. VTT is the web standard (WEBVTT). We offer both from the same cues."),
        ("UTF-8?", "Yes. Use it for non-English tracks."),
        ("Can I get JSON instead?", "Yes — fmt=json on the API, or the JSON button in the app."),
        ("Does Edit mode affect the SRT?", "Downloads from the on-page buttons use the cues you see, including local edits."),
        ("Is there a cue limit?", "We refuse absurd ingest sizes; normal videos are fine."),
    ],
};
