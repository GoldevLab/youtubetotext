use resuma::prelude::*;

use crate::workspace::workspace;

pub fn page(req: FlowRequest) -> View {
    let lang = req.query_param("lang").unwrap_or("").to_string();
    let tlang = req.query_param("tlang").unwrap_or("").to_string();
    let video_id = req.param("id").unwrap_or("").to_string();
    let retry_href = format!("/v/{video_id}");
    load_boundary(
        crate::actions::use_video_doc_load(),
        {
            let retry = retry_href.clone();
            let vid = video_id.clone();
            let lang_ok = lang.clone();
            let tlang_ok = tlang.clone();
            let lang_s = lang.clone();
            let tlang_s = tlang.clone();
            move |res| match res {
            Ok(doc) => view! {
                <main class="workspace-main">
                    {workspace(doc, lang_ok, tlang_ok)}
                </main>
            },
            Err(e) => fail_view(e.message, retry, vid, lang_s, tlang_s),
        }},
        {
            let retry = retry_href;
            let vid = video_id;
            move |err| fail_view(err.message, retry, vid, lang, tlang)
        },
        || {
            view! {
                <main class="content-section">
                    <h1>"Loading transcript…"</h1>
                    <p class="hero-lead">"Talking to YouTube."</p>
                </main>
            }
        },
    )
}

fn fail_view(message: String, retry: String, vid: String, lang: String, tlang: String) -> View {
    let blocked = message.to_ascii_lowercase().contains("rate-limit")
        || message.to_ascii_lowercase().contains("could not read");
    view! {
        <main class="content-section" data-fetch-failed="" data-vid={vid} data-lang={lang} data-tlang={tlang}>
            <h1>"Could not load that transcript"</h1>
            <p class="hero-lead">{message}</p>
            <p class="hint" data-rescue-status="" hidden="" role="status"></p>
            {if blocked {
                view! {
                    <div class="rescue">
                        <h2>"This network is blocked — not the video"</h2>
                        <p>
                            "Sites like YouTubeToTranscript work without an extension because "
                            <em>"their server"</em>
                            " talks to YouTube. This computer’s IP is rate-limited. Deploy YouTubeToText (Fly) and paste the link in the browser — no install. The extension is only a backup if cloud IPs get blocked too."
                        </p>
                        <p class="error-actions">
                            <a class="btn btn-primary" href={retry.clone()}>"Try the server again"</a>
                            <a class="btn btn-ghost" href="/extension">"Optional: browser backup"</a>
                            <a class="btn btn-ghost" href="/">"Another video"</a>
                        </p>
                    </div>
                }
            } else {
                view! {
                    <p class="error-actions">
                        <a class="btn btn-primary" href={retry}>"Try again"</a>
                        <a class="btn btn-ghost" href="/">"Another video"</a>
                    </p>
                }
            }}
        </main>
    }
}
