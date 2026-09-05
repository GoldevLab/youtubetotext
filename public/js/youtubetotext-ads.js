/** Request AdSense fills for reserved <ins> units. Dialogs wait until open. */

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

function bootVisible() {
  fillAds(document, false);
}

function waitForAdsense(tries) {
  if (Array.isArray(globalThis.adsbygoogle) || globalThis.adsbygoogle?.loaded) {
    bootVisible();
    return;
  }
  if (tries <= 0) {
    bootVisible();
    return;
  }
  setTimeout(() => waitForAdsense(tries - 1), 250);
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", () => waitForAdsense(20), { once: true });
} else {
  waitForAdsense(20);
}

document.addEventListener("resuma:navigate", () => {
  queueMicrotask(bootVisible);
  requestAnimationFrame(bootVisible);
  setTimeout(bootVisible, 400);
});

document.addEventListener(
  "toggle",
  (event) => {
    const t = event.target;
    if (t instanceof HTMLDialogElement && t.open) {
      fillAds(t, true);
      return;
    }
    if (!(t instanceof HTMLElement) || t.popover == null) return;
    if (t.matches(":popover-open")) fillAds(t, true);
  },
  true,
);

if ("IntersectionObserver" in window) {
  const lazyIo = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue;
        fillAds(entry.target, true);
        lazyIo.unobserve(entry.target);
      }
    },
    { rootMargin: "200px 0px" },
  );
  const watchLazy = () => {
    document.querySelectorAll("[data-ad-lazy]").forEach((el) => lazyIo.observe(el));
  };
  watchLazy();
  document.addEventListener("resuma:navigate", () => queueMicrotask(watchLazy));
}

globalThis.__yttFillAds = (el) => fillAds(el || document, true);
