use resuma::prelude::*;

use crate::family::Mode;

pub fn related(current: Mode) -> View {
    let cards = Mode::all()
        .into_iter()
        .filter(|m| *m != current)
        .map(|m| {
            let href = m.landing_path().to_string();
            let label = match m {
                Mode::Text => "Read the transcript",
                Mode::Audio => "Download audio file",
                Mode::Translate => "Translate captions",
                Mode::Summary => "Summarize with chapters",
                Mode::Srt => "Download SRT / VTT",
            };
            view! {
                <li>
                    <a href={href} class="related-card" data-r-nav="true">
                        <strong>{label}</strong>
                        <span>"How this job works"</span>
                    </a>
                </li>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <nav class="cross-sell" aria-label="Related YouTube tools">
            <h2>"Other YouTubeForge tools"</h2>
            <p class="hint">
                "Transcript, audio, translation, summary, and SRT are separate jobs — pick the one you need."
            </p>
            <ul class="related-grid">{cards}</ul>
        </nav>
    }
}
