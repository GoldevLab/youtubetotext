use resuma::prelude::*;

use crate::keyword_landings::CONVERT;
use crate::landing::seo_page;

pub fn page(_req: FlowRequest) -> View {
    seo_page(CONVERT.mode, CONVERT.path, CONVERT.landing)
}
