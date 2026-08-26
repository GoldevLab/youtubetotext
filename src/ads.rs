//! Reserved AdSense slots — empty frames until units are wired.
//!
//! Each `data-ad` value is a stable placement id for a future AdSense unit:
//! header, footer, anchor, home-hero, home-mid, home-faq, home-bottom,
//! workspace-top, workspace-player, workspace-cues, workspace-loading,
//! api-top, api-bottom, extension-mid, extension-bottom, error-mid, notfound-mid.

use resuma::prelude::*;

pub fn slot(placement: &'static str, size: &'static str) -> View {
    let class = format!("ad-slot ad-slot-{size}");
    view! {
        <aside class={class} data-ad={placement} aria-label="Advertisement">
            <div class="ad-slot-frame">
                <span class="ad-slot-label">"Ad"</span>
            </div>
        </aside>
    }
}
