use resuma::prelude::*;

use crate::family::{app_href, Mode};

pub fn related(current: Mode, video_id: &str) -> View {
    let vid = video_id.to_string();
    let cards = Mode::all()
        .into_iter()
        .filter(|m| *m != current)
        .map(|m| {
            let href = if vid.is_empty() {
                m.landing_path().to_string()
            } else {
                app_href(&vid, m)
            };
            let label = match m {
                Mode::Text => "Read the transcript",
                Mode::Audio => "Download audio",
                Mode::Translate => "Translate captions",
                Mode::Summary => "Summarize with chapters",
                Mode::Srt => "Download SRT / VTT",
            };
            let extra = if vid.is_empty() {
                "How this job works".to_string()
            } else {
                "Same video — no new link".to_string()
            };
            view! {
                <li>
                    <NavLink href={href} class="related-card">
                        <strong>{label}</strong>
                        <span>{extra}</span>
                    </NavLink>
                </li>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <nav class="cross-sell" aria-label="Related YouTube tools">
            <h2>"Same video — next job"</h2>
            <p class="hint">
                "Audio, translation, summary, and SRT use the transcript you already loaded."
            </p>
            <ul class="related-grid">{cards}</ul>
        </nav>
    }
}

pub fn mode_tabs(active: Mode, video_id: &str) -> View {
    let vid = video_id.to_string();
    if vid.is_empty() {
        return view! { <span class="mode-tabs-skip" hidden=""></span> };
    }
    let links = Mode::all()
        .into_iter()
        .map(|m| {
            let href = app_href(&vid, m);
            let label = m.nav_label();
            if m == active {
                view! {
                    <NavLink href={href} class="mode-tab is-active">{label}</NavLink>
                }
            } else {
                view! {
                    <NavLink href={href} class="mode-tab">{label}</NavLink>
                }
            }
        })
        .collect::<Vec<_>>();
    view! {
        <nav class="mode-tabs" aria-label="Same video">
            {links}
        </nav>
    }
}
