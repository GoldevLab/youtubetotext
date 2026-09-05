pub mod api;
pub mod extension;
pub mod index;
pub mod pricing;
pub mod privacy;
pub mod terms;
pub mod youtube_a_mp3;
pub mod youtube_a_srt;
pub mod youtube_a_texto;
pub mod youtube_resumen;
pub mod youtube_summary;
pub mod youtube_to_audio;
pub mod youtube_to_srt;
pub mod youtube_to_text;
pub mod youtube_traductor;
pub mod youtube_translator;

mod _registry;
pub use _registry::PagesRegistry;
