(() => {
  let stopHero = null;
  const mountCaptionStream = () => {
    if (typeof stopHero === "function") {
      stopHero();
      stopHero = null;
    }
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    const root = document.querySelector("[data-hero-particles]");
    if (!root) return;
    const canvas = document.createElement("canvas");
    canvas.className = "hero-particles-canvas";
    canvas.setAttribute("aria-hidden", "true");
    root.replaceChildren(canvas);
    const ctx = canvas.getContext("2d", { alpha: true });
    if (!ctx) return;

    const COUNT = window.matchMedia("(max-width: 640px)").matches ? 28 : 46;
    const items = Array.from({ length: COUNT }, () => {
      const roll = Math.random();
      const kind = roll < 0.58 ? "cue" : roll < 0.88 ? "tick" : "play";
      return {
        kind,
        x: Math.random() * 1.2 - 0.1,
        y: 0.14 + Math.random() * 0.72,
        z: 0.28 + Math.random() * 0.72,
        len: kind === "cue" ? 0.07 + Math.random() * 0.16 : 0,
        vx: -(0.012 + Math.random() * 0.028),
      };
    });

    let w = 0;
    let h = 0;
    let pointerX = 0;
    let raf = 0;
    let last = 0;
    let accent = "#ff2d20";
    let ink = "#ff8f88";

    const readColors = () => {
      const cs = getComputedStyle(document.documentElement);
      accent = (cs.getPropertyValue("--accent") || "#ff2d20").trim() || "#ff2d20";
      ink = (cs.getPropertyValue("--primary") || "#ff8f88").trim() || "#ff8f88";
    };
    const resize = () => {
      const dpr = Math.min(window.devicePixelRatio || 1, 2);
      w = root.clientWidth;
      h = root.clientHeight;
      if (!w || !h) return;
      canvas.width = Math.floor(w * dpr);
      canvas.height = Math.floor(h * dpr);
      canvas.style.width = w + "px";
      canvas.style.height = h + "px";
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    };
    const onMove = (e) => {
      const r = root.getBoundingClientRect();
      if (!r.width) return;
      pointerX = (e.clientX - r.left) / r.width - 0.5;
    };
    const roundChip = (x, y, bw, bh) => {
      const r = Math.min(bh / 2, 4);
      if (ctx.roundRect) {
        ctx.beginPath();
        ctx.roundRect(x, y, bw, bh, r);
        ctx.fill();
      } else {
        ctx.fillRect(x, y, bw, bh);
      }
    };
    const tick = (t) => {
      if (!root.isConnected) {
        if (typeof stopHero === "function") stopHero();
        return;
      }
      if (document.hidden) {
        raf = 0;
        last = 0;
        return;
      }
      const dt = Math.min(0.032, last ? (t - last) / 1000 : 0.016);
      last = t;
      ctx.clearRect(0, 0, w, h);

      // Playhead — a thin vertical seek line that follows the pointer.
      ctx.globalAlpha = 0.18;
      ctx.fillStyle = accent;
      ctx.fillRect((0.5 + pointerX * 0.55) * w, h * 0.08, 1.5, h * 0.84);

      for (const d of items) {
        d.x += (d.vx + pointerX * 0.04 * d.z) * dt * 14;
        if (d.x < -0.22) {
          d.x = 1.12;
          d.y = 0.14 + Math.random() * 0.72;
        } else if (d.x > 1.18) {
          d.x = -0.1;
        }
        const x = d.x * w;
        const y = d.y * h;
        ctx.globalAlpha = 0.2 + d.z * 0.42;
        if (d.kind === "cue") {
          ctx.fillStyle = ink;
          roundChip(x, y, d.len * w, 5 + d.z * 4);
        } else if (d.kind === "tick") {
          ctx.fillStyle = accent;
          ctx.fillRect(x, y - 5 - d.z * 6, 1.2, 10 + d.z * 10);
        } else {
          const s = 4 + d.z * 4;
          ctx.fillStyle = accent;
          ctx.beginPath();
          ctx.moveTo(x, y - s);
          ctx.lineTo(x + s * 1.35, y);
          ctx.lineTo(x, y + s);
          ctx.closePath();
          ctx.fill();
        }
      }
      raf = requestAnimationFrame(tick);
    };
    const onVis = () => {
      if (!document.hidden && !raf && root.isConnected) raf = requestAnimationFrame(tick);
    };
    const themeWatch = new MutationObserver(readColors);
    themeWatch.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });
    readColors();
    resize();
    window.addEventListener("resize", resize, { passive: true });
    window.addEventListener("pointermove", onMove, { passive: true });
    document.addEventListener("visibilitychange", onVis);
    raf = requestAnimationFrame(tick);
    stopHero = () => {
      cancelAnimationFrame(raf);
      raf = 0;
      last = 0;
      themeWatch.disconnect();
      window.removeEventListener("resize", resize);
      window.removeEventListener("pointermove", onMove);
      document.removeEventListener("visibilitychange", onVis);
      canvas.remove();
      stopHero = null;
    };
  };
  const tryHero = () => {
    const start = () => mountCaptionStream();
    if ("requestIdleCallback" in window) {
      requestIdleCallback(start, { timeout: 4000 });
    } else {
      setTimeout(start, 1200);
    }
  };
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", tryHero, { once: true });
  } else {
    tryHero();
  }
  document.addEventListener("resuma:navigate", () => requestAnimationFrame(tryHero));

  /* Meta Pixel + CAPI ViewContent (shared eventID). Pixel boot defines __yttMetaViewContent. */
  const trackMetaNav = () => {
    try {
      if (typeof window.__yttMetaViewContent === "function") {
        window.__yttMetaViewContent();
        return;
      }
      if (typeof window.fbq !== "function") return;
      window.fbq("track", "PageView");
      const v = new URLSearchParams(location.search).get("v");
      if (!v) return;
      const eid =
        (crypto.randomUUID && crypto.randomUUID()) ||
        `vc_${Date.now().toString(36)}${Math.random().toString(36).slice(2, 10)}`;
      window.fbq(
        "track",
        "ViewContent",
        { content_name: "transcript", content_type: "product", content_ids: [v] },
        { eventID: eid },
      );
      fetch("/api/meta/view-content", {
        method: "POST",
        headers: { "content-type": "application/json" },
        credentials: "same-origin",
        keepalive: true,
        body: JSON.stringify({
          event_id: eid,
          v,
          event_source_url: String(location.href || "").slice(0, 2048),
        }),
      }).catch(() => {});
    } catch (_) {}
  };
  document.addEventListener("resuma:navigate", () => {
    if (typeof window.fbq === "function") {
      try {
        window.fbq("track", "PageView");
      } catch (_) {}
    }
    queueMicrotask(trackMetaNav);
  });

  const readClipboardText = async () => {
    try {
      if (navigator.clipboard?.readText) {
        const t = await navigator.clipboard.readText();
        const text = String(t || "").trim();
        if (text) return text;
      }
    } catch (_) {}
    try {
      if (navigator.clipboard?.read) {
        const items = await navigator.clipboard.read();
        for (const item of items) {
          if (!item.types.includes("text/plain")) continue;
          const blob = await item.getType("text/plain");
          const text = String((await blob.text()) || "").trim();
          if (text) return text;
        }
      }
    } catch (_) {}
    return "";
  };

  /* Capture-phase Paste so mobile taps work before the home island chunk loads.
     iOS/Android often deny clipboard.readText — then focus the field for OS paste. */
  document.addEventListener(
    "click",
    async (e) => {
      const t = e.target instanceof Element ? e.target : null;
      const btn = t?.closest("[data-paste]");
      if (!btn) return;
      e.preventDefault();
      e.stopPropagation();
      e.stopImmediatePropagation();

      const root =
        btn.closest("#ytt-home") ||
        btn.closest("form") ||
        document.getElementById("ytt-home");
      const input =
        root?.querySelector('input[name="url"]') ||
        document.querySelector('input[name="url"]');
      const err =
        root?.querySelector("[data-form-error]") ||
        document.querySelector("[data-form-error]");

      const showHint = (msg) => {
        if (!err) return;
        err.hidden = false;
        err.textContent = msg;
        err.scrollIntoView?.({ block: "nearest", behavior: "smooth" });
        window.clearTimeout(showHint._t);
        showHint._t = window.setTimeout(() => {
          if (err.textContent === msg) {
            err.hidden = true;
            err.textContent = "";
          }
        }, 5000);
      };

      btn.disabled = true;
      try {
        const text = await readClipboardText();
        if (text && input) {
          input.value = text;
          input.dispatchEvent(new Event("input", { bubbles: true }));
          input.removeAttribute("aria-invalid");
          if (err) {
            err.hidden = true;
            err.textContent = "";
          }
          input.focus({ preventScroll: false });
          return;
        }
      } finally {
        btn.disabled = false;
      }

      if (input) {
        input.focus({ preventScroll: false });
        try {
          input.select?.();
        } catch (_) {}
      }
      const coarse =
        window.matchMedia("(pointer: coarse)").matches ||
        navigator.maxTouchPoints > 0;
      showHint(
        coarse
          ? "Long-press the link field, then tap Paste."
          : "Clipboard blocked — paste into the link field (Ctrl/⌘+V).",
      );
    },
    true,
  );

  const parseYouTubeId = (raw) => {
    const s = String(raw || "").trim();
    if (/^[\w-]{11}$/.test(s)) return s;
    try {
      const u = new URL(s.startsWith("http") ? s : "https://" + s);
      const v = u.searchParams.get("v") || u.searchParams.get("vi");
      if (v && /^[\w-]{11}$/.test(v)) return v;
      const parts = u.pathname.split("/").filter(Boolean);
      const host = u.hostname.replace(/^www\./, "");
      if (host === "youtu.be" && /^[\w-]{11}$/.test(parts[0] || "")) return parts[0];
      const i = parts.findIndex((p) => ["embed", "shorts", "live", "v", "watch"].includes(p));
      const id = i >= 0 ? (parts[i + 1] || "").slice(0, 11) : "";
      if (/^[\w-]{11}$/.test(id)) return id;
    } catch (_) {}
    return null;
  };

  const armDlDialog = (dlg) => {
    if (!(dlg instanceof HTMLDialogElement) || dlg.dataset.ready) return;
    dlg.dataset.ready = "1";
    if (!("closedBy" in HTMLDialogElement.prototype)) {
      dlg.addEventListener("click", (event) => {
        if (event.target !== dlg) return;
        const rect = dlg.getBoundingClientRect();
        const inside =
          rect.top <= event.clientY &&
          event.clientY <= rect.top + rect.height &&
          rect.left <= event.clientX &&
          event.clientX <= rect.left + rect.width;
        if (!inside) dlg.close();
      });
    }
  };

  const showDlDialog = async (root, kind) => {
    const dlg = root?.querySelector("#r-modal-media-dl") || root?.querySelector(".dl-dialog");
    armDlDialog(dlg);
    if (!(dlg instanceof HTMLDialogElement)) return;
    const title = dlg.querySelector("[data-dl-title]") || dlg.querySelector("h2");
    const lead = dlg.querySelector("[data-dl-lead]") || dlg.querySelector("p");
    if (title) title.textContent = kind === "audio" ? "Your audio is downloading" : "Your video is downloading";
    if (lead) {
      lead.textContent =
        kind === "audio"
          ? "The file will save to your downloads folder. Long talks / MP3 conversion can take a few minutes — keep this tab open."
          : "The file will save to your downloads folder. 1–2 hour videos at 1080p/4K can take several minutes to start. If it stalls, try 360p or 480p.";
    }
    try {
      const open = globalThis.__resuma?.showModal?.("media-dl");
      if (open && typeof open.then === "function") await open;
      else if (typeof dlg.showModal === "function" && !dlg.open) dlg.showModal();
    } catch (_) {
      if (typeof dlg.showModal === "function" && !dlg.open) dlg.showModal();
    }
    try {
      globalThis.__yttFillAds?.(dlg);
    } catch (_) {}
  };

  const closeDlDialog = (root) => {
    try {
      globalThis.__resuma?.closeModal?.("media-dl");
    } catch (_) {}
    const dlg = root?.querySelector("#r-modal-media-dl") || root?.querySelector(".dl-dialog");
    if (dlg instanceof HTMLDialogElement && dlg.open) {
      try {
        dlg.close();
      } catch (_) {}
    }
  };

  /** Keep the page; never navigate to bare /api/video. Surface JSON errors from the iframe. */
  const readIframeJsonError = (frame) => {
    try {
      const doc = frame.contentDocument;
      if (!doc) return null;
      const text = (doc.body?.innerText || doc.body?.textContent || "").trim();
      if (!text) return null;
      const data = JSON.parse(text);
      if (data && typeof data.error === "string" && data.error) {
        const err = new Error(data.error);
        err.retryAfter = data.retry_after;
        return err;
      }
    } catch (_) {}
    return null;
  };

  const startIframeDownload = (href, onError) => {
    if (!href || !/[?&]v=/.test(href)) return false;
    const frame = document.createElement("iframe");
    frame.hidden = true;
    frame.setAttribute("aria-hidden", "true");
    let reported = false;
    const report = (err) => {
      if (reported || !err) return;
      reported = true;
      if (typeof onError === "function") onError(err);
      frame.remove();
    };
    const probe = () => {
      const err = readIframeJsonError(frame);
      if (err) report(err);
    };
    frame.addEventListener("load", probe);
    const iv = setInterval(probe, 1500);
    frame.src = href;
    document.body.append(frame);
    setTimeout(() => {
      clearInterval(iv);
      if (!reported) frame.remove();
    }, 180000);
    return true;
  };

  const sha256hex = async (text) => {
    const buf = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(text));
    return [...new Uint8Array(buf)].map((b) => b.toString(16).padStart(2, "0")).join("");
  };

  const solvePow = async (salt, challenge, max) => {
    const cap = Number(max) || 0;
    const want = String(challenge || "").toLowerCase();
    for (let n = 0; n <= cap; n++) {
      if ((await sha256hex(salt + String(n))) === want) return n;
      if (n % 96 === 0) await new Promise((r) => setTimeout(r, 0));
    }
    throw new Error("Could not confirm you are not a bot. Try again.");
  };

  let gateCap = "";
  let gateUntil = 0;
  const ensureGate = async (kind) => {
    const want = kind === "hd" ? "hd" : "media";
    const now = Date.now();
    if (now < gateUntil) {
      if (want === "media" && (gateCap === "media" || gateCap === "hd")) return true;
      if (want === "hd" && gateCap === "hd") return true;
    } else {
      gateCap = "";
    }
    try {
      const chRes = await fetch("/api/challenge?k=" + encodeURIComponent(want), {
        credentials: "same-origin",
        headers: { Accept: "application/json" },
      });
      const ch = await chRes.json().catch(() => ({}));
      if (!chRes.ok) {
        const err = new Error(ch.error || "Confirm you are not a bot, then try again.");
        err.retryAfter = ch.retry_after;
        throw err;
      }
      const number = await solvePow(ch.salt, ch.challenge, ch.maxnumber);
      const gRes = await fetch("/api/gate", {
        method: "POST",
        credentials: "same-origin",
        headers: { "Content-Type": "application/json", Accept: "application/json" },
        body: JSON.stringify({
          salt: ch.salt,
          challenge: ch.challenge,
          number,
          maxnumber: ch.maxnumber,
          signature: ch.signature,
          exp: ch.exp,
          kind: ch.kind || want,
        }),
      });
      const g = await gRes.json().catch(() => ({}));
      if (!gRes.ok) {
        const err = new Error(g.error || "Confirm you are not a bot, then try again.");
        err.retryAfter = g.retry_after;
        throw err;
      }
      gateCap = g.hd ? "hd" : "media";
      const ttl = Math.max(30, Number(g.expires_in) || 600) - 30;
      gateUntil = Date.now() + ttl * 1000;
      return true;
    } catch (e) {
      gateCap = "";
      gateUntil = 0;
      throw e;
    }
  };

  const dryMedia = async (href) => {
    const u = href.includes("?") ? href + "&dry=1" : href + "?dry=1";
    const r = await fetch(u, {
      credentials: "same-origin",
      headers: { Accept: "application/json" },
    });
    const data = await r.json().catch(() => ({}));
    if (!r.ok) {
      const err = new Error(data.error || "Download is not available right now.");
      err.retryAfter = data.retry_after;
      err.status = r.status;
      throw err;
    }
    return true;
  };

  const isHdQ = (q) => ["1080", "1440", "2160", "best"].includes(String(q || ""));

  const runMediaDownload = async ({ href, kind, q, root, onError }) => {
    const fail = (msg) => {
      if (typeof onError === "function") onError(msg);
      else if (root) {
        const err = root.querySelector("[data-form-error]");
        if (err) {
          err.hidden = false;
          err.textContent = msg;
        }
      }
    };
    try {
      const needHd = kind === "video" && isHdQ(q);
      if (needHd) {
        const ok = window.confirm(
          "1080p and 4K are limited to one download per day from this network, and need a short extra check. Continue?",
        );
        if (!ok) return false;
      }
      try {
        await dryMedia(href);
      } catch (e) {
        if (e && e.status === 403) {
          await ensureGate(needHd ? "hd" : "media");
          await dryMedia(href);
        } else {
          throw e;
        }
      }
      await showDlDialog(root, kind);
      startIframeDownload(href, (e) => {
        closeDlDialog(root);
        fail(e && e.message ? e.message : "Download is not available right now.");
      });
      return true;
    } catch (e) {
      fail(e && e.message ? e.message : "Download is not available right now.");
      return false;
    }
  };

  window.__yttGate = { ensure: ensureGate, download: runMediaDownload, solvePow };

  document.addEventListener(
    "click",
    (e) => {
      const t = e.target instanceof Element ? e.target : null;
      if (!t?.closest) return;

      const failAudio = t.closest("[data-fail-audio]");
      if (failAudio) {
        e.preventDefault();
        e.stopPropagation();
        const id = failAudio.getAttribute("data-vid") || "";
        if (!/^[\w-]{11}$/.test(id)) return;
        const href = `/api/audio?v=${encodeURIComponent(id)}&fmt=mp3`;
        void runMediaDownload({ href, kind: "audio", q: "", root: document.getElementById("ytt-home") });
        return;
      }

      const videoBtn = t.closest("[data-home-video]");
      const audioBtn = t.closest("[data-home-audio]");
      if (!videoBtn && !audioBtn) return;
      e.preventDefault();
      e.stopPropagation();
      const root = document.getElementById("ytt-home") || t.closest("#ytt-home");
      const input = root?.querySelector('input[name="url"]');
      const err = root?.querySelector("[data-form-error]");
      const id = parseYouTubeId(input?.value);
      if (!id) {
        if (err) {
          err.hidden = false;
          err.textContent = "Paste a YouTube link first.";
        }
        input?.setAttribute("aria-invalid", "true");
        input?.focus();
        return;
      }
      if (err) err.hidden = true;
      input?.removeAttribute("aria-invalid");
      const q = root?.querySelector("[data-vq]")?.value || "480";
      const afmt = root?.querySelector("[data-afmt]")?.value || "mp3";
      const kind = videoBtn ? "video" : "audio";
      const href = videoBtn
        ? `/api/video?v=${encodeURIComponent(id)}&q=${encodeURIComponent(q)}`
        : `/api/audio?v=${encodeURIComponent(id)}&fmt=${encodeURIComponent(afmt)}`;
      void runMediaDownload({
        href,
        kind,
        q,
        root,
        onError: (msg) => {
          if (err) {
            err.hidden = false;
            err.textContent = msg;
          }
        },
      });
    },
    true,
  );

  const runWelcomeReveal = async () => {
    const root = document.querySelector("[data-forge-welcome]");
    if (!root) return;
    const t = root.getAttribute("data-forge-token") || "";
    if (!t.startsWith("ft_")) return;
    const status = root.querySelector("[data-welcome-status]");
    const keyEl = root.querySelector("[data-welcome-key]");
    const usage = root.querySelector("[data-welcome-usage]");
    const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
    for (let i = 0; i < 20; i++) {
      try {
        const r = await fetch("/api/billing/reveal?t=" + encodeURIComponent(t), {
          headers: { Accept: "application/json" },
          credentials: "same-origin",
        });
        const data = await r.json().catch(() => ({}));
        if (data.api_key && keyEl) {
          keyEl.hidden = false;
          keyEl.textContent = data.api_key;
          if (status) status.textContent = data.message || "Copy this key now. It will not be shown again.";
          if (usage && data.usage) {
            usage.hidden = false;
            usage.textContent =
              "Plan: " +
              (data.plan || "") +
              " · transcripts " +
              (data.usage.transcripts?.used ?? 0) +
              "/" +
              (data.usage.transcripts?.cap ?? "?");
          }
          return;
        }
        if (status) status.textContent = data.message || data.error || "Still provisioning…";
        if (!data.pending && r.status === 404) return;
      } catch (_) {
        if (status) status.textContent = "Network error — retrying…";
      }
      await sleep(1500);
    }
    if (status) {
      status.textContent =
        "Timed out waiting for the key. Check your Lemon receipt email, then refresh this page.";
    }
  };
  runWelcomeReveal();
  document.addEventListener("resuma:navigate", () => requestAnimationFrame(runWelcomeReveal));
})();
