//! YouTubeForge — YouTube transcripts, audio, translation, summaries, and SRT.

mod actions;
mod ads;
mod api;
mod cross_sell;
mod export;
mod family;
mod guard;
mod landing;
mod landing_es;
mod langs;
mod site;
mod pages;
mod parse;
mod summary;
mod tool;
mod workspace;
mod youtube;

use axum::extract::{Path, Query};
use axum::http::Uri;
use axum::response::Redirect;
use axum::routing::{get, post};
use serde::Deserialize;
use pages::PagesRegistry;
use resuma::prelude::*;
use resuma::SeoKit;
use serde_json::json;

fn view_transition_name(path: &str) -> String {
    let slug = path.trim_matches('/');
    if slug.is_empty() {
        "home".into()
    } else {
        slug.replace('/', "-")
    }
}

fn chrome(body: View) -> View {
    let vt = view_transition_name(
        &current_request()
            .map(|r| r.path)
            .unwrap_or_else(|| "/".into()),
    );
    view! {
        <div class="app">
            <div class="liquid-orbs" aria-hidden="true">
                <div class="liquid-blob liquid-blob-a"></div>
                <div class="liquid-blob liquid-blob-b"></div>
                <div class="liquid-blob liquid-blob-c"></div>
            </div>
            <header class="site-header">
                <div class="header-inner">
                    <NavLink href="/" class="brand" activeClass="is-active" exact=true>
                        <span class="brand-mark" aria-hidden="true">"Yf"</span>
                        <span class="brand-name">"YouTubeForge"</span>
                    </NavLink>
                    <span class="nav-progress" aria-hidden="true"></span>
                </div>
            </header>
            {with_view_transition(vt, vec![Child::View(body)])}
            <footer class="site-footer">
                {crate::cross_sell::seo_footer_links()}
                {crate::cross_sell::sister_apps_links()}
                <p>
                    <strong>"YouTubeForge"</strong>
                    " — YouTube transcripts, audio, translation, summaries, and SRT. Not affiliated with YouTube or Google."
                </p>
            </footer>
        </div>
    }
}

#[layout("/")]
fn RootLayout() -> View {
    visible_task!(
        r##"
        async (_state, __resuma) => {
            const onNav = () => {
                document.documentElement.classList.remove("is-navigating");
                const heading = document.querySelector("[data-r-vt] h1");
                if (!heading) return;
                heading.setAttribute("tabindex", "-1");
                heading.focus({ preventScroll: true });
            };
            const onClick = (e) => {
                const el = e.target instanceof Element ? e.target.closest("a[data-r-nav]") : null;
                if (el?.getAttribute("href")) {
                    document.documentElement.classList.add("is-navigating");
                }
            };
            document.addEventListener("resuma:navigate", onNav);
            document.addEventListener("click", onClick, true);
            if (!window.__yttVtGuard) {
                window.__yttVtGuard = true;
                window.addEventListener("unhandledrejection", (e) => {
                    const err = e.reason;
                    const name = err && err.name;
                    const msg = err && (err.message || String(err)) || "";
                    if (name === "AbortError" && /Transition was skipped|ViewTransition/i.test(msg)) {
                        e.preventDefault();
                    }
                });
            }
            if (!window.__yttCopyBound) {
                window.__yttCopyBound = true;
                const clock = (ms) => {
                    const s = Math.floor(ms / 1000);
                    const h = Math.floor(s / 3600);
                    const m = Math.floor((s % 3600) / 60);
                    const sec = s % 60;
                    return h ? h + ":" + String(m).padStart(2,"0") + ":" + String(sec).padStart(2,"0")
                             : m + ":" + String(sec).padStart(2,"0");
                };
                const execCopy = (body) => {
                    const ta = document.createElement("textarea");
                    ta.value = body;
                    ta.setAttribute("readonly", "");
                    ta.style.cssText = "position:fixed;top:0;left:0;width:1px;height:1px;opacity:0";
                    document.body.appendChild(ta);
                    ta.focus();
                    ta.select();
                    ta.setSelectionRange(0, ta.value.length);
                    let ok = false;
                    try { ok = document.execCommand("copy"); } catch (_) {}
                    ta.remove();
                    return ok;
                };
                document.addEventListener("click", (e) => {
                    const t = e.target instanceof Element ? e.target : e.target && e.target.parentElement;
                    if (!t || !t.closest) return;
                    const copyBtn = t.closest("[data-copy]");
                    const mdBtn = t.closest("[data-copy-md]");
                    if (!copyBtn && !mdBtn) return;
                    const ws = t.closest("#ytt-ws") || document.querySelector("#resuma-root #ytt-ws");
                    if (!ws) return;
                    e.preventDefault();
                    const md = !!mdBtn;
                    const btn = mdBtn || copyBtn;
                    const q = (ws.querySelector("[data-search]") && ws.querySelector("[data-search]").value || "").trim().toLowerCase();
                    const from = Math.max(0, Number((ws.querySelector("[data-from]") || {}).value || 0) * 1000);
                    const toRaw = Number((ws.querySelector("[data-to]") || {}).value || 0);
                    const to = toRaw > 0 ? toRaw * 1000 : Infinity;
                    const cues = [];
                    ws.querySelectorAll(".cue").forEach((el) => {
                        if (el.hidden) return;
                        const ms = Number(el.dataset.ms || 0);
                        if (ms < from || ms > to) return;
                        const text = (el.querySelector("[data-cue-text]") && el.querySelector("[data-cue-text]").textContent) || el.dataset.text || "";
                        if (q && !String(text).toLowerCase().includes(q)) return;
                        cues.push({ start_ms: ms, text: text });
                    });
                    const status = ws.querySelector("[data-status]");
                    const setStatus = (msg, wait) => {
                        if (!status) return;
                        status.textContent = msg || "";
                        status.hidden = !msg;
                        if (msg) setTimeout(() => { if (status.textContent === msg) { status.textContent = ""; status.hidden = true; } }, wait || 1800);
                    };
                    if (!cues.length) {
                        setStatus("Nothing to copy - clear the search or widen the time range.", 2200);
                        return;
                    }
                    const id = ws.dataset.vid || "";
                    const stamps = ws.querySelector("[data-stamps]") && ws.querySelector("[data-stamps]").checked;
                    let body = "";
                    if (md) {
                        body = cues.map((c) => "- [" + clock(c.start_ms) + "](https://www.youtube.com/watch?v=" + id + "&t=" + Math.floor(c.start_ms/1000) + "s) " + c.text).join("\n");
                    } else if (stamps) {
                        body = cues.map((c) => "[" + clock(c.start_ms) + "] " + c.text).join("\n");
                    } else {
                        body = cues.map((c) => c.text).join("\n");
                    }
                    const done = () => {
                        const label = btn.querySelector("[data-copy-label]");
                        const idle = md ? "Copy Markdown" : "Copy transcript";
                        btn.dataset.copied = "1";
                        btn.classList.remove("is-press");
                        void btn.offsetWidth;
                        btn.classList.add("is-press");
                        if (label) label.textContent = "Copied!";
                        else btn.textContent = "Copied!";
                        clearTimeout(btn._yttCopyTimer);
                        btn._yttCopyTimer = setTimeout(() => {
                            btn.dataset.copied = "0";
                            btn.classList.remove("is-press");
                            if (label) label.textContent = idle;
                            else btn.textContent = idle;
                        }, 1800);
                        setStatus("Transcript copied to the clipboard.", 1800);
                    };
                    if (execCopy(body)) done();
                    else if (navigator.clipboard && navigator.clipboard.writeText) {
                        navigator.clipboard.writeText(body).then(done).catch(() => {
                            setStatus("Could not copy - select the transcript and copy manually.", 2200);
                        });
                    } else {
                        setStatus("Could not copy - select the transcript and copy manually.", 2200);
                    }
                }, true);
            }
            return () => {
                document.removeEventListener("resuma:navigate", onNav);
                document.removeEventListener("click", onClick, true);
            };
        }
    "##
    );

    chrome(view! { <Slot /> })
}

fn not_found() -> View {
    chrome(view! {
        <main class="content-section">
            <h1>"Page not found"</h1>
            <p class="hero-lead">"That page does not exist on YouTubeForge."</p>
            <p>
                <NavLink href="/" class="btn btn-primary">"Back to home"</NavLink>
            </p>
        </main>
    })
}

#[derive(Debug, Default, Deserialize)]
struct LegacyVidQuery {
    #[serde(default)]
    mode: String,
    #[serde(default)]
    lang: String,
    #[serde(default)]
    tlang: String,
}

async fn redirect_app_youtube(uri: Uri) -> Redirect {
    match uri.query() {
        Some(q) if !q.is_empty() => Redirect::permanent(&format!("/?{q}")),
        _ => Redirect::permanent("/"),
    }
}

async fn redirect_video(Path(id): Path<String>, Query(q): Query<LegacyVidQuery>) -> Redirect {
    let mode = if q.mode.trim().is_empty() {
        "text"
    } else {
        q.mode.trim()
    };
    let mut url = format!("/?v={id}&mode={mode}");
    if !q.lang.trim().is_empty() {
        url.push_str("&lang=");
        url.push_str(q.lang.trim());
    }
    if !q.tlang.trim().is_empty() {
        url.push_str("&tlang=");
        url.push_str(q.tlang.trim());
    }
    Redirect::permanent(&url)
}

const HEAD: &str = r##"
<link rel="preconnect" href="https://fonts.googleapis.com" />
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
<link href="https://fonts.googleapis.com/css2?family=Roboto:ital,wght@0,400;0,500;0,700;0,900;1,400&display=swap" rel="stylesheet" />
<link rel="apple-touch-icon" href="/icons/apple-touch-icon.png" sizes="180x180" />
<script type="module" src="/js/youtubetotext.js?v=1"></script>
"##;

fn seo_kit() -> SeoKit {
    let origin = crate::family::public_origin();
    let mut kit = SeoKit::new("YouTubeForge", &origin)
        .with_locale("en_US")
        .with_keywords(
            "YouTube transcript, YouTube to text, download YouTube captions, \
             YouTube SRT, YouTube VTT, YouTube to audio, free transcript, caption translator",
        )
        .with_llms_summary(
            "YouTubeForge turns a YouTube URL into a searchable, downloadable transcript. \
             Copy as text or Markdown, export SRT/VTT/JSON, translate captions, download audio, \
             and trim sections. No account.",
        )
        .with_default_json_ld()
        .push_json_ld(crate::landing::web_application_json_ld())
        .push_json_ld(json!({
            "@context": "https://schema.org",
            "@type": "FAQPage",
            "mainEntity": [
                {
                    "@type": "Question",
                    "name": "Is YouTubeForge free to use?",
                    "acceptedAnswer": {
                        "@type": "Answer",
                        "text": "Yes. No account, no sign-up. Public YouTube captions are extracted as-is."
                    }
                },
                {
                    "@type": "Question",
                    "name": "How do I access the transcript after generating it?",
                    "acceptedAnswer": {
                        "@type": "Answer",
                        "text": "Every transcript has a shareable URL at /?v={videoId}&mode=text. Optional query params: lang, tlang, and mode (audio, translate, summary, srt)."
                    }
                },
                {
                    "@type": "Question",
                    "name": "Can I download the transcript?",
                    "acceptedAnswer": {
                        "@type": "Answer",
                        "text": "Yes. Download TXT, SRT, VTT, Markdown with timestamp links, or JSON in one click. Copy and Copy Markdown are also available."
                    }
                },
                {
                    "@type": "Question",
                    "name": "Is there a limit to the length of the video?",
                    "acceptedAnswer": {
                        "@type": "Answer",
                        "text": "If YouTube has captions for a public video, YouTubeForge can load them. There is no extra length cap on our side."
                    }
                }
            ]
        }));
    kit.theme_color = Some("#14090a".into());
    kit.author = "YouTubeForge".into();
    kit.llms_sections = vec![
        (
            "How to use".into(),
            "Open / and paste a YouTube URL. The result is /?v={videoId}&mode=text (noindex). Optional query: lang, tlang, mode.".into(),
        ),
        (
            "SEO landings".into(),
            "/youtube-to-text and Spanish /youtube-a-texto (audio, traductor, resumen, srt). /privacy /terms /pricing /api /extension.".into(),
        ),
    ];
    kit.ai.disallow = vec!["/api/".into()];
    kit
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // `with_seo_kit` owns keywords/author/theme-color meta, JSON-LD, and the
    // `/robots.txt` + `/llms.txt` routes (AI crawler policy included).
    let head = format!("{HEAD}{}{}", ads::head_snippet(), crate::site::head_extras());
    let ads_txt = ads::ads_txt().map(|s| -> &'static [u8] {
        Box::leak(s.into_bytes().into_boxed_slice())
    });
    let public = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("public");

    let mut serve = FlowServeOptions::default();
    ads::apply_csp(&mut serve.security.csp);

    let mut app = FlowApp::new()
        .with_title("YouTube transcript, audio, SRT and translation | YouTubeForge")
        .with_description(
            "Get a free YouTube transcript from any public video. Search, copy, download SRT/VTT/Markdown, translate captions. No cookie wall, no account.",
        )
        .with_site_url(crate::family::public_origin())
        .with_og_image("/og.svg")
        .with_head(head)
        .with_seo_kit(seo_kit())
        .with_html_theme(
            HtmlTheme::new(["studio"])
                .dark(["studio"])
                .cookie("ytt_theme")
                .storage_key("ytt-theme"),
        )
        .with_stylesheet("/css/youtubetotext.css");
    if let Some(body) = ads_txt {
        app = app.static_asset("/ads.txt", body, "text/plain; charset=utf-8");
    }
    app.with_public_dir(public)
        .with_pwa(FlowPwaConfig {
            name: "YouTubeForge".into(),
            short_name: "Yf".into(),
            description: "YouTube transcripts, audio, SRT, translation, and summaries.".into(),
            theme_color: "#14090a".into(),
            background_color: "#14090a".into(),
            start_url: "/".into(),
            scope: "/".into(),
            cache_version: "yf-8".into(),
            display: "standalone".into(),
            orientation: "any".into(),
            lang: "en".into(),
            icon_char: Some("Y".into()),
            precache_paths: vec![
                "/themes.css".into(),
                "/css/youtubetotext.css".into(),
                "/js/youtubetotext.js?v=1".into(),
            ],
            shortcuts: vec![PwaShortcut {
                name: "New transcript".into(),
                short_name: "Home".into(),
                url: "/".into(),
            }],
            offline_title: "You're offline".into(),
            offline_message: "YouTubeForge needs a connection to fetch captions and downloads. Reconnect and try again.".into(),
            manifest_icons: Vec::new(),
        })
        .route("/app/youtube", get(redirect_app_youtube))
        .route("/v/{id}", get(redirect_video))
        .route("/api/transcript", get(api::transcript).options(api::preflight))
        .route("/api/audio", get(api::audio).options(api::preflight))
        .route("/api/video", get(api::video).options(api::preflight))
        .route("/api/ingest", post(api::ingest).options(api::preflight))
        .route("/api/translate", post(api::translate).options(api::preflight))
        .route("/api/gate", post(api::gate).options(api::preflight))
        .not_found(not_found)
        .auto_pages(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/pages"),
            PagesRegistry,
        )
        .serve(serve)
        .await
}
