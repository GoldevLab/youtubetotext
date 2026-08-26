//! YouTubeToText — free YouTube transcripts in pure Rust (Resuma Flow).

mod actions;
mod api;
mod export;
mod langs;
mod pages;
mod parse;
mod tool;
mod workspace;
mod youtube;

use axum::routing::{get, post};
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
            <header class="site-header">
                <div class="header-inner">
                    <NavLink href="/" class="brand" activeClass="is-active" exact=true>
                        <span class="brand-mark" aria-hidden="true"></span>
                        <span class="brand-name">"YouTubeToText"</span>
                    </NavLink>
                    <nav class="nav" data-r-nav-exclusive="">
                        <NavLink href="/" class="nav-link" activeClass="is-active" exact=true>"Transcript"</NavLink>
                        <NavLink href="/extension" class="nav-link" activeClass="is-active" exact=true>"Extension"</NavLink>
                        <NavLink href="/api" class="nav-link" activeClass="is-active" exact=true>"API"</NavLink>
                    </nav>
                    <span class="nav-progress" aria-hidden="true"></span>
                </div>
            </header>
            {with_view_transition(vt, vec![Child::View(body)])}
            <footer class="site-footer">
                <p>
                    <strong>"YouTubeToText"</strong>
                    " — free YouTube transcripts, no sign-up. Not affiliated with YouTube or Google."
                </p>
            </footer>
        </div>
    }
}

#[layout("/")]
fn RootLayout() -> View {
    visible_task!(
        r#"
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
            return () => {
                document.removeEventListener("resuma:navigate", onNav);
                document.removeEventListener("click", onClick, true);
            };
        }
    "#
    );

    chrome(view! { <Slot /> })
}

fn not_found() -> View {
    chrome(view! {
        <main class="content-section">
            <h1>"Page not found"</h1>
            <p class="hero-lead">"That path does not exist on YouTubeToText."</p>
            <p>
                <NavLink href="/" class="btn btn-primary">"Back to home"</NavLink>
            </p>
        </main>
    })
}

const HEAD: &str = r##"
<link rel="preconnect" href="https://fonts.googleapis.com" />
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
<link href="https://fonts.googleapis.com/css2?family=Fraunces:opsz,wght@9..144,600;9..144,700;9..144,800&family=Source+Sans+3:ital,wght@0,400;0,500;0,600;0,700;1,400&display=swap" rel="stylesheet" />
<meta property="og:title" content="YouTube Transcript — Free YouTube to Text, SRT & VTT | YouTubeToText" />
<meta property="og:description" content="Paste a YouTube link. Get a searchable transcript. Copy, download SRT/VTT/Markdown, translate. No ads, no account." />
<meta property="og:type" content="website" />
"##;

fn seo_kit() -> SeoKit {
    let mut kit = SeoKit::new("YouTubeToText", "https://youtubetotext.fly.dev")
        .with_locale("en_US")
        .with_keywords(
            "YouTube transcript, YouTube to transcript, YouTube to text, download YouTube captions, \
             YouTube SRT, YouTube VTT, free transcript, video transcript, caption translator",
        )
        .with_llms_summary(
            "YouTubeToText turns a YouTube URL into a searchable, downloadable transcript. \
             Copy as text or Markdown, export SRT/VTT/JSON, translate captions, trim \
             sections, and fetch the same data from a free HTTP API. No account.",
        )
        .with_default_json_ld()
        .push_json_ld(json!({
            "@context": "https://schema.org",
            "@type": "WebApplication",
            "name": "YouTubeToText",
            "alternateName": ["YouTube transcript", "YouTube to text"],
            "url": "https://youtubetotext.fly.dev",
            "applicationCategory": "UtilitiesApplication",
            "operatingSystem": "Web",
            "offers": {"@type": "Offer", "price": "0", "priceCurrency": "USD"},
            "description": "Free YouTube transcript tool: search, download SRT/VTT/Markdown, translate captions. No ads."
        }));
    kit.theme_color = Some("#c45c26".into());
    kit.author = "YouTubeToText".into();
    kit.llms_sections = vec![
        (
            "How to use".into(),
            "Open / with a YouTube URL, or go to /v/{videoId}. Optional query: lang, tlang.".into(),
        ),
        (
            "API".into(),
            "GET /api/transcript?v=VIDEO_ID&fmt=json|txt|srt|vtt|md&lang=&tlang=".into(),
        ),
    ];
    kit
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let kit = seo_kit();
    let head = format!("{HEAD}{}", kit.head_extras());
    let json_ld = serde_json::to_string(&kit.json_ld_blocks).unwrap_or_else(|_| "[]".into());
    let llms: &'static [u8] = Box::leak(kit.llms_txt().into_bytes().into_boxed_slice());
    let public = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("public");

    FlowApp::new()
        .with_title("YouTube Transcript — Free YouTube to Text, SRT & VTT | YouTubeToText")
        .with_description(
            "Get a free YouTube transcript from any public video. Search, copy, download SRT/VTT/Markdown, translate captions. No ads, no account.",
        )
        .with_site_url("https://youtubetotext.fly.dev")
        .with_og_image("/og.svg")
        .with_json_ld(json_ld)
        .with_head(head)
        .with_stylesheet("/css/youtubetotext.css")
        .static_asset("/llms.txt", llms, "text/plain; charset=utf-8")
        .with_public_dir(public)
        .without_pwa()
        .route("/api/transcript", get(api::transcript).options(api::preflight))
        .route("/api/ingest", post(api::ingest).options(api::preflight))
        .not_found(not_found)
        .auto_pages(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/pages"),
            PagesRegistry,
        )
        .serve(FlowServeOptions::default())
        .await
}
