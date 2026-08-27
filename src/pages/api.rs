use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <main class="content-section">
            <h1>"YouTube transcript API"</h1>
            <p class="hero-lead">
                "Fetch a public YouTube transcript as JSON, plain text, SRT, VTT, or Markdown. No API key for reasonable use."
            </p>
            {crate::ads::slot("api-top", "infeed")}
            <pre class="code-block"><code>
{r#"curl "https://youtubetotext.fly.dev/api/transcript?v=dQw4w9WgXcQ&fmt=json""#}
            </code></pre>
            <h2>"Query"</h2>
            <div class="table-scroll">
                <table class="api-table">
                    <thead>
                        <tr><th>"Param"</th><th>"What it does"</th></tr>
                    </thead>
                    <tbody>
                        <tr><td><code>"v"</code></td><td>"Video id or YouTube URL"</td></tr>
                        <tr><td><code>"url"</code></td><td>"Same as v (alias)"</td></tr>
                        <tr><td><code>"lang"</code></td><td>"Caption track, e.g. en, es, ja"</td></tr>
                        <tr><td><code>"tlang"</code></td><td>"Translate into this language via YouTube"</td></tr>
                        <tr><td><code>"fmt"</code></td><td>"json (default), txt, timed, srt, vtt, md"</td></tr>
                    </tbody>
                </table>
            </div>
            <h2>"Examples"</h2>
            <pre class="code-block"><code>
{r#"curl "https://youtubetotext.fly.dev/api/transcript?v=dQw4w9WgXcQ&fmt=srt" -o video.srt
curl "https://youtubetotext.fly.dev/api/transcript?url=https://youtu.be/dQw4w9WgXcQ&lang=en&tlang=es&fmt=md""#}
            </code></pre>
            {crate::ads::slot("api-bottom", "rectangle")}
            <p>
                <NavLink href="/" class="btn btn-primary">"Get a transcript"</NavLink>
            </p>
        </main>
    }
}
