use resuma::prelude::*;
use crate::family::Mode;
use crate::landing::seo_landing_es;

pub fn page(_req: FlowRequest) -> View {
    seo_landing_es(Mode::Srt)
}
