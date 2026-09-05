(() => {
  let stopHero = null;
  const mountHeroParticles = () => {
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
    const COUNT = 160;
    const dots = Array.from({ length: COUNT }, () => ({
      x: Math.random(),
      y: Math.random(),
      z: Math.random(),
      s: 0.4 + Math.random() * 1.4,
      p: Math.random() * Math.PI * 2,
    }));
    let w = 0;
    let h = 0;
    let pointerX = 0;
    let pointerY = 0;
    let raf = 0;
    let accent = "#ff2d20";
    const readAccent = () => {
      const cs = getComputedStyle(document.documentElement);
      accent = (cs.getPropertyValue("--accent") || "#ff2d20").trim() || "#ff2d20";
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
      if (!r.width || !r.height) return;
      pointerX = (e.clientX - r.left) / r.width - 0.5;
      pointerY = (e.clientY - r.top) / r.height - 0.5;
    };
    const tick = (t) => {
      if (!root.isConnected) {
        if (typeof stopHero === "function") stopHero();
        return;
      }
      if (document.hidden) {
        raf = 0;
        return;
      }
      const time = t * 0.001;
      ctx.clearRect(0, 0, w, h);
      ctx.fillStyle = accent;
      for (const d of dots) {
        d.p += 0.002 * d.s;
        const x = (d.x + Math.sin(time * d.s + d.p) * 0.03 + pointerX * 0.04 * d.z) * w;
        const y = (d.y + Math.cos(time * 0.7 * d.s + d.p) * 0.025 + pointerY * 0.03 * d.z) * h;
        ctx.globalAlpha = 0.18 + d.z * 0.35;
        ctx.beginPath();
        ctx.arc(x, y, 0.6 + d.z * 1.6, 0, Math.PI * 2);
        ctx.fill();
      }
      raf = requestAnimationFrame(tick);
    };
    const onVis = () => {
      if (!document.hidden && !raf && root.isConnected) raf = requestAnimationFrame(tick);
    };
    const themeWatch = new MutationObserver(readAccent);
    themeWatch.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });
    readAccent();
    resize();
    window.addEventListener("resize", resize, { passive: true });
    window.addEventListener("pointermove", onMove, { passive: true });
    document.addEventListener("visibilitychange", onVis);
    raf = requestAnimationFrame(tick);
    stopHero = () => {
      cancelAnimationFrame(raf);
      raf = 0;
      themeWatch.disconnect();
      window.removeEventListener("resize", resize);
      window.removeEventListener("pointermove", onMove);
      document.removeEventListener("visibilitychange", onVis);
      canvas.remove();
      stopHero = null;
    };
  };
  const tryHero = () => mountHeroParticles();
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", tryHero, { once: true });
  } else {
    tryHero();
  }
  document.addEventListener("resuma:navigate", () => requestAnimationFrame(tryHero));
})();
