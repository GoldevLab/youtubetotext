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
                    {crate::ads::slot("workspace-top", "infeed")}
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
                <main class="content-section page-pending">
                    <p class="eyebrow">"Fetching captions"</p>
                    <h1>"Loading transcript…"</h1>
                    <p class="hero-lead">"Talking to YouTube. This usually takes a second."</p>
                    <div class="skeleton-stack" aria-hidden="true">
                        <div class="skeleton-block"></div>
                        <div class="skeleton-line w-70"></div>
                        <div class="skeleton-line w-50"></div>
                        <div class="skeleton-line"></div>
                        <div class="skeleton-line w-60"></div>
                    </div>
                    {crate::ads::slot("workspace-loading", "infeed")}
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
            {crate::ads::slot("error-mid", "infeed")}
            <p class="hint" data-rescue-status="" hidden="" role="status"></p>
            {if blocked {
                view! {
                    <div class="rescue">
                        <h2>"This network is blocked — not the video"</h2>
                        <p>
                            "This computer’s IP is rate-limited talking to YouTube. Try again in a bit, or paste the link from another network — no install."
                        </p>
                        <p class="error-actions">
                            <NavLink href={retry.clone()} class="btn btn-primary">"Try the server again"</NavLink>
                            <NavLink href="/" class="btn btn-ghost">"Another video"</NavLink>
                        </p>
                    </div>
                }
            } else {
                view! {
                    <p class="error-actions">
                        <NavLink href={retry} class="btn btn-primary">"Try again"</NavLink>
                        <NavLink href="/" class="btn btn-ghost">"Another video"</NavLink>
                    </p>
                }
            }}
        </main>
    }
}
