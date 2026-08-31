use resuma::prelude::*;

use crate::family::{app_href, Mode};

pub fn related(current: Mode, video_id: &str) -> View {
    let vid = video_id.to_string();
    let cards = Mode::all()
        .into_iter()
        .filter(|m| *m != current)
        .map(|m| {
            let slug = m.slug();
            let (href, extra, spa) = if vid.is_empty() {
                (
                    m.landing_path().to_string(),
                    "How this job works".to_string(),
                    true,
                )
            } else if m == Mode::Audio {
                (
                    format!("/api/audio?v={vid}"),
                    "Same video — saves the soundtrack".to_string(),
                    false,
                )
            } else {
                (
                    app_href(&vid, m),
                    "Same video — no new link".to_string(),
                    true,
                )
            };
            let label = match m {
                Mode::Text => "Read the transcript",
                Mode::Audio => "Download audio file",
                Mode::Translate => "Translate captions",
                Mode::Summary => "Summarize with chapters",
                Mode::Srt => "Download SRT / VTT",
            };
            if spa {
                view! {
                    <li>
                        <a href={href} class="related-card" data-r-nav="true" data-mode-tab={slug}>
                            <strong>{label}</strong>
                            <span>{extra}</span>
                        </a>
                    </li>
                }
            } else {
                view! {
                    <li>
                        <a href={href} class="related-card" data-mode-tab={slug} rel="noopener">
                            <strong>{label}</strong>
                            <span>{extra}</span>
                        </a>
                    </li>
                }
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
            let href = if m == Mode::Audio {
                format!("/api/audio?v={vid}")
            } else {
                app_href(&vid, m)
            };
            let label = m.nav_label();
            let slug = m.slug();
            let class = if m == active {
                "mode-tab nav-link is-active"
            } else {
                "mode-tab nav-link"
            };
            let spa = m != Mode::Audio;
            if spa {
                view! {
                    <a href={href} class={class} data-r-nav="true" data-mode-tab={slug}>{label}</a>
                }
            } else {
                view! {
                    <a href={href} class={class} data-mode-tab={slug} rel="noopener">{label}</a>
                }
            }
        })
        .collect::<Vec<_>>();
    view! {
        <nav class="mode-tabs" aria-label="Jobs for this video">
            {links}
        </nav>
    }
}
