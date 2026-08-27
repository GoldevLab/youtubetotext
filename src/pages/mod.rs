// Hand-written: /v/:id is a dynamic segment (do not run `resuma routes --generate`
// or it will flatten this to the static path /v/id).

pub mod index;
pub mod v;

#[allow(dead_code)]
pub mod api;
#[allow(dead_code)]
pub mod extension;

mod _registry;
pub use _registry::PagesRegistry;
