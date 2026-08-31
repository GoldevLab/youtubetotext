/** Request AdSense fills for reserved <ins> units. Dialogs wait until open. */

function fillAds(scope, includeLazy) {
  const root = scope instanceof Element ? scope : document;
  const nodes = root.querySelectorAll("ins.adsbygoogle[data-ad-client][data-ad-slot]");
  for (const ins of nodes) {
    if (ins.getAttribute("data-adsbygoogle-status")) continue;
    const lazyHost = ins.closest("[data-ad-lazy]");
    if (lazyHost && !includeLazy) continue;
    const dialog = ins.closest("dialog");
    if (dialog && !dialog.open && !includeLazy) continue;
    try {
      (globalThis.adsbygoogle = globalThis.adsbygoogle || []).push({});
    } catch (_) {}
  }
}

function bootVisible() {
  fillAds(document, false);
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", bootVisible, { once: true });
} else {
  bootVisible();
}

document.addEventListener("resuma:navigate", () => {
  queueMicrotask(bootVisible);
  requestAnimationFrame(bootVisible);
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

globalThis.__yttFillAds = (el) => fillAds(el || document, true);
