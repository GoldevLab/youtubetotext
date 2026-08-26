// Hand-written: /v/:id is a dynamic segment (do not run `resuma routes --generate`
// or it will flatten this to the static path /v/id).

pub mod api;
pub mod extension;
pub mod index;
pub mod v;

mod _registry;
pub use _registry::PagesRegistry;
