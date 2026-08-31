//! Home search form — island for paste-to-go and recent links.

use resuma::prelude::*;

use crate::family::Mode;

#[island]
pub fn home_search(mode: Mode) -> View {
    let cta = mode.cta().to_string();
    let slug = mode.slug().to_string();
    visible_task!(
        r##"
(async (state, __resuma) => {
    const root = document.getElementById("ytt-home");
    if (!root || root.dataset.ready === "1") return;
    root.dataset.ready = "1";
    const form = root.querySelector("form");
    const input = root.querySelector('input[name="url"]');
    const err = root.querySelector("[data-form-error]");
    const recents = root.querySelector("[data-recents]");
    const mode = root.dataset.mode || "text";
    const KEY = "youtubetotext-recent";
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
    const loadRecent = () => {
        try { return JSON.parse(localStorage.getItem(KEY) || "[]"); } catch { return []; }
    };
    const saveRecent = (id, title) => {
        const next = [{ id, title: title || id, at: Date.now() }, ...loadRecent().filter((x) => x.id !== id)].slice(0, 8);
        localStorage.setItem(KEY, JSON.stringify(next));
    };
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
    const renderRecent = () => {
        if (!recents) return;
        recents.replaceChildren();
        const items = loadRecent();
        recents.hidden = items.length === 0;
        if (!items.length) return;
        const label = document.createElement("p");
        label.textContent = "Recent";
        const ul = document.createElement("ul");
        for (const x of items) {
            const li = document.createElement("li");
            const a = document.createElement("a");
            a.href = "/?v=" + encodeURIComponent(x.id) + "&mode=" + encodeURIComponent(mode);
            a.textContent = x.title || x.id;
            a.setAttribute("data-r-nav", "true");
            li.appendChild(a);
            ul.appendChild(li);
        }
        recents.append(label, ul);
    };
    renderRecent();
    const pasteBtn = root.querySelector("[data-paste]");
    if (pasteBtn && navigator.clipboard?.readText) {
        pasteBtn.hidden = false;
        pasteBtn.addEventListener("click", async () => {
            try {
                const t = await navigator.clipboard.readText();
                if (input) input.value = t.trim();
                input?.focus();
            } catch (_) {}
        });
    }
    form?.addEventListener("submit", async (e) => {
        const hp = form.querySelector('[name="website"]');
        if (hp && String(hp.value || "").trim()) {
            e.preventDefault();
            return;
        }
        const id = parseId(input?.value);
        if (!id) {
            e.preventDefault();
            if (err) {
                err.hidden = false;
                err.textContent = "That does not look like a YouTube link.";
            }
            input?.setAttribute("aria-invalid", "true");
            input?.focus();
            return;
        }
        e.preventDefault();
        if (err) err.hidden = true;
        input?.removeAttribute("aria-invalid");
        if (root.dataset.turnstile === "1") {
            const token = window.turnstile?.getResponse?.() || form.querySelector('[name="turnstile"]')?.value;
            if (!token) {
                if (err) { err.hidden = false; err.textContent = "Confirm you are not a bot, then try again."; }
                return;
            }
            try {
                const r = await fetch("/api/gate", {
                    method: "POST",
                    headers: { "Content-Type": "application/json", Accept: "application/json" },
                    body: JSON.stringify({ token }),
                });
                if (!r.ok) {
                    const data = await r.json().catch(() => ({}));
                    if (err) { err.hidden = false; err.textContent = data.error || "Confirm you are not a bot, then try again."; }
                    return;
                }
            } catch (_) {
                if (err) { err.hidden = false; err.textContent = "Confirm you are not a bot, then try again."; }
                return;
            }
        }
        form?.classList.add("is-busy");
        saveRecent(id, id);
        go("/?v=" + encodeURIComponent(id) + "&mode=" + encodeURIComponent(mode));
    });
})
"##
    );

    let site = crate::guard::turnstile_site_key().unwrap_or_default();
    let ts_flag = if site.is_empty() { String::new() } else { "1".into() };
    let ts_block = if site.is_empty() {
        View::empty()
    } else {
        View::raw(format!(
            r#"<div class="cf-turnstile" data-sitekey="{}"></div><script src="https://challenges.cloudflare.com/turnstile/v0/api.js" async defer></script>"#,
            html_escape::encode_double_quoted_attribute(&site)
        ))
    };

    view! {
        <div id="ytt-home" data-mode={slug.clone()} data-turnstile={ts_flag}>
            <Form submit={crate::actions::open_transcript} class="hero-form">
                <input type="hidden" name="mode" value={slug} />
                <input type="hidden" name="turnstile" value="" />
                <label class="hp-field" aria-hidden="true">
                    "Company website"
                    <input
                        name="website"
                        type="text"
                        tabindex="-1"
                        autocomplete="off"
                    />
                </label>
                <label class="hero-label">
                    "YouTube link"
                    <span class="hero-field">
                        <input
                            name="url"
                            type="text"
                            inputmode="url"
                            enterkeyhint="go"
                            autocomplete="url"
                            spellcheck="false"
                            required=true
                            placeholder="https://www.youtube.com/watch?v=…"
                            aria-describedby="url-help url-error"
                        />
                        <button type="button" class="btn btn-ghost" data-paste="" hidden="">"Paste"</button>
                        <button type="submit" class="btn btn-primary">{cta}</button>
                    </span>
                </label>
                {ts_block}
                <p id="url-help" class="hint">"Works with watch, shorts, youtu.be, and a bare video id. No account."</p>
                <p id="url-error" class="hint form-error" data-form-error="" hidden="" role="alert"></p>
            </Form>
            <div class="recents" data-recents="" hidden=""></div>
        </div>
    }
}
