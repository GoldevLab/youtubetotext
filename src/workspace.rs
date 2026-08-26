//! Video workspace — player, search, copy, downloads, cue list.

use resuma::prelude::*;
use serde::Serialize;

use crate::export::{format_clock, reading_minutes};
use crate::youtube::{track_key, Cue, TranscriptDoc};

#[derive(Serialize)]
struct ClientCue {
    start_ms: u64,
    duration_ms: u64,
    text: String,
}

#[derive(Serialize)]
struct ClientDoc {
    video_id: String,
    title: String,
    author: String,
    cues: Vec<ClientCue>,
}

#[island]
pub fn workspace(doc: TranscriptDoc, lang: String, tlang: String) -> View {
    let video_id = doc.video_id.clone();
    let title = doc.title.clone();
    let author = doc.author.clone();
    let duration = format_clock(doc.duration_secs.saturating_mul(1000), false);
    let words = doc.word_count();
    let read_mins = reading_minutes(&doc.plain_text());
    let track_label = doc.track.name.clone();
    let is_auto = doc.track.kind == "asr";
    let thumb = format!("https://i.ytimg.com/vi/{video_id}/hqdefault.jpg");
    let watch = format!("https://www.youtube.com/watch?v={video_id}");
    let action = format!("/v/{video_id}");
    let cue_count = doc.cues.len();
    let payload = ClientDoc {
        video_id: video_id.clone(),
        title: title.clone(),
        author: author.clone(),
        cues: doc
            .cues
            .iter()
            .map(|c| ClientCue {
                start_ms: c.start_ms,
                duration_ms: c.duration_ms,
                text: c.text.clone(),
            })
            .collect(),
    };
    let json = serde_json::to_string(&payload)
        .unwrap_or_else(|_| "{}".into())
        .replace('<', "\\u003c")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029");
    let cues_html = render_cues(&video_id, &doc.cues);
    let chapters_html = render_chapters(&video_id, &doc.chapters);
    let track_options = doc
        .tracks
        .iter()
        .map(|t| {
            let selected = t.lang == doc.track.lang && t.kind == doc.track.kind;
            let value = track_key(&t.lang, &t.kind);
            let label = t.name.clone();
            if selected {
                view! { <option value={value} selected=true>{label}</option> }
            } else {
                view! { <option value={value}>{label}</option> }
            }
        })
        .collect::<Vec<_>>();
    let tlang_options = {
        let mut opts = vec![view! { <option value="">"Original (no translation)"</option> }];
        for lang_opt in &doc.translations {
            let code = lang_opt.code.clone();
            let name = lang_opt.name.clone();
            if !tlang.is_empty() && tlang == code {
                opts.push(view! { <option value={code} selected=true>{name}</option> });
            } else {
                opts.push(view! { <option value={code}>{name}</option> });
            }
        }
        opts
    };
    let kind_note = if is_auto { "Auto-captions" } else { "Human captions" };
    let duration_attr = doc.duration_secs.to_string();
    let max_trim = if doc.duration_secs > 0 {
        duration_attr.clone()
    } else {
        String::new()
    };
    let thumb_alt = if title.is_empty() {
        "Video thumbnail".to_string()
    } else {
        title.clone()
    };
    let stats = if words == 0 {
        format!("{cue_count} lines · {duration}")
    } else {
        format!("{cue_count} lines · {words} words · ~{read_mins} min read · {duration}")
    };

    visible_task!(
        r##"
(async (state, __resuma) => {
    const root = document.getElementById("ytt-ws");
    if (!root || root.dataset.ready === "1") return;
    root.dataset.ready = "1";
    const dataEl = document.getElementById("ytt-data");
    let data = { video_id: root.dataset.vid, title: "", author: "", cues: [] };
    try { data = JSON.parse(dataEl?.textContent || "{}"); } catch (_) {}
    try {
        const KEY = "youtubetotext-recent";
        const cur = JSON.parse(localStorage.getItem(KEY) || "[]");
        const next = [{ id: data.video_id, title: data.title || data.video_id, at: Date.now() }, ...cur.filter((x) => x.id !== data.video_id)].slice(0, 8);
        localStorage.setItem(KEY, JSON.stringify(next));
    } catch (_) {}
    const list = root.querySelector("[data-cues]");
    const search = root.querySelector("[data-search]");
    const countEl = root.querySelector("[data-count]");
    const status = root.querySelector("[data-status]");
    const fromInp = root.querySelector("[data-from]");
    const toInp = root.querySelector("[data-to]");
    const stamps = root.querySelector("[data-stamps]");
    let player = null;
    let pendingMs = 0;
    let ytApi = null;

    const setStatus = (t) => { if (status) { status.textContent = t || ""; status.hidden = !t; } };
    const fmtClock = (ms) => {
        const s = Math.floor(ms / 1000);
        const h = Math.floor(s / 3600);
        const m = Math.floor((s % 3600) / 60);
        const sec = s % 60;
        return h ? `${h}:${String(m).padStart(2,"0")}:${String(sec).padStart(2,"0")}`
                 : `${m}:${String(sec).padStart(2,"0")}`;
    };
    const durationMs = () => {
        const secs = Number(root.dataset.duration || 0);
        if (secs > 0) return secs * 1000;
        return (data.cues.at(-1)?.start_ms || 0) + 5000;
    };
    const rangeMs = () => {
        const max = durationMs();
        const from = Math.max(0, Number(fromInp?.value || 0) * 1000);
        const toRaw = Number(toInp?.value || 0);
        const to = toRaw > 0 ? Math.min(toRaw * 1000, max) : max;
        return { from, to };
    };
    const visibleCues = () => {
        const q = (search?.value || "").trim().toLowerCase();
        const { from, to } = rangeMs();
        return data.cues.filter((c) => {
            if (c.start_ms < from || c.start_ms > to) return false;
            if (q && !c.text.toLowerCase().includes(q)) return false;
            return true;
        });
    };
    const applyFilters = () => {
        const q = (search?.value || "").trim().toLowerCase();
        const { from, to } = rangeMs();
        let shown = 0;
        list?.querySelectorAll(".cue").forEach((el) => {
            const ms = Number(el.dataset.ms || 0);
            const text = (el.dataset.text || el.textContent || "").toLowerCase();
            const hide = ms < from || ms > to || (q && !text.includes(q));
            el.hidden = hide;
            el.classList.toggle("is-hit", !hide && !!q);
            if (!hide) shown++;
        });
        if (countEl) countEl.textContent = q || (fromInp?.value || toInp?.value)
            ? `${shown} shown`
            : "";
    };
    const copyText = async (mode) => {
        const cues = visibleCues();
        if (!cues.length) {
            setStatus("Nothing to copy — clear the search or widen the time range.");
            setTimeout(() => setStatus(""), 2200);
            return;
        }
        const id = data.video_id;
        let body = "";
        if (mode === "md") {
            body = cues.map((c) => `- [${fmtClock(c.start_ms)}](https://www.youtube.com/watch?v=${id}&t=${Math.floor(c.start_ms/1000)}s) ${c.text}`).join("\n");
        } else if (mode === "timed" || stamps?.checked) {
            body = cues.map((c) => `[${fmtClock(c.start_ms)}] ${c.text}`).join("\n");
        } else {
            body = cues.map((c) => c.text).join("\n");
        }
        try {
            await navigator.clipboard.writeText(body);
            setStatus("Copied.");
        } catch {
            setStatus("Could not copy — select the transcript and copy manually.");
        }
        setTimeout(() => setStatus(""), 1800);
    };
    const download = (fmt) => {
        const cues = visibleCues();
        if (!cues.length) {
            setStatus("Nothing to download — clear the search or widen the time range.");
            setTimeout(() => setStatus(""), 2200);
            return;
        }
        const id = data.video_id;
        const srtClock = (ms) => {
            const h = Math.floor(ms/3600000), m = Math.floor(ms/60000)%60, s = Math.floor(ms/1000)%60, f = ms%1000;
            return `${String(h).padStart(2,"0")}:${String(m).padStart(2,"0")}:${String(s).padStart(2,"0")},${String(f).padStart(3,"0")}`;
        };
        let name = `${id}.txt`, mime = "text/plain", body = "";
        if (fmt === "srt") {
            name = `${id}.srt`; mime = "application/x-subrip";
            body = cues.map((c,i) => `${i+1}\n${srtClock(c.start_ms)} --> ${srtClock(c.start_ms + Math.max(c.duration_ms,500))}\n${c.text}\n`).join("\n");
        } else if (fmt === "vtt") {
            name = `${id}.vtt`; mime = "text/vtt";
            body = "WEBVTT\n\n" + cues.map((c) => `${srtClock(c.start_ms).replace(",",".")} --> ${srtClock(c.start_ms + Math.max(c.duration_ms,500)).replace(",",".")}\n${c.text}\n`).join("\n");
        } else if (fmt === "md") {
            name = `${id}.md`; mime = "text/markdown";
            body = `# ${data.title || "Transcript"}\n\n` + cues.map((c) => `- [${fmtClock(c.start_ms)}](https://www.youtube.com/watch?v=${id}&t=${Math.floor(c.start_ms/1000)}s) ${c.text}`).join("\n");
        } else if (fmt === "json") {
            name = `${id}.json`; mime = "application/json";
            body = JSON.stringify({ video_id: id, title: data.title, cues }, null, 2);
        } else {
            body = (stamps?.checked ? cues.map((c) => `[${fmtClock(c.start_ms)}] ${c.text}`) : cues.map((c) => c.text)).join("\n");
        }
        const blob = new Blob([body], { type: mime });
        const a = document.createElement("a");
        a.href = URL.createObjectURL(blob);
        a.download = name;
        a.rel = "noopener";
        a.style.display = "none";
        document.body.append(a);
        a.click();
        a.remove();
        setTimeout(() => URL.revokeObjectURL(a.href), 1500);
    };
    let playerP = null;
    const loadApi = () => new Promise((resolve) => {
        if (window.YT?.Player) return resolve(window.YT);
        const prev = window.onYouTubeIframeAPIReady;
        window.onYouTubeIframeAPIReady = () => { prev?.(); resolve(window.YT); };
        if (!document.querySelector('script[src*="iframe_api"]')) {
            const s = document.createElement("script");
            s.src = "https://www.youtube.com/iframe_api";
            document.head.appendChild(s);
        }
        setTimeout(() => resolve(window.YT), 8000);
    });
    const mountPlayer = async (startMs) => {
        const host = root.querySelector("[data-player]");
        if (!host) return;
        pendingMs = startMs || 0;
        if (player) {
            try { player.seekTo(pendingMs / 1000, true); player.playVideo?.(); } catch (_) {}
            return;
        }
        if (playerP) {
            await playerP;
            try { player?.seekTo(pendingMs / 1000, true); player?.playVideo?.(); } catch (_) {}
            return;
        }
        playerP = (async () => {
            host.innerHTML = `<div id="yt-frame"></div>`;
            ytApi = await loadApi();
            if (!ytApi?.Player) {
                window.location.href = `https://www.youtube.com/watch?v=${data.video_id}&t=${Math.floor(pendingMs/1000)}s`;
                return;
            }
            player = new ytApi.Player("yt-frame", {
                videoId: data.video_id,
                host: "https://www.youtube-nocookie.com",
                playerVars: { rel: 0, modestbranding: 1, start: Math.floor(pendingMs/1000), origin: location.origin },
                events: {
                    onReady: (e) => { try { e.target.seekTo(pendingMs/1000, true); e.target.playVideo(); } catch (_) {} }
                }
            });
            let last = "";
            setInterval(() => {
                let t = 0;
                try { t = player.getCurrentTime() * 1000; } catch (_) { return; }
                let current = null;
                for (const c of data.cues) {
                    if (c.start_ms <= t) current = c;
                    else break;
                }
                const key = current ? String(current.start_ms) : "";
                if (key === last) return;
                last = key;
                list?.querySelectorAll(".cue.is-current").forEach((el) => el.classList.remove("is-current"));
                const el = list?.querySelector(`.cue[data-ms="${key}"]`);
                if (el && list) {
                    el.classList.add("is-current");
                    const r = list.getBoundingClientRect();
                    const er = el.getBoundingClientRect();
                    if (er.top < r.top + 40 || er.bottom > r.bottom - 40) {
                        el.scrollIntoView({ block: "nearest" });
                    }
                }
            }, 250);
        })();
        await playerP;
    };
    root.querySelector("[data-play]")?.addEventListener("click", () => mountPlayer(0));
    list?.addEventListener("click", (e) => {
        const a = e.target.closest("a.cue");
        if (!a) return;
        e.preventDefault();
        mountPlayer(Number(a.dataset.ms || 0));
    });
    root.querySelector("[data-chapters]")?.addEventListener("click", (e) => {
        const a = e.target.closest("a[data-ms]");
        if (!a) return;
        e.preventDefault();
        mountPlayer(Number(a.dataset.ms || 0));
    });
    search?.addEventListener("input", applyFilters);
    fromInp?.addEventListener("input", applyFilters);
    toInp?.addEventListener("input", applyFilters);
    root.querySelector("[data-copy]")?.addEventListener("click", () => copyText(stamps?.checked ? "timed" : "plain"));
    root.querySelector("[data-copy-md]")?.addEventListener("click", () => copyText("md"));
    root.querySelectorAll("[data-dl]").forEach((btn) => {
        btn.addEventListener("click", () => download(btn.getAttribute("data-dl")));
    });
    root.querySelectorAll("[data-prompt]").forEach((btn) => {
        btn.addEventListener("click", async () => {
            const kind = btn.getAttribute("data-prompt");
            const text = visibleCues().map((c) => c.text).join("\n");
            if (!text.trim()) {
                setStatus("Nothing to copy — clear the search or widen the time range.");
                setTimeout(() => setStatus(""), 2200);
                return;
            }
            const prompts = {
                summary: "Summarize this YouTube transcript in 8 tight bullets. Keep names, numbers, and claims. Then give a 2-sentence overview.\n\n",
                notes: "Turn this transcript into structured study notes with headings, key points, and a short glossary of terms.\n\n",
                quiz: "Create 8 mixed quiz questions (multiple choice + short answer) from this transcript, with an answer key at the end.\n\n",
                quotes: "Extract the strongest quotes from this transcript. For each, add a one-line why-it-matters.\n\n",
            };
            const body = (prompts[kind] || prompts.summary) + text;
            try {
                await navigator.clipboard.writeText(body);
                setStatus("Prompt copied — paste it into any AI chat.");
            } catch { setStatus("Could not copy the prompt."); }
            setTimeout(() => setStatus(""), 2200);
        });
    });
    document.addEventListener("keydown", (e) => {
        const tag = document.activeElement?.tagName;
        if (e.key === "/" && tag !== "INPUT" && tag !== "TEXTAREA" && tag !== "SELECT") {
            e.preventDefault();
            search?.focus();
        }
        if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
            e.preventDefault();
            search?.focus();
        }
    });
})
"##
    );

    view! {
        <div id="ytt-ws" class="workspace" data-vid={video_id.clone()} data-lang={lang} data-duration={duration_attr}>
            {View::raw(format!(
                r#"<script type="application/json" id="ytt-data">{json}</script>"#
            ))}
            <aside class="player-col">
                <div class="player-card">
                    <div class="player-facade" data-player="">
                        <button type="button" class="play-hit" data-play="" aria-label="Play video">
                            <img src={thumb} alt={thumb_alt} width="480" height="360" />
                            <span class="play-badge">"Play"</span>
                        </button>
                    </div>
                    <h1 class="vid-title">{title}</h1>
                    <p class="vid-meta">
                        <span>{author}</span>
                        <span aria-hidden="true">" · "</span>
                        <span>{track_label}</span>
                        <span aria-hidden="true">" · "</span>
                        <span>{kind_note}</span>
                    </p>
                    <p class="vid-stats">{stats}</p>
                    <p>
                        <a class="btn btn-ghost" href={watch} target="_blank" rel="noreferrer noopener">"Open on YouTube"</a>
                    </p>
                    {View::raw(chapters_html)}
                </div>
            </aside>
            <section class="transcript-col">
                <form class="toolbar toolbar-lang" method="get" action={action}>
                    <label>
                        "Captions"
                        <select name="lang">{track_options}</select>
                    </label>
                    <label>
                        "Translate"
                        <select name="tlang">{tlang_options}</select>
                    </label>
                    <button type="submit" class="btn btn-secondary">"Apply"</button>
                </form>
                <div class="toolbar">
                    <label class="grow">
                        "Search"
                        <input type="search" data-search="" placeholder="Filter lines  (press /)" autocomplete="off" />
                    </label>
                    <span class="match-count" data-count=""></span>
                </div>
                <div class="toolbar toolbar-actions">
                    <label class="check">
                        <input type="checkbox" data-stamps="" />
                        <span>"Timestamps"</span>
                    </label>
                    <label>
                        "Skip from (s)"
                        <input type="number" data-from="" min="0" max={max_trim.clone()} step="1" inputmode="numeric" placeholder="0" />
                    </label>
                    <label>
                        "to (s)"
                        <input type="number" data-to="" min="0" max={max_trim} step="1" inputmode="numeric" placeholder="end" />
                    </label>
                    <button type="button" class="btn btn-primary" data-copy="">"Copy"</button>
                    <button type="button" class="btn btn-ghost" data-copy-md="">"Copy Markdown"</button>
                    <button type="button" class="btn btn-ghost" data-dl="txt">"TXT"</button>
                    <button type="button" class="btn btn-ghost" data-dl="srt">"SRT"</button>
                    <button type="button" class="btn btn-ghost" data-dl="vtt">"VTT"</button>
                    <button type="button" class="btn btn-ghost" data-dl="md">"MD"</button>
                    <button type="button" class="btn btn-ghost" data-dl="json">"JSON"</button>
                </div>
                <div class="toolbar prompts">
                    <span class="prompt-label">"AI prompts"</span>
                    <button type="button" class="btn btn-ghost" data-prompt="summary">"Summary"</button>
                    <button type="button" class="btn btn-ghost" data-prompt="notes">"Notes"</button>
                    <button type="button" class="btn btn-ghost" data-prompt="quiz">"Quiz"</button>
                    <button type="button" class="btn btn-ghost" data-prompt="quotes">"Quotes"</button>
                </div>
                <p class="hint" data-status="" hidden="" role="status" aria-live="polite"></p>
                {View::raw(cues_html)}
            </section>
        </div>
    }
}

fn render_cues(video_id: &str, cues: &[Cue]) -> String {
    let mut s = String::from(r#"<div class="cues" data-cues="">"#);
    for cue in cues {
        let href = format!(
            "https://www.youtube.com/watch?v={video_id}&t={}s",
            cue.start_ms / 1000
        );
        let clock = format_clock(cue.start_ms, false);
        s.push_str(&format!(
            r#"<a class="cue" data-ms="{ms}" data-text="{text_attr}" href="{href}" rel="noreferrer noopener"><time>{clock}</time><span>{text}</span></a>"#,
            ms = cue.start_ms,
            text_attr = html_escape::encode_double_quoted_attribute(&cue.text),
            href = html_escape::encode_double_quoted_attribute(&href),
            clock = html_escape::encode_text(&clock),
            text = html_escape::encode_text(&cue.text),
        ));
    }
    s.push_str("</div>");
    s
}

fn render_chapters(video_id: &str, chapters: &[crate::youtube::Chapter]) -> String {
    if chapters.is_empty() {
        return String::new();
    }
    let mut s = String::from(r#"<nav class="chapters" data-chapters="" aria-label="Chapters"><p>Chapters</p><ul>"#);
    for ch in chapters {
        let href = format!(
            "https://www.youtube.com/watch?v={video_id}&t={}s",
            ch.start_ms / 1000
        );
        s.push_str(&format!(
            r#"<li><a data-ms="{ms}" href="{href}" rel="noreferrer noopener"><time>{clock}</time> {title}</a></li>"#,
            ms = ch.start_ms,
            href = html_escape::encode_double_quoted_attribute(&href),
            clock = html_escape::encode_text(&format_clock(ch.start_ms, false)),
            title = html_escape::encode_text(&ch.title),
        ));
    }
    s.push_str("</ul></nav>");
    s
}
