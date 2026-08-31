use resuma::prelude::*;

/// Old /app/youtube bookmarks use the same `/` page.
pub fn page(req: FlowRequest) -> View {
    crate::pages::index::page(req)
}
