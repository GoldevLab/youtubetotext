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
    let cues_html = render_cues(&doc.cues);
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
    const liveWorkspace = () =>
        document.querySelector("#resuma-root #ytt-ws") || document.getElementById("ytt-ws");
    const fmtClock = (ms) => {
        const s = Math.floor(ms / 1000);
        const h = Math.floor(s / 3600);
        const m = Math.floor((s % 3600) / 60);
        const sec = s % 60;
        return h ? `${h}:${String(m).padStart(2,"0")}:${String(sec).padStart(2,"0")}`
                 : `${m}:${String(sec).padStart(2,"0")}`;
    };
    const copyViaExec = (body) => {
        const ta = document.createElement("textarea");
        ta.value = body;
        ta.setAttribute("readonly", "");
        ta.setAttribute("aria-hidden", "true");
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
    const writeClipboard = (body) => {
        if (copyViaExec(body)) return Promise.resolve();
        if (navigator.clipboard?.writeText) return navigator.clipboard.writeText(body);
        return Promise.reject(new Error("copy"));
    };
    const cuesFromWorkspace = (ws) => {
        const q = (ws?.querySelector("[data-search]")?.value || "").trim().toLowerCase();
        const from = Math.max(0, Number(ws?.querySelector("[data-from]")?.value || 0) * 1000);
        const toRaw = Number(ws?.querySelector("[data-to]")?.value || 0);
        const to = toRaw > 0 ? toRaw * 1000 : Infinity;
        const out = [];
        ws?.querySelectorAll(".cue").forEach((el) => {
            if (el.hidden) return;
            const ms = Number(el.dataset.ms || 0);
            if (ms < from || ms > to) return;
            const text = el.querySelector("[data-cue-text]")?.textContent ?? el.dataset.text ?? "";
            if (q && !String(text).toLowerCase().includes(q)) return;
            out.push({ start_ms: ms, text });
        });
        return out;
    };
    const flashCopied = (btn, idle) => {
        if (!btn) return;
        const label = btn.querySelector("[data-copy-label]");
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
    };
    const setWsStatus = (ws, t, ms) => {
        const status = ws?.querySelector("[data-status]");
        if (status) { status.textContent = t || ""; status.hidden = !t; }
        if (t) setTimeout(() => { if (status && status.textContent === t) { status.textContent = ""; status.hidden = true; } }, ms || 1800);
    };
    if (!window.__yttCopyBound) {
        window.__yttCopyBound = true;
        document.addEventListener("click", (e) => {
            const t = e.target instanceof Element ? e.target : e.target?.parentElement;
            if (!t) return;
            const copyBtn = t.closest("[data-copy]");
            const mdBtn = t.closest("[data-copy-md]");
            if (!copyBtn && !mdBtn) return;
            const ws = t.closest("#ytt-ws") || liveWorkspace();
            if (!ws) return;
            e.preventDefault();
            const md = !!mdBtn;
            const btn = mdBtn || copyBtn;
            const cues = cuesFromWorkspace(ws);
            if (!cues.length) {
                setWsStatus(ws, "Nothing to copy — clear the search or widen the time range.", 2200);
                return;
            }
            const id = ws.dataset.vid || "";
            const stamps = ws.querySelector("[data-stamps]")?.checked;
            let body = "";
            if (md) {
                body = cues.map((c) => `- [${fmtClock(c.start_ms)}](https://www.youtube.com/watch?v=${id}&t=${Math.floor(c.start_ms/1000)}s) ${c.text}`).join("\n");
            } else if (stamps) {
                body = cues.map((c) => `[${fmtClock(c.start_ms)}] ${c.text}`).join("\n");
            } else {
                body = cues.map((c) => c.text).join("\n");
            }
            Promise.resolve(writeClipboard(body)).then(() => {
                flashCopied(btn, md ? "Copy Markdown" : "Copy transcript");
                setWsStatus(ws, "Transcript copied to the clipboard.", 1800);
            }).catch(() => {
                setWsStatus(ws, "Could not copy — select the transcript and copy manually.", 2200);
            });
        }, true);
    }
    const root = liveWorkspace();
    if (!root || root.dataset.ready === "1") return;
    root.dataset.ready = "1";
    const dataEl = root.querySelector("#ytt-data") || document.getElementById("ytt-data");
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
    const pickBtn = root.querySelector("[data-pick-range]");
    const editBtn = root.querySelector("[data-edit]");
    const EDIT_KEY = "youtubetotext-edits-" + (data.video_id || root.dataset.vid);
    let player = null;
    let pendingMs = 0;
    let ytApi = null;
    let pickRange = false;
    let rangePick = null;

    const go = (path) => {
        document.documentElement.classList.add("is-navigating");
        if (typeof __resuma?.navigate === "function") {
            Promise.resolve(__resuma.navigate(path)).finally(() => {
                document.documentElement.classList.remove("is-navigating");
            });
        } else {
            location.assign(path);
        }
    };
    const parseId = (raw) => {
        const s = String(raw || "").trim();
        if (/^[\w-]{11}$/.test(s)) return s;
        try {
            const u = new URL(s.startsWith("http") ? s : "https://" + s);
            const v = u.searchParams.get("v") || u.searchParams.get("vi");
            if (v && /^[\w-]{11}$/.test(v)) return v;
            const parts = u.pathname.split("/").filter(Boolean);
            const host = u.hostname.replace(/^www\./, "");
            if (host === "youtu.be" && /^[\w-]{11}$/.test(parts[0] || "")) return parts[0];
            const i = parts.findIndex((p) => ["embed","shorts","live","v","watch"].includes(p));
            const id = i >= 0 ? (parts[i + 1] || "").slice(0, 11) : "";
            if (/^[\w-]{11}$/.test(id)) return id;
        } catch (_) {}
        return null;
    };
    const loadEdits = () => {
        try { return JSON.parse(localStorage.getItem(EDIT_KEY) || "{}"); } catch { return {}; }
    };
    const saveEdits = () => {
        const map = {};
        for (const c of data.cues) map[c.start_ms] = c.text;
        try { localStorage.setItem(EDIT_KEY, JSON.stringify(map)); } catch (_) {}
    };
    const applyStoredEdits = () => {
        const edits = loadEdits();
        data.cues.forEach((c) => {
            const next = edits[c.start_ms] ?? edits[String(c.start_ms)];
            if (typeof next === "string") c.text = next;
        });
        list?.querySelectorAll(".cue").forEach((el) => {
            const ms = el.dataset.ms;
            const next = edits[ms] ?? edits[Number(ms)];
            if (typeof next !== "string") return;
            el.dataset.text = next;
            const span = el.querySelector("[data-cue-text]");
            if (span) span.textContent = next;
        });
    };
    applyStoredEdits();

    const setStatus = (t) => { if (status) { status.textContent = t || ""; status.hidden = !t; } };
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
        const fromRaw = Number(fromInp?.value || 0);
        const toRaw = Number(toInp?.value || 0);
        let shown = 0;
        list?.querySelectorAll(".cue").forEach((el) => {
            const ms = Number(el.dataset.ms || 0);
            const text = (el.dataset.text || el.textContent || "").toLowerCase();
            const hide = ms < from || ms > to || (q && !text.includes(q));
            el.hidden = hide;
            el.classList.toggle("is-hit", !hide && !!q);
            el.classList.toggle("is-range-start", !hide && fromRaw > 0 && ms === from);
            el.classList.toggle("is-range-end", !hide && (ms === rangePick || (toRaw > 0 && ms === to)));
            if (!hide) shown++;
        });
        if (countEl) countEl.textContent = q || (fromInp?.value || toInp?.value)
            ? `${shown} shown`
            : "";
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
            if (player) {
                try { player.seekTo(pendingMs / 1000, true); player.playVideo?.(); } catch (_) {}
                return;
            }
            playerP = null;
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
    const navigateLang = () => {
        const form = root.querySelector(".toolbar-lang");
        if (!form) return;
        const params = new URLSearchParams(new FormData(form));
        if (!params.get("tlang")) params.delete("tlang");
        const qs = params.toString();
        const path = form.getAttribute("action") || location.pathname;
        go(qs ? `${path}?${qs}` : path);
    };
    root.querySelector(".toolbar-lang")?.addEventListener("submit", (e) => {
        e.preventDefault();
        navigateLang();
    });
    root.querySelectorAll("[data-lang-select], [data-tlang-select]").forEach((el) => {
        el.addEventListener("change", navigateLang);
    });
    const setTrim = (fromSec, toSec) => {
        if (fromInp) fromInp.value = fromSec > 0 ? String(fromSec) : "";
        if (toInp) toInp.value = toSec > 0 ? String(toSec) : "";
        applyFilters();
    };
    root.querySelectorAll("[data-skip-intro]").forEach((btn) => {
        btn.addEventListener("click", () => {
            const n = Number(btn.getAttribute("data-skip-intro") || 0);
            const toSec = Number(toInp?.value || 0);
            setTrim(n, toSec);
            setStatus(`Skipped the first ${n} seconds.`);
            setTimeout(() => setStatus(""), 1600);
        });
    });
    root.querySelectorAll("[data-skip-outro]").forEach((btn) => {
        btn.addEventListener("click", () => {
            const n = Number(btn.getAttribute("data-skip-outro") || 0);
            const max = Math.floor(durationMs() / 1000);
            const end = Math.max(Number(fromInp?.value || 0), max - n);
            setTrim(Number(fromInp?.value || 0), end);
            setStatus(`Skipped the last ${n} seconds.`);
            setTimeout(() => setStatus(""), 1600);
        });
    });
    root.querySelector("[data-clear-trim]")?.addEventListener("click", () => {
        rangePick = null;
        pickRange = false;
        pickBtn?.setAttribute("aria-pressed", "false");
        pickBtn?.classList.remove("is-on");
        setTrim(0, 0);
        setStatus("Trim cleared.");
        setTimeout(() => setStatus(""), 1400);
    });
    pickBtn?.addEventListener("click", () => {
        pickRange = !pickRange;
        rangePick = null;
        pickBtn.setAttribute("aria-pressed", pickRange ? "true" : "false");
        pickBtn.classList.toggle("is-on", pickRange);
        setStatus(pickRange ? "Click a start line, then an end line." : "");
        if (!pickRange) setTimeout(() => setStatus(""), 400);
    });
    const setEditing = (on) => {
        root.classList.toggle("is-editing", on);
        editBtn?.setAttribute("aria-pressed", on ? "true" : "false");
        editBtn?.classList.toggle("is-on", on);
        list?.querySelectorAll("[data-cue-text]").forEach((span) => {
            span.contentEditable = on ? "true" : "false";
            span.spellcheck = true;
        });
        setStatus(on ? "Edit lines, then Copy or download. Changes stay on this device." : "");
        if (!on) setTimeout(() => setStatus(""), 1600);
    };
    editBtn?.addEventListener("click", () => setEditing(!root.classList.contains("is-editing")));
    list?.addEventListener("input", (e) => {
        const span = e.target.closest?.("[data-cue-text]");
        const cueEl = span?.closest(".cue");
        if (!cueEl) return;
        const ms = Number(cueEl.dataset.ms || 0);
        const text = span.textContent || "";
        cueEl.dataset.text = text;
        const cue = data.cues.find((c) => c.start_ms === ms);
        if (cue) cue.text = text;
        saveEdits();
    });
    const applyBtn = root.querySelector("[data-apply]");
    if (applyBtn) applyBtn.hidden = true;
    root.dataset.ready = "1";
    root.querySelector("[data-play]")?.addEventListener("click", () => mountPlayer(0));
    const cueFromEvent = (e) => {
        const el = e.target instanceof Element ? e.target : e.target?.parentElement;
        return el?.closest?.(".cue");
    };
    list?.addEventListener("click", (e) => {
        const a = cueFromEvent(e);
        if (!a) return;
        e.preventDefault();
        if (root.classList.contains("is-editing") && e.target.closest("[data-cue-text]")) return;
        const ms = Number(a.dataset.ms || 0);
        if (pickRange) {
            if (rangePick == null) {
                rangePick = ms;
                if (fromInp) fromInp.value = String(Math.floor(ms / 1000));
                applyFilters();
                setStatus("Now click the last line to keep.");
                return;
            }
            let start = rangePick;
            let end = ms;
            if (end < start) { const t = start; start = end; end = t; }
            setTrim(Math.floor(start / 1000), Math.floor(end / 1000));
            rangePick = null;
            pickRange = false;
            pickBtn?.setAttribute("aria-pressed", "false");
            pickBtn?.classList.remove("is-on");
            setStatus("Range set. Copy and downloads use these lines.");
            setTimeout(() => setStatus(""), 2000);
            return;
        }
        mountPlayer(ms);
    });
    list?.addEventListener("keydown", (e) => {
        if (e.key !== "Enter" && e.key !== " ") return;
        const cue = cueFromEvent(e);
        if (!cue) return;
        if (root.classList.contains("is-editing") && e.target.closest("[data-cue-text]")) return;
        e.preventDefault();
        cue.click();
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
                await writeClipboard(body);
                setStatus("Prompt copied — paste it into any AI chat.");
            } catch { setStatus("Could not copy the prompt."); }
            setTimeout(() => setStatus(""), 2200);
        });
    });
    const another = root.querySelector("[data-another]");
    another?.addEventListener("submit", (e) => {
        e.preventDefault();
        const input = another.querySelector('input[name="url"]');
        const err = another.querySelector("[data-another-error]");
        const id = parseId(input?.value);
        if (!id) {
            if (err) {
                err.hidden = false;
                err.textContent = "That does not look like a YouTube link.";
            }
            input?.focus();
            return;
        }
        if (err) err.hidden = true;
        go("/v/" + encodeURIComponent(id));
    });
    document.addEventListener("keydown", (e) => {
        const tag = document.activeElement?.tagName;
        if (e.key === "/" && tag !== "INPUT" && tag !== "TEXTAREA" && tag !== "SELECT" && !document.activeElement?.isContentEditable) {
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
                            <span class="play-badge" aria-hidden="true">"Play"</span>
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
                    {crate::ads::slot("workspace-player", "infeed")}
                    <form class="another-form" data-another="">
                        <label>
                            "Another video"
                            <span class="hero-field">
                                <input
                                    name="url"
                                    type="text"
                                    inputmode="url"
                                    enterkeyhint="go"
                                    autocomplete="url"
                                    spellcheck="false"
                                    placeholder="Paste another YouTube link"
                                />
                                <button type="submit" class="btn btn-secondary">"Go"</button>
                            </span>
                        </label>
                        <p class="hint form-error" data-another-error="" hidden="" role="alert"></p>
                    </form>
                </div>
            </aside>
            <section class="transcript-col">
                <form class="toolbar toolbar-lang" method="get" action={action}>
                    <label>
                        "Captions"
                        <select name="lang" data-lang-select="">{track_options}</select>
                    </label>
                    <label>
                        "Translate"
                        <select name="tlang" data-tlang-select="">{tlang_options}</select>
                    </label>
                    <button type="submit" class="btn btn-secondary" data-apply="">"Apply"</button>
                </form>
                <div class="toolbar">
                    <label class="grow">
                        "Search"
                        <input type="search" data-search="" placeholder="Filter lines  (press /)" autocomplete="off" />
                    </label>
                    <span class="match-count" data-count=""></span>
                </div>
                <fieldset class="tool-group">
                    <legend>"Trim"</legend>
                    <div class="toolbar trim-chips">
                        <button type="button" class="btn btn-ghost" data-skip-intro="30">"Skip intro 30s"</button>
                        <button type="button" class="btn btn-ghost" data-skip-intro="60">"Skip intro 60s"</button>
                        <button type="button" class="btn btn-ghost" data-skip-outro="30">"Skip outro 30s"</button>
                        <button type="button" class="btn btn-ghost" data-skip-outro="60">"Skip outro 60s"</button>
                        <button type="button" class="btn btn-ghost" data-pick-range="" aria-pressed="false">"Click two lines"</button>
                        <button type="button" class="btn btn-ghost" data-clear-trim="">"Clear trim"</button>
                    </div>
                    <div class="toolbar toolbar-range">
                        <label>
                            "Skip from (s)"
                            <input type="number" data-from="" min="0" max={max_trim.clone()} step="1" inputmode="numeric" placeholder="0" />
                        </label>
                        <label>
                            "to (s)"
                            <input type="number" data-to="" min="0" max={max_trim} step="1" inputmode="numeric" placeholder="end" />
                        </label>
                    </div>
                </fieldset>
                <div class="toolbar toolbar-copy">
                    <button type="button" class="btn btn-primary btn-copy" data-copy="">
                        <span class="btn-copy-icon" aria-hidden="true"></span>
                        <span data-copy-label="">"Copy transcript"</span>
                        <span class="btn-copy-burst" aria-hidden="true"></span>
                    </button>
                    <div class="toolbar-copy-more">
                        <label class="check">
                            <input type="checkbox" data-stamps="" />
                            <span>"Timestamps"</span>
                        </label>
                        <button type="button" class="btn btn-ghost" data-copy-md="">"Copy Markdown"</button>
                        <button type="button" class="btn btn-ghost" data-edit="" aria-pressed="false">"Edit"</button>
                    </div>
                </div>
                <p class="copy-status hint" data-status="" hidden="" role="status" aria-live="polite"></p>
                <fieldset class="tool-group">
                    <legend>"Download"</legend>
                    <div class="toolbar toolbar-exports">
                        <button type="button" class="btn btn-ghost" data-dl="txt">"TXT"</button>
                        <button type="button" class="btn btn-ghost" data-dl="srt">"SRT"</button>
                        <button type="button" class="btn btn-ghost" data-dl="vtt">"VTT"</button>
                        <button type="button" class="btn btn-ghost" data-dl="md">"MD"</button>
                        <button type="button" class="btn btn-ghost" data-dl="json">"JSON"</button>
                    </div>
                </fieldset>
                <fieldset class="tool-group">
                    <legend>"AI prompts"</legend>
                    <div class="toolbar prompts">
                        <button type="button" class="btn btn-ghost" data-prompt="summary">"Summary"</button>
                        <button type="button" class="btn btn-ghost" data-prompt="notes">"Notes"</button>
                        <button type="button" class="btn btn-ghost" data-prompt="quiz">"Quiz"</button>
                        <button type="button" class="btn btn-ghost" data-prompt="quotes">"Quotes"</button>
                    </div>
                </fieldset>
                {crate::ads::slot("workspace-cues", "infeed")}
                {View::raw(cues_html)}
            </section>
        </div>
    }
}

fn render_cues(cues: &[Cue]) -> String {
    let mut s = String::from(r#"<div class="cues" data-cues="">"#);
    for cue in cues {
        let clock = format_clock(cue.start_ms, false);
        s.push_str(&format!(
            r#"<div class="cue" data-ms="{ms}" data-text="{text_attr}" role="button" tabindex="0"><time>{clock}</time><span data-cue-text="">{text}</span></div>"#,
            ms = cue.start_ms,
            text_attr = html_escape::encode_double_quoted_attribute(&cue.text),
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
