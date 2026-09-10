//! Subscription plans — priced to beat youtube-transcript.io / TranscriptAPI
//! on transcripts-per-dollar while keeping audio/video hard-capped (expensive).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Plan {
    Basic,
    Pro,
}

impl Plan {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "basic" | "starter" => Some(Self::Basic),
            "pro" => Some(Self::Pro),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Basic => "basic",
            Self::Pro => "pro",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Basic => "Basic",
            Self::Pro => "Pro",
        }
    }

    /// List price shown on /pricing (USD / month).
    pub fn price_usd(self) -> u32 {
        match self {
            Self::Basic => 9,
            Self::Pro => 29,
        }
    }

    pub fn transcripts_month(self) -> u32 {
        match self {
            // Competitors: ~1k @ $10. We give 5× for less.
            Self::Basic => 5_000,
            // Competitors: ~3k @ $25. We give ~8×.
            Self::Pro => 25_000,
        }
    }

    pub fn translates_month(self) -> u32 {
        match self {
            Self::Basic => 1_000,
            Self::Pro => 5_000,
        }
    }

    pub fn audio_month(self) -> u32 {
        match self {
            Self::Basic => 100,
            Self::Pro => 400,
        }
    }

    pub fn video_sd_month(self) -> u32 {
        match self {
            Self::Basic => 0,
            Self::Pro => 80,
        }
    }

    pub fn video_720_month(self) -> u32 {
        match self {
            Self::Basic => 0,
            Self::Pro => 20,
        }
    }

    /// HD/4K stays web-only (too expensive to sell via API).
    pub fn video_hd_month(self) -> u32 {
        0
    }

    pub fn rate_per_min(self) -> u32 {
        match self {
            Self::Basic => 120,
            Self::Pro => 300,
        }
    }

    pub fn lemon_variant_env(self) -> &'static str {
        match self {
            Self::Basic => "LEMON_VARIANT_BASIC",
            Self::Pro => "LEMON_VARIANT_PRO",
        }
    }

    pub fn from_variant_id(variant_id: &str) -> Option<Self> {
        let basic = std::env::var("LEMON_VARIANT_BASIC").ok();
        let pro = std::env::var("LEMON_VARIANT_PRO").ok();
        if basic.as_deref() == Some(variant_id) {
            Some(Self::Basic)
        } else if pro.as_deref() == Some(variant_id) {
            Some(Self::Pro)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Meter {
    Transcript,
    Translate,
    Audio,
    VideoSd,
    Video720,
    VideoHd,
}

impl Plan {
    pub fn meter_cap(self, meter: Meter) -> u32 {
        match meter {
            Meter::Transcript => self.transcripts_month(),
            Meter::Translate => self.translates_month(),
            Meter::Audio => self.audio_month(),
            Meter::VideoSd => self.video_sd_month(),
            Meter::Video720 => self.video_720_month(),
            Meter::VideoHd => self.video_hd_month(),
        }
    }
}
