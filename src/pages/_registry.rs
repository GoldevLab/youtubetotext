use resuma::prelude::*;
use resuma::FlowPageRegistry;

pub struct PagesRegistry;

const LAYOUT: &[&str] = &["/"];

impl FlowPageRegistry for PagesRegistry {
    fn routes(&self) -> &'static [(&'static str, &'static str)] {
        &[("/", "index")]
    }

    fn layout_for(&self, pattern: &str) -> &'static [&'static str] {
        match pattern {
            _ => LAYOUT,
        }
    }

    fn render(&self, module: &str, req: FlowRequest) -> Option<View> {
        match module {
            "index" => Some(super::index::page(req)),
            _ => None,
        }
    }
}
