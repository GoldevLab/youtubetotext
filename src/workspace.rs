//! Video workspace — player, search, copy, downloads, cue list.

use resuma::prelude::*;
use serde::Serialize;

use crate::export::{format_clock, reading_minutes};
use crate::family::Mode;
use crate::summary::extractive_summary;
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
pub fn workspace(doc: TranscriptDoc, lang: String, tlang: String, mode: String) -> View {
    let video_id = doc.video_id.clone();
    let title = doc.title.clone();
    let author = doc.author.clone();
    let selected_track = track_key(&doc.track.lang, &doc.track.kind);
    let lang_attr = selected_track;
    let _ = lang;
    let duration = format_clock(doc.duration_secs.saturating_mul(1000), false);
    let words = doc.word_count();
    let read_mins = reading_minutes(&doc.plain_text());
    let track_label = doc.track.name.clone();
    let is_auto = doc.track.kind == "asr";
    let thumb = format!("https://i.ytimg.com/vi/{video_id}/hqdefault.jpg");
    let watch = format!("https://www.youtube.com/watch?v={video_id}");
    let action = "/".to_string();
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
    let parsed_mode = Mode::parse(&mode);
    let audio_href = format!("/api/audio?v={video_id}&fmt=mp3");
    let video_href = format!("/api/video?v={video_id}&q=720");
    let recap = extractive_summary(&doc);
    let ws_class = format!("workspace is-mode-{}", parsed_mode.slug());
    let share = crate::family::app_href(&video_id, parsed_mode);
    let mode_slug = parsed_mode.slug().to_string();

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
    const parseId = (raw) => {
        const s = String(raw || "").trim();
        if (/^[\w-]{11}$/.test(s)) return s;
        try {
            const u = new URL(s.startsWith("http") ? s : "https://" + s);
            const v = u.searchParams.get("v") || u.searchParams.get("vi");
            if (v && /^[\w-]{11}$/.test(v)) return v;
            const parts = u.pathname.split("/").filter(Boolean);
            const host = u.hostname.replace(/^www./, "");
            if (host === "youtu.be" && /^[\w-]{11}$/.test(parts[0] || "")) return parts[0];
            const i = parts.findIndex((p) => ["embed","shorts","live","v","watch"].includes(p));
            const id = i >= 0 ? (parts[i + 1] || "").slice(0, 11) : "";
            if (/^[\w-]{11}$/.test(id)) return id;
        } catch (_) {}
        return null;
    };
    const go = (path) => {
        document.documentElement.classList.add("is-navigating");
        if (typeof __resuma?.navigate === "function") {
            Promise.resolve(__resuma.navigate(path)).catch(() => {
                location.assign(path);
            }).finally(() => {
                document.documentElement.classList.remove("is-navigating");
            });
        } else {
            location.assign(path);
        }
    };
    const wsData = (ws) => {
        if (!ws) return { video_id: "", title: "", author: "", cues: [] };
        const dataEl = ws.querySelector("#ytt-data") || document.getElementById("ytt-data");
        let data = { video_id: ws?.dataset.vid || "", title: "", author: "", cues: [] };
        try { data = JSON.parse(dataEl?.textContent || "{}"); } catch (_) {}
        return data;
    };
    const durationMs = (ws) => {
        const secs = Number(ws?.dataset.duration || 0);
        if (secs > 0) return secs * 1000;
        const data = wsData(ws);
        return (data.cues.at(-1)?.start_ms || 0) + 5000;
    };
    const rangeMs = (ws) => {
        const fromInp = ws.querySelector("[data-from]");
        const toInp = ws.querySelector("[data-to]");
        const max = durationMs(ws);
        const from = Math.max(0, Number(fromInp?.value || 0) * 1000);
        const toRaw = Number(toInp?.value || 0);
        const to = toRaw > 0 ? Math.min(toRaw * 1000, max) : max;
        return { from, to, fromInp, toInp };
    };
    const visibleCues = (ws) => {
        const data = wsData(ws);
        const q = (ws.querySelector("[data-search]")?.value || "").trim().toLowerCase();
        const { from, to } = rangeMs(ws);
        return data.cues.filter((c) => {
            if (c.start_ms < from || c.start_ms > to) return false;
            if (q && !c.text.toLowerCase().includes(q)) return false;
            return true;
        });
    };
    const applyFilters = (ws) => {
        const search = ws.querySelector("[data-search]");
        const countEl = ws.querySelector("[data-count]");
        const list = ws.querySelector("[data-cues]");
        const q = (search?.value || "").trim().toLowerCase();
        const { from, to, fromInp, toInp } = rangeMs(ws);
        const fromRaw = Number(fromInp?.value || 0);
        const toRaw = Number(toInp?.value || 0);
        const st = window.__yttForgeState || {};
        let shown = 0;
        list?.querySelectorAll(".cue").forEach((el) => {
            const ms = Number(el.dataset.ms || 0);
            const text = (el.dataset.text || el.textContent || "").toLowerCase();
            const hide = ms < from || ms > to || (q && !text.includes(q));
            el.hidden = hide;
            el.classList.toggle("is-hit", !hide && !!q);
            el.classList.toggle("is-range-start", !hide && fromRaw > 0 && ms === from);
            el.classList.toggle("is-range-end", !hide && (ms === st.rangePick || (toRaw > 0 && ms === to)));
            if (!hide) shown++;
        });
        if (countEl) countEl.textContent = q || (fromInp?.value || toInp?.value) ? `${shown} shown` : "";
    };
    const setStatus = (ws, t, holdOrMs) => {
        const nodes = ws?.querySelectorAll("[data-status], [data-apply-status]") || [];
        nodes.forEach((status) => {
            status.textContent = t || "";
            status.hidden = !t;
        });
        const hold = holdOrMs === true;
        const ms = typeof holdOrMs === "number" ? holdOrMs : 4000;
        if (t && !hold) setTimeout(() => {
            nodes.forEach((status) => {
                if (status.textContent === t) { status.textContent = ""; status.hidden = true; }
            });
        }, ms);
    };
    const setBtnBusy = (btn, busy, label) => {
        if (!btn) return;
        const labelEl = btn.querySelector("[data-apply-label]");
        if (busy) {
            if (!btn.dataset.idleLabel) {
                btn.dataset.idleLabel = (labelEl?.textContent || btn.textContent || "Apply").trim() || "Apply";
            }
            btn.disabled = true;
            btn.setAttribute("aria-busy", "true");
            btn.classList.add("is-busy");
            if (labelEl) labelEl.textContent = label || "Working…";
            else btn.textContent = label || "Working…";
        } else {
            btn.disabled = false;
            btn.removeAttribute("aria-busy");
            btn.classList.remove("is-busy");
            const idle = btn.dataset.idleLabel || "Apply";
            delete btn.dataset.idleLabel;
            if (labelEl) labelEl.textContent = idle;
            else btn.textContent = idle;
        }
    };
    const downloadFmt = (ws, fmt) => {
        const cues = visibleCues(ws);
        if (!cues.length) {
            setStatus(ws, "Nothing to download — clear the search or widen the time range.");
            return;
        }
        const data = wsData(ws);
        const id = data.video_id || ws.dataset.vid;
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
            const stamps = ws.querySelector("[data-stamps]")?.checked;
            body = (stamps ? cues.map((c) => `[${fmtClock(c.start_ms)}] ${c.text}`) : cues.map((c) => c.text)).join("\n");
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
        setStatus(ws, `Downloaded ${name}.`);
    };
    const downloadAudio = async (ws, href) => {
        startMediaDownload(ws, href, "audio");
    };
    const armVideoDialog = (dlg) => {
        if (!(dlg instanceof HTMLDialogElement) || dlg.dataset.ready) return;
        dlg.dataset.ready = "1";
        if (!("closedBy" in HTMLDialogElement.prototype)) {
            dlg.addEventListener("click", (event) => {
                if (event.target !== dlg) return;
                const rect = dlg.getBoundingClientRect();
                const inside = rect.top <= event.clientY && event.clientY <= rect.top + rect.height
                    && rect.left <= event.clientX && event.clientX <= rect.left + rect.width;
                if (!inside) dlg.close();
            });
        }
    };
    const startVideoDownload = (ws, href) => startMediaDownload(ws, href, "video");
    const startMediaDownload = async (ws, href, kind) => {
        const dlg = ws.querySelector("#r-modal-media-dl") || ws.querySelector(".dl-dialog");
        armVideoDialog(dlg);
        if (dlg instanceof HTMLDialogElement) {
            const title = dlg.querySelector("[data-dl-title]") || dlg.querySelector("h2");
            const lead = dlg.querySelector("[data-dl-lead]") || dlg.querySelector("p");
            if (title) title.textContent = kind === "audio" ? "Your audio is downloading" : "Your video is downloading";
            if (lead) lead.textContent = kind === "audio"
                ? "The file will save to your downloads folder. MP3 conversion can take a moment."
                : "The file will save to your downloads folder. Higher qualities can take a minute.";
            try {
                const open = globalThis.__resuma?.showModal?.("media-dl");
                if (open && typeof open.then === "function") await open;
                else if (typeof dlg.showModal === "function" && !dlg.open) dlg.showModal();
            } catch (_) {
                if (typeof dlg.showModal === "function" && !dlg.open) dlg.showModal();
            }
            try { globalThis.__yttFillAds?.(dlg); } catch (_) {}
        }
        const a = document.createElement("a");
        a.href = href;
        a.download = "";
        a.rel = "noopener";
        a.setAttribute("data-r-full", "");
        a.style.display = "none";
        document.body.append(a);
        a.click();
        a.remove();
    };
    const sourcePayload = (ws) => {
        const el = ws.querySelector("#ytt-source");
        try {
            const src = JSON.parse(el?.textContent || "{}");
            if (Array.isArray(src.cues) && src.cues.length) return src;
        } catch (_) {}
        return wsData(ws);
    };
    const TRANSLATE_CHUNK = 45;
    const translateCues = async (ws, vid, sl, tlang, cues, applyBtn) => {
        const out = [];
        const total = cues.length;
        for (let i = 0; i < total; i += TRANSLATE_CHUNK) {
            const chunk = cues.slice(i, i + TRANSLATE_CHUNK);
            const done = Math.min(i + chunk.length, total);
            setBtnBusy(applyBtn, true, total > TRANSLATE_CHUNK ? `Translating… ${done}/${total}` : "Translating…");
            setStatus(ws, total > TRANSLATE_CHUNK
                ? `Translating captions… ${done} of ${total}`
                : "Translating captions…", true);
            const res = await fetch("/api/translate", {
                method: "POST",
                headers: { Accept: "application/json", "Content-Type": "application/json" },
                credentials: "same-origin",
                body: JSON.stringify({ video_id: vid, lang: sl, tlang, cues: chunk }),
            });
            const data = await res.json().catch(() => ({}));
            if (!res.ok || data.error || !Array.isArray(data.cues) || data.cues.length !== chunk.length) {
                throw new Error(data.error || "Could not translate those captions. Try Apply again in a minute.");
            }
            out.push(...data.cues);
        }
        return out;
    };
    const paintCues = (ws, payload) => {
        const el = ws.querySelector("#ytt-data");
        if (el) el.textContent = JSON.stringify(payload);
        const list = ws.querySelector("[data-cues]");
        if (!list || !Array.isArray(payload.cues)) return;
        const editing = ws.classList.contains("is-editing");
        const frag = document.createDocumentFragment();
        for (const c of payload.cues) {
            const div = document.createElement("div");
            div.className = "cue";
            div.dataset.ms = String(c.start_ms ?? 0);
            div.dataset.text = c.text || "";
            div.setAttribute("role", "button");
            div.tabIndex = 0;
            const time = document.createElement("time");
            time.textContent = fmtClock(Number(c.start_ms || 0));
            const span = document.createElement("span");
            span.setAttribute("data-cue-text", "");
            span.textContent = c.text || "";
            if (editing) {
                span.contentEditable = "true";
                span.spellcheck = true;
            }
            div.append(time, span);
            frag.append(div);
        }
        list.replaceChildren(frag);
    };
    const storeSource = (ws, payload) => {
        const el = ws.querySelector("#ytt-source");
        if (el) el.textContent = JSON.stringify(payload);
    };
    const paintMode = (ws, mode) => {
        if (!ws || !mode) return;
        ws.dataset.mode = mode;
        ws.className = String(ws.className || "").replace(/\bis-mode-\w+/g, "").trim() + " is-mode-" + mode;
        ws.querySelectorAll(".mode-tab").forEach((el) => {
            el.classList.toggle("is-active", el.getAttribute("data-mode-tab") === mode);
        });
        const modeInp = ws.querySelector('.toolbar-lang input[name="mode"]');
        if (modeInp) modeInp.value = mode;
        const u = new URL(location.href);
        if (u.pathname === "/" || u.pathname === "") {
            u.searchParams.set("v", ws.dataset.vid || "");
            u.searchParams.set("mode", mode);
            history.replaceState({}, "", u.pathname + u.search);
        }
    };
    const langHref = (ws) => {
        const form = ws.querySelector(".toolbar-lang");
        const fd = form ? new FormData(form) : new FormData();
        const lang = String(fd.get("lang") || "").trim();
        const tlang = String(fd.get("tlang") || "").trim();
        const vid = ws.dataset.vid || String(fd.get("v") || "");
        const mode = ws.dataset.mode || String(fd.get("mode") || "text") || "text";
        const u = new URL(location.href);
        if (u.pathname === "/" || u.pathname === "") {
            u.pathname = "/";
            u.search = "";
            u.searchParams.set("v", vid);
            u.searchParams.set("mode", mode);
        } else {
            u.searchParams.delete("v");
        }
        if (lang) u.searchParams.set("lang", lang); else u.searchParams.delete("lang");
        if (tlang) u.searchParams.set("tlang", tlang); else u.searchParams.delete("tlang");
        u.searchParams.delete("_r");
        return u.pathname + u.search;
    };
    const applyLang = async (ws) => {
        const modeInp = ws.querySelector('.toolbar-lang input[name="mode"]');
        if (modeInp) modeInp.value = ws.dataset.mode || modeInp.value || "text";
        const form = ws.querySelector(".toolbar-lang");
        const fd = form ? new FormData(form) : new FormData();
        const lang = String(fd.get("lang") || "").trim();
        const tlang = String(fd.get("tlang") || "").trim();
        const vid = ws.dataset.vid || String(fd.get("v") || "");
        if (!vid) return;
        const href = langHref(ws);
        const applyBtn = ws.querySelector("[data-apply]");
        if (applyBtn?.getAttribute("aria-busy") === "true") return;
        const prevLang = ws.dataset.lang || "";
        const langChanged = Boolean(lang) && lang !== prevLang;
        setBtnBusy(applyBtn, true, langChanged ? "Loading…" : (tlang ? "Translating…" : "Applying…"));
        try {
            if (langChanged) {
                setStatus(ws, "Loading captions…", true);
                const qs = new URLSearchParams({ v: vid, fmt: "json" });
                if (lang) qs.set("lang", lang);
                const res = await fetch("/api/transcript?" + qs.toString(), {
                    headers: { Accept: "application/json" },
                    credentials: "same-origin",
                });
                const data = await res.json().catch(() => ({}));
                if (!res.ok || data.error || !Array.isArray(data.cues)) {
                    setStatus(ws, data.error || "Could not load that language. Try Apply again in a minute.");
                    return;
                }
                const payload = {
                    video_id: data.video_id || vid,
                    title: data.title || "",
                    author: data.author || "",
                    cues: data.cues,
                };
                storeSource(ws, payload);
                ws.dataset.lang = lang;
                const trackName = data.track && data.track.name;
                const metaBits = ws.querySelectorAll(".vid-meta span");
                if (trackName && metaBits.length >= 3) metaBits[2].textContent = trackName;
                if (!tlang) {
                    paintCues(ws, payload);
                    history.replaceState({}, "", href);
                    paintMode(ws, ws.dataset.mode || "text");
                    applyFilters(ws);
                    setStatus(ws, "Captions updated.");
                    return;
                }
            }
            if (!tlang) {
                const src = sourcePayload(ws);
                if (!Array.isArray(src.cues) || !src.cues.length) {
                    setStatus(ws, "Original captions are missing. Pick a caption track and Apply.");
                    return;
                }
                paintCues(ws, src);
                history.replaceState({}, "", href);
                paintMode(ws, ws.dataset.mode || "text");
                applyFilters(ws);
                setStatus(ws, "Original captions are on the page.");
                return;
            }
            const src = sourcePayload(ws);
            const cues = Array.isArray(src.cues) ? src.cues : [];
            if (!cues.length) {
                setStatus(ws, "Load captions first, then translate.");
                return;
            }
            const sl = (lang.split(/[:|]/)[0] || "auto").split("-")[0] || "auto";
            const next = await translateCues(ws, vid, sl, tlang, cues, applyBtn);
            paintCues(ws, {
                video_id: src.video_id || vid,
                title: src.title || "",
                author: src.author || "",
                cues: next,
            });
            history.replaceState({}, "", href);
            paintMode(ws, "translate");
            applyFilters(ws);
            setStatus(ws, "Translated captions are on the page.");
        } catch (err) {
            const msg = err && err.message ? String(err.message) : "Could not update captions.";
            setStatus(ws, msg);
        } finally {
            setBtnBusy(applyBtn, false);
        }
    };
    const root = liveWorkspace();
    if (root) {
    const dataEl = root.querySelector("#ytt-data") || document.getElementById("ytt-data");
    let data = { video_id: root.dataset.vid, title: "", author: "", cues: [] };
    try { data = JSON.parse(dataEl?.textContent || "{}"); } catch (_) {}
    try {
        const KEY = "youtubetotext-recent";
        const cur = JSON.parse(localStorage.getItem(KEY) || "[]");
        const next = [{ id: data.video_id, title: data.title || data.video_id, at: Date.now() }, ...cur.filter((x) => x.id !== data.video_id)].slice(0, 8);
        localStorage.setItem(KEY, JSON.stringify(next));
    } catch (_) {}
    const applyStoredEdits = (ws) => {
        const dataNow = wsData(ws);
        const edits = (() => {
            try { return JSON.parse(localStorage.getItem("youtubetotext-edits-" + (dataNow.video_id || ws.dataset.vid || "") + "-" + (location.search || "")) || "{}"); } catch { return {}; }
        })();
        dataNow.cues.forEach((c) => {
            const next = edits[c.start_ms] ?? edits[String(c.start_ms)];
            if (typeof next === "string") c.text = next;
        });
        const el = ws.querySelector("#ytt-data");
        if (el) el.textContent = JSON.stringify(dataNow);
        ws.querySelectorAll(".cue").forEach((cueEl) => {
            const ms = cueEl.dataset.ms;
            const next = edits[ms] ?? edits[Number(ms)];
            if (typeof next !== "string") return;
            cueEl.dataset.text = next;
            const span = cueEl.querySelector("[data-cue-text]");
            if (span) span.textContent = next;
        });
    };
    applyStoredEdits(root);
    applyFilters(root);
    root.dataset.ready = "1";
    if (root.dataset.mode === "translate") {
        root.querySelector("[data-tlang-select]")?.focus();
    } else if (root.dataset.mode === "summary") {
        root.querySelector(".recap")?.scrollIntoView({ block: "nearest", behavior: "smooth" });
    } else if (root.dataset.mode === "srt") {
        root.querySelector('[data-dl="srt"]')?.focus();
    } else if (root.dataset.mode === "audio") {
        root.querySelector("[data-audio]")?.focus();
    }
    }

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
    const mountPlayer = async (ws, startMs) => {
        window.__yttForgeState = window.__yttForgeState || { pickRange: false, rangePick: null, player: null, playerP: null, pendingMs: 0, vid: "" };
        const host = ws.querySelector("[data-player]");
        if (!host) return;
        const S = window.__yttForgeState;
        const vid = ws.dataset.vid || "";
        if (S.vid && S.vid !== vid) {
            S.player = null;
            S.playerP = null;
        }
        S.vid = vid;
        S.pendingMs = startMs || 0;
        if (S.player) {
            try { S.player.seekTo(S.pendingMs / 1000, true); S.player.playVideo?.(); } catch (_) {}
            return;
        }
        if (S.playerP) {
            await S.playerP;
            if (S.player) {
                try { S.player.seekTo(S.pendingMs / 1000, true); S.player.playVideo?.(); } catch (_) {}
                return;
            }
            S.playerP = null;
        }
        const dataNow = wsData(ws);
        S.playerP = (async () => {
            host.innerHTML = `<div id="yt-frame"></div>`;
            const ytApi = await loadApi();
            if (!ytApi?.Player) {
                window.location.href = `https://www.youtube.com/watch?v=${dataNow.video_id}&t=${Math.floor(S.pendingMs/1000)}s`;
                return;
            }
            S.player = new ytApi.Player("yt-frame", {
                videoId: dataNow.video_id,
                host: "https://www.youtube-nocookie.com",
                playerVars: { rel: 0, modestbranding: 1, start: Math.floor(S.pendingMs/1000), origin: location.origin },
                events: {
                    onReady: (e) => { try { e.target.seekTo(S.pendingMs/1000, true); e.target.playVideo(); } catch (_) {} }
                }
            });
            let last = "";
            setInterval(() => {
                const live = liveWorkspace();
                const list = live?.querySelector("[data-cues]");
                let t = 0;
                try { t = S.player.getCurrentTime() * 1000; } catch (_) { return; }
                let current = null;
                for (const c of wsData(live).cues) {
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
        await S.playerP;
    };

    window.__yttForge = {
        applyLang, downloadFmt, downloadAudio, startVideoDownload, mountPlayer, paintMode, applyFilters, setStatus,
        parseId, go, visibleCues, writeClipboard, durationMs, wsData,
    };

    if (!window.__yttForgeUi) {
        window.__yttForgeUi = true;
        window.__yttForgeState = { pickRange: false, rangePick: null, player: null, playerP: null, pendingMs: 0, vid: "" };
        const prompts = {
            summary: "Summarize this YouTube transcript in 8 tight bullets. Keep names, numbers, and claims. Then give a 2-sentence overview.\n\n",
            notes: "Turn this transcript into structured study notes with headings, key points, and a short glossary of terms.\n\n",
            quiz: "Create 8 mixed quiz questions (multiple choice + short answer) from this transcript, with an answer key at the end.\n\n",
            quotes: "Extract the strongest quotes from this transcript. For each, add a one-line why-it-matters.\n\n",
        };
        document.addEventListener("click", (e) => {
            const F = window.__yttForge;
            if (!F) return;
            const t = e.target instanceof Element ? e.target : e.target?.parentElement;
            if (!t?.closest) return;
            const ws = t.closest("#ytt-ws") || liveWorkspace();
            const S = window.__yttForgeState;
            const tab = t.closest("[data-mode-tab]");
            if (tab && ws) {
                const mode = tab.getAttribute("data-mode-tab") || "text";
                e.preventDefault();
                F.paintMode(ws, mode);
                if (mode === "srt") {
                    F.downloadFmt(ws, "srt");
                    ws.querySelector('[data-dl="srt"]')?.focus();
                    F.setStatus(ws, "SRT downloaded. Use VTT beside it if you need WebVTT.", 3500);
                } else if (mode === "translate") {
                    ws.querySelector("[data-tlang-select]")?.focus();
                    F.setStatus(ws, "Pick a Translate language, then Apply.", 5000);
                } else if (mode === "summary") {
                    const recap = ws.querySelector(".recap");
                    recap?.scrollIntoView({ block: "nearest", behavior: "smooth" });
                    ws.querySelector('[data-prompt="summary"]')?.focus();
                    F.setStatus(ws, recap ? "Chapter recap is below. Summary copies a prompt for any AI chat." : "Copy Summary to paste into an AI chat.", 5000);
                } else {
                    ws.querySelector("[data-search]")?.focus();
                }
                return;
            }
            if (!ws) return;
            const audioBtn = t.closest("[data-audio]");
            if (audioBtn) {
                e.preventDefault();
                F.downloadAudio(ws, audioBtn.getAttribute("href") || "");
                return;
            }
            const videoBtn = t.closest("[data-video]");
            if (videoBtn) {
                e.preventDefault();
                F.startVideoDownload(ws, videoBtn.getAttribute("href") || "");
                return;
            }
            if (t.closest("[data-apply]")) {
                e.preventDefault();
                F.applyLang(ws);
                return;
            }
            if (t.closest("[data-play]")) {
                e.preventDefault();
                F.mountPlayer(ws, 0);
                return;
            }
            const dl = t.closest("[data-dl]");
            if (dl) {
                e.preventDefault();
                F.downloadFmt(ws, dl.getAttribute("data-dl"));
                return;
            }
            const promptBtn = t.closest("[data-prompt]");
            if (promptBtn) {
                e.preventDefault();
                const kind = promptBtn.getAttribute("data-prompt");
                const text = F.visibleCues(ws).map((c) => c.text).join("\n");
                if (!text.trim()) {
                    F.setStatus(ws, "Nothing to copy — clear the search or widen the time range.");
                    return;
                }
                const body = (prompts[kind] || prompts.summary) + text;
                Promise.resolve(F.writeClipboard(body)).then(() => F.setStatus(ws, "Prompt copied — paste it into any AI chat.")).catch(() => F.setStatus(ws, "Could not copy the prompt."));
                return;
            }
            const skipIntro = t.closest("[data-skip-intro]");
            if (skipIntro) {
                const n = Number(skipIntro.getAttribute("data-skip-intro") || 0);
                const fromInp = ws.querySelector("[data-from]");
                if (fromInp) fromInp.value = String(n);
                F.applyFilters(ws);
                F.setStatus(ws, `Skipped the first ${n} seconds.`);
                return;
            }
            const skipOutro = t.closest("[data-skip-outro]");
            if (skipOutro) {
                const n = Number(skipOutro.getAttribute("data-skip-outro") || 0);
                const max = Math.floor(F.durationMs(ws) / 1000);
                const fromInp = ws.querySelector("[data-from]");
                const toInp = ws.querySelector("[data-to]");
            const end = Math.max(Number(fromInp?.value || 0), max - n);
                if (toInp) toInp.value = String(end);
                F.applyFilters(ws);
                F.setStatus(ws, `Skipped the last ${n} seconds.`);
                return;
            }
            if (t.closest("[data-clear-trim]")) {
                S.rangePick = null;
                S.pickRange = false;
                const pickBtn = ws.querySelector("[data-pick-range]");
        pickBtn?.setAttribute("aria-pressed", "false");
        pickBtn?.classList.remove("is-on");
                const fromInp = ws.querySelector("[data-from]");
                const toInp = ws.querySelector("[data-to]");
                if (fromInp) fromInp.value = "";
                if (toInp) toInp.value = "";
                F.applyFilters(ws);
                F.setStatus(ws, "Trim cleared.");
                return;
            }
            const pickBtn = t.closest("[data-pick-range]");
            if (pickBtn) {
                S.pickRange = !S.pickRange;
                S.rangePick = null;
                pickBtn.setAttribute("aria-pressed", S.pickRange ? "true" : "false");
                pickBtn.classList.toggle("is-on", S.pickRange);
                F.setStatus(ws, S.pickRange ? "Click a start line, then an end line." : "");
                return;
            }
            const editBtn = t.closest("[data-edit]");
            if (editBtn) {
                const on = !ws.classList.contains("is-editing");
                ws.classList.toggle("is-editing", on);
                editBtn.setAttribute("aria-pressed", on ? "true" : "false");
                editBtn.classList.toggle("is-on", on);
                ws.querySelectorAll("[data-cue-text]").forEach((span) => {
            span.contentEditable = on ? "true" : "false";
            span.spellcheck = true;
        });
                F.setStatus(ws, on ? "Edit lines, then Copy or download. Changes stay on this device." : "");
                return;
            }
            const chapter = t.closest("[data-chapters] a[data-ms]");
            if (chapter) {
        e.preventDefault();
                F.mountPlayer(ws, Number(chapter.dataset.ms || 0));
                return;
            }
            const cue = t.closest(".cue");
            if (cue && ws.contains(cue)) {
                if (ws.classList.contains("is-editing") && t.closest("[data-cue-text]")) return;
                e.preventDefault();
                const ms = Number(cue.dataset.ms || 0);
                const fromInp = ws.querySelector("[data-from]");
                const toInp = ws.querySelector("[data-to]");
                const pick = ws.querySelector("[data-pick-range]");
                if (S.pickRange) {
                    if (S.rangePick == null) {
                        S.rangePick = ms;
                if (fromInp) fromInp.value = String(Math.floor(ms / 1000));
                        F.applyFilters(ws);
                        F.setStatus(ws, "Now click the last line to keep.");
                return;
            }
                    let start = S.rangePick;
            let end = ms;
                    if (end < start) { const tmp = start; start = end; end = tmp; }
                    if (fromInp) fromInp.value = String(Math.floor(start / 1000));
                    if (toInp) toInp.value = String(Math.floor(end / 1000));
                    S.rangePick = null;
                    S.pickRange = false;
                    pick?.setAttribute("aria-pressed", "false");
                    pick?.classList.remove("is-on");
                    F.applyFilters(ws);
                    F.setStatus(ws, "Range set. Copy and downloads use these lines.");
            return;
        }
                F.mountPlayer(ws, ms);
            }
        });
        document.addEventListener("change", (e) => {
            const t = e.target instanceof Element ? e.target : null;
            const sel = t?.closest("[data-vq], [data-afmt]");
            if (!sel || !(sel instanceof HTMLSelectElement)) return;
            const ws = sel.closest("#ytt-ws") || liveWorkspace();
            const vid = ws?.dataset.vid;
            if (!ws || !vid) return;
            if (sel.hasAttribute("data-vq")) {
                const a = ws.querySelector("[data-video]");
                if (a) a.setAttribute("href", `/api/video?v=${encodeURIComponent(vid)}&q=${encodeURIComponent(sel.value)}`);
            }
            if (sel.hasAttribute("data-afmt")) {
                const a = ws.querySelector("[data-audio]");
                if (a) a.setAttribute("href", `/api/audio?v=${encodeURIComponent(vid)}&fmt=${encodeURIComponent(sel.value)}`);
            }
        });
        document.addEventListener("submit", (e) => {
            const F = window.__yttForge;
            if (!F) return;
            const form = e.target;
            if (!(form instanceof HTMLFormElement)) return;
            const ws = form.closest("#ytt-ws") || liveWorkspace();
            if (!ws) return;
            if (form.classList.contains("toolbar-lang")) {
        e.preventDefault();
                F.applyLang(ws);
                return;
            }
            if (form.hasAttribute("data-another")) {
        e.preventDefault();
                const hp = form.querySelector('[name="website"]');
                if (hp && String(hp.value || "").trim()) return;
                const input = form.querySelector('input[name="url"]');
                const err = form.querySelector("[data-another-error]");
                const id = F.parseId(input?.value);
        if (!id) {
                    if (err) { err.hidden = false; err.textContent = "That does not look like a YouTube link."; }
            input?.focus();
            return;
        }
        if (err) err.hidden = true;
                F.go("/?v=" + encodeURIComponent(id) + "&mode=" + encodeURIComponent(ws.dataset.mode || "text"));
                const goBtn = form.querySelector('button[type="submit"]');
                if (goBtn) {
                    goBtn.disabled = true;
                    goBtn.setAttribute("aria-busy", "true");
                    goBtn.classList.add("is-busy");
                }
            }
        });
        document.addEventListener("input", (e) => {
            const F = window.__yttForge;
            if (!F) return;
            const t = e.target;
            if (!(t instanceof Element)) return;
            const ws = t.closest("#ytt-ws");
            if (!ws) return;
            if (t.matches("[data-search], [data-from], [data-to]")) F.applyFilters(ws);
            const span = t.closest("[data-cue-text]");
            const cueEl = span?.closest(".cue");
            if (!cueEl) return;
            const ms = Number(cueEl.dataset.ms || 0);
            const text = span.textContent || "";
            cueEl.dataset.text = text;
            const dataNow = F.wsData(ws);
            const cue = dataNow.cues.find((c) => c.start_ms === ms);
            if (cue) cue.text = text;
            const el = ws.querySelector("#ytt-data");
            if (el) el.textContent = JSON.stringify(dataNow);
            const map = {};
            for (const c of dataNow.cues) map[c.start_ms] = c.text;
            try { localStorage.setItem("youtubetotext-edits-" + (ws.dataset.vid || "") + "-" + (location.search || ""), JSON.stringify(map)); } catch (_) {}
    });
    document.addEventListener("keydown", (e) => {
            const ws = liveWorkspace();
            if (!ws) return;
        const tag = document.activeElement?.tagName;
        if (e.key === "/" && tag !== "INPUT" && tag !== "TEXTAREA" && tag !== "SELECT" && !document.activeElement?.isContentEditable) {
            e.preventDefault();
                ws.querySelector("[data-search]")?.focus();
        }
        if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
            e.preventDefault();
                ws.querySelector("[data-search]")?.focus();
            }
            if ((e.key === "Enter" || e.key === " ") && document.activeElement?.classList?.contains("cue")) {
                if (ws.classList.contains("is-editing") && document.activeElement.closest("[data-cue-text]")) return;
                e.preventDefault();
                document.activeElement.click();
            }
        });
    }
})
"##
    );

    view! {
        <div id="ytt-ws" class={ws_class} data-vid={video_id.clone()} data-lang={lang_attr} data-duration={duration_attr} data-mode={mode_slug.clone()}>
            {View::raw(format!(
                r#"<script type="application/json" id="ytt-data">{json}</script><script type="application/json" id="ytt-source">{json}</script>"#
            ))}
            <aside class="player-col">
                <div class="player-card">
                    <div class="player-facade" data-player="">
                        <button type="button" class="play-hit" data-play="" aria-label="Play video">
                            <img src={thumb} alt={thumb_alt} width="480" height="360" fetchpriority="high" decoding="async" />
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
                        <NavLink href={share} class="btn btn-ghost">"Shareable transcript"</NavLink>
                    </p>
                    <div class="media-dl">
                        <div class="media-dl-row">
                            <label class="media-dl-qwrap">
                                <span>"Format"</span>
                                <select class="media-dl-q" data-afmt="" aria-label="Audio format">
                                    <option value="m4a">"M4A"</option>
                                    <option value="mp3" selected=true>"MP3"</option>
                                    <option value="opus">"Opus"</option>
                                    <option value="wav">"WAV"</option>
                                </select>
                            </label>
                            <a class="btn btn-primary" href={audio_href.clone()} download="" data-r-full="" data-audio="">"Download audio"</a>
                        </div>
                        <div class="media-dl-row">
                            <label class="media-dl-qwrap">
                                <span>"Quality"</span>
                                <select class="media-dl-q" data-vq="" aria-label="Video quality">
                                    <option value="360">"360p"</option>
                                    <option value="480">"480p"</option>
                                    <option value="720" selected=true>"720p"</option>
                                    <option value="1080">"1080p"</option>
                                    <option value="best">"Best"</option>
                                </select>
                            </label>
                            <a class="btn btn-secondary" href={video_href} download="" data-r-full="" data-video="">"Download video"</a>
                        </div>
                    </div>
                    <Modal id="media-dl" closedBy="any" class="dl-dialog">
                        <h2 id="ytt-vdl-title" data-dl-title="">"Your file is downloading"</h2>
                        <p data-dl-lead="">"The file will save to your downloads folder."</p>
                        {crate::ads::slot("workspace-video-dl", "rectangle")}
                        <form method="dialog">
                            <button type="submit" class="btn btn-primary">"Got it"</button>
                        </form>
                    </Modal>
                    {View::raw(chapters_html)}
                    {crate::ads::slot("workspace-player", "infeed")}
                    <form class="another-form" data-another="">
                        <div class="hp-field" aria-hidden="true">
                            <label>
                                "Company website"
                                <input name="website" type="text" tabindex="-1" autocomplete="off" />
                            </label>
                        </div>
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
                                <button type="submit" class="btn btn-secondary">
                                    <span class="btn-spinner" aria-hidden="true"></span>
                                    <span>"Go"</span>
                                </button>
                            </span>
                        </label>
                        <p class="hint form-error" data-another-error="" hidden="" role="alert"></p>
                    </form>
                </div>
            </aside>
            <section class="transcript-col">
                <form class="toolbar toolbar-lang" method="get" action={action} data-lang-form="">
                    <input type="hidden" name="v" value={video_id.clone()} />
                    <input type="hidden" name="mode" value={mode_slug.clone()} />
                    <label>
                        "Captions"
                        <select name="lang" data-lang-select="">{track_options}</select>
                    </label>
                    <label>
                        "Translate"
                        <select name="tlang" data-tlang-select="">{tlang_options}</select>
                    </label>
                    <button type="button" class="btn btn-primary" data-apply="">
                        <span class="btn-spinner" aria-hidden="true"></span>
                        <span data-apply-label="">"Apply"</span>
                    </button>
                    <p class="hint apply-status" data-apply-status="" hidden="" role="status" aria-live="polite"></p>
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
                {if recap.is_empty() {
                    view! { <span class="recap-skip" hidden=""></span> }
                } else {
                    view! {
                        <section class="recap">
                            <h2>"Chapter recap from captions"</h2>
                            <p class="hint">"Extractive sentences from this transcript — not a third-party model."</p>
                            <pre class="recap-body">{recap}</pre>
                        </section>
                    }
                }}
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
