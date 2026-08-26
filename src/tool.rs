//! Home search form — island for paste-to-go and recent links.

use resuma::prelude::*;

#[island]
pub fn home_search() -> View {
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
            a.href = "/v/" + encodeURIComponent(x.id);
            a.textContent = x.title || x.id;
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
    form?.addEventListener("submit", (e) => {
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
        saveRecent(id, id);
        location.assign("/v/" + encodeURIComponent(id));
    });
})
"##
    );

    view! {
        <div id="ytt-home">
            <Form submit={crate::actions::open_transcript} class="hero-form">
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
                        <button type="submit" class="btn btn-primary">"Get transcript"</button>
                    </span>
                </label>
                <p id="url-help" class="hint">"Works with watch, shorts, youtu.be, and a bare video id. No account."</p>
                <p id="url-error" class="hint form-error" data-form-error="" hidden="" role="alert"></p>
            </Form>
            <div class="recents" data-recents="" hidden=""></div>
        </div>
    }
}
