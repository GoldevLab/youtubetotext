use resuma::prelude::*;
use crate::family::Mode;
use crate::landing::seo_landing;

pub fn page(_req: FlowRequest) -> View {
    seo_landing(Mode::Audio)
}
