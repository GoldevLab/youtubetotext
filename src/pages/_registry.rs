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
            ("/youtube-a-texto", "youtube_a_texto"),
            ("/youtube-a-mp3", "youtube_a_mp3"),
            ("/youtube-traductor", "youtube_traductor"),
            ("/youtube-resumen", "youtube_resumen"),
            ("/youtube-a-srt", "youtube_a_srt"),
            ("/privacy", "privacy"),
            ("/terms", "terms"),
            ("/pricing", "pricing"),
            ("/api", "api"),
            ("/extension", "extension"),
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
            "youtube_a_texto" => Some(super::youtube_a_texto::page(req)),
            "youtube_a_mp3" => Some(super::youtube_a_mp3::page(req)),
            "youtube_traductor" => Some(super::youtube_traductor::page(req)),
            "youtube_resumen" => Some(super::youtube_resumen::page(req)),
            "youtube_a_srt" => Some(super::youtube_a_srt::page(req)),
            "privacy" => Some(super::privacy::page(req)),
            "terms" => Some(super::terms::page(req)),
            "pricing" => Some(super::pricing::page(req)),
            "api" => Some(super::api::page(req)),
            "extension" => Some(super::extension::page(req)),
            _ => None,
        }
    }
}
