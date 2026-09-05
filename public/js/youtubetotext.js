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
  const tryHero = () => mountCaptionStream();
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", tryHero, { once: true });
  } else {
    tryHero();
  }
  document.addEventListener("resuma:navigate", () => requestAnimationFrame(tryHero));
})();
