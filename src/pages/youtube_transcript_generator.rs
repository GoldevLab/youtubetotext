use resuma::prelude::*;

use crate::keyword_landings::GENERATOR;
use crate::landing::seo_page;

pub fn page(_req: FlowRequest) -> View {
    seo_page(GENERATOR.mode, GENERATOR.path, GENERATOR.landing)
}
