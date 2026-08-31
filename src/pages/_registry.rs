// Hand-written registry so /v/:id stays dynamic.
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
            ("/app/youtube", "app::youtube"),
            ("/v/:id", "v::id"),
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
            "app::youtube" => Some(super::app::youtube::page(req)),
            "v::id" => Some(super::v::id::page(req)),
            "api" => Some(super::api::page(req)),
            "extension" => Some(super::extension::page(req)),
            _ => None,
        }
    }
}
