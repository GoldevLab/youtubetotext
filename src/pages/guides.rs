use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    crate::guides::index_page()
}
