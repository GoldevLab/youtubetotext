//! One working URL: `/` and `/?v={id}&mode=…`.

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

}

pub fn app_href(video_id: &str, mode: Mode) -> String {
    if video_id.is_empty() {
        "/".into()
    } else {
        format!("/?v={video_id}&mode={}", mode.slug())
    }
}
