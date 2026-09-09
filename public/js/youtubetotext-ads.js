/** Lazy-load AdSense after first paint, then fill reserved <ins> units. */

function adsenseClient() {
  const meta = document.querySelector('meta[name="ytt-adsense-client"]');
  const id = meta?.getAttribute("content")?.trim();
  return id && id.startsWith("ca-pub-") ? id : "";
}

function ensureAdsenseScript() {
  const id = adsenseClient();
  if (!id) return Promise.resolve(false);
  if (document.querySelector('script[src*="adsbygoogle.js"]')) {
    return Promise.resolve(true);
  }
  return new Promise((resolve) => {
    const s = document.createElement("script");
    s.async = true;
    s.crossOrigin = "anonymous";
    s.src = `https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js?client=${encodeURIComponent(id)}`;
    s.onload = () => resolve(true);
    s.onerror = () => resolve(false);
    document.head.appendChild(s);
  });
}

function pendingIns(scope, includeLazy) {
  const root = scope instanceof Element ? scope : document;
  return [...root.querySelectorAll("ins.adsbygoogle[data-ad-client][data-ad-slot]")].filter((ins) => {
    if (ins.getAttribute("data-adsbygoogle-status")) return false;
    const lazyHost = ins.closest("[data-ad-lazy]");
    if (lazyHost && !includeLazy) return false;
    const dialog = ins.closest("dialog");
    if (dialog && !dialog.open && !includeLazy) return false;
    return true;
  });
}

function pushFill(ins) {
  try {
    (globalThis.adsbygoogle = globalThis.adsbygoogle || []).push({});
    return true;
  } catch (_) {
    return false;
  }
}

function fillAds(scope, includeLazy) {
  const nodes = pendingIns(scope, includeLazy);
  if (!nodes.length) return;
  if (!globalThis.adsbygoogle && !document.querySelector('script[src*="adsbygoogle.js"]')) {
    return;
  }
  for (const ins of nodes) pushFill(ins);
}

async function bootVisible() {
  await ensureAdsenseScript();
  fillAds(document, false);
}

function scheduleBoot() {
  const run = () => {
    bootVisible().catch(() => {});
  };
  if ("requestIdleCallback" in window) {
    requestIdleCallback(run, { timeout: 2500 });
  } else {
    setTimeout(run, 1200);
  }
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", scheduleBoot, { once: true });
} else {
  scheduleBoot();
}

document.addEventListener("resuma:navigate", () => {
  queueMicrotask(() => {
    bootVisible().catch(() => {});
  });
  setTimeout(() => {
    bootVisible().catch(() => {});
  }, 400);
});

document.addEventListener(
  "toggle",
  (event) => {
    const t = event.target;
    if (t instanceof HTMLDialogElement && t.open) {
      ensureAdsenseScript().then(() => fillAds(t, true));
      return;
    }
    if (!(t instanceof HTMLElement) || t.popover == null) return;
    if (t.matches(":popover-open")) {
      ensureAdsenseScript().then(() => fillAds(t, true));
    }
  },
  true,
);

if ("IntersectionObserver" in window) {
  const lazyIo = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue;
        ensureAdsenseScript().then(() => fillAds(entry.target, true));
        lazyIo.unobserve(entry.target);
      }
    },
    { rootMargin: "200px 0px" },
  );
  const observeLazy = () => {
    document.querySelectorAll("[data-ad-lazy]").forEach((el) => lazyIo.observe(el));
  };
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", observeLazy, { once: true });
  } else {
    observeLazy();
  }
  document.addEventListener("resuma:navigate", () => queueMicrotask(observeLazy));
}
