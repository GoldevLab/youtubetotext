use resuma::prelude::*;
use resuma::FlowPageRegistry;

pub struct PagesRegistry;

const LAYOUT: &[&str] = &["/"];

impl FlowPageRegistry for PagesRegistry {
    fn routes(&self) -> &'static [(&'static str, &'static str)] {
        &[
            ("/", "index"),
            ("/youtube-to-text", "youtube_to_text"),
            ("/youtube-to-audio", "youtube_to_audio"),
            ("/youtube-translator", "youtube_translator"),
            ("/youtube-summary", "youtube_summary"),
            ("/youtube-to-srt", "youtube_to_srt"),
            ("/guides", "guides"),
            ("/guides/youtube-transcript", "guide_youtube_transcript"),
            ("/guides/srt-vs-vtt", "guide_srt_vs_vtt"),
            ("/guides/youtube-to-mp3", "guide_youtube_to_mp3"),
            ("/privacy", "privacy"),
            ("/terms", "terms"),
            ("/extension", "extension"),
            ("/pricing", "pricing"),
            ("/developers", "developers"),
            ("/developers/welcome", "developers_welcome"),
        ]
    }

    fn layout_for(&self, pattern: &str) -> &'static [&'static str] {
        match pattern {
            _ => LAYOUT,
        }
    }

    fn render(&self, module: &str, req: FlowRequest) -> Option<View> {
        match module {
            "index" => Some(super::index::page(req)),
            "youtube_to_text" => Some(super::youtube_to_text::page(req)),
            "youtube_to_audio" => Some(super::youtube_to_audio::page(req)),
            "youtube_translator" => Some(super::youtube_translator::page(req)),
            "youtube_summary" => Some(super::youtube_summary::page(req)),
            "youtube_to_srt" => Some(super::youtube_to_srt::page(req)),
            "guides" => Some(super::guides::page(req)),
            "guide_youtube_transcript" => Some(super::guide_youtube_transcript::page(req)),
            "guide_srt_vs_vtt" => Some(super::guide_srt_vs_vtt::page(req)),
            "guide_youtube_to_mp3" => Some(super::guide_youtube_to_mp3::page(req)),
            "privacy" => Some(super::privacy::page(req)),
            "terms" => Some(super::terms::page(req)),
            "extension" => Some(super::extension::page(req)),
            "pricing" => Some(super::pricing::page(req)),
            "developers" => Some(super::developers::page(req)),
            "developers_welcome" => Some(super::developers_welcome::page(req)),
            _ => None,
        }
    }
}
