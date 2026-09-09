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
          ? "The file will save to your downloads folder. MP3 conversion can take a moment."
          : "The file will save to your downloads folder. Higher qualities can take a minute.";
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

  /** Keep the page; never navigate to bare /api/video. */
  const startIframeDownload = (href) => {
    if (!href || !/[?&]v=/.test(href)) return false;
    const frame = document.createElement("iframe");
    frame.hidden = true;
    frame.setAttribute("aria-hidden", "true");
    frame.src = href;
    document.body.append(frame);
    setTimeout(() => frame.remove(), 180000);
    return true;
  };

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
        startIframeDownload(`/api/audio?v=${encodeURIComponent(id)}&fmt=mp3`);
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
      const q = root?.querySelector("[data-vq]")?.value || "720";
      const afmt = root?.querySelector("[data-afmt]")?.value || "mp3";
      const kind = videoBtn ? "video" : "audio";
      const href = videoBtn
        ? `/api/video?v=${encodeURIComponent(id)}&q=${encodeURIComponent(q)}`
        : `/api/audio?v=${encodeURIComponent(id)}&fmt=${encodeURIComponent(afmt)}`;
      void showDlDialog(root, kind);
      startIframeDownload(href);
    },
    true,
  );
})();
