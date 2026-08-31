// Hand-written: /v/:id is a dynamic segment (do not run `resuma routes --generate`).

pub mod app;
pub mod index;
pub mod v;
pub mod youtube_summary;
pub mod youtube_to_audio;
pub mod youtube_to_srt;
pub mod youtube_to_text;
pub mod youtube_translator;

#[allow(dead_code)]
pub mod api;
#[allow(dead_code)]
pub mod extension;

mod _registry;
pub use _registry::PagesRegistry;
