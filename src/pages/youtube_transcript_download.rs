use resuma::prelude::*;

use crate::keyword_landings::DOWNLOAD;
use crate::landing::seo_page;

pub fn page(_req: FlowRequest) -> View {
    seo_page(DOWNLOAD.mode, DOWNLOAD.path, DOWNLOAD.landing)
}
