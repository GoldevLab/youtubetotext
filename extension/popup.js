import { SITE } from "./config.js";
import { videoIdFromUrl } from "./youtube-fetch.js";

const run = document.getElementById("run");
const home = document.getElementById("home");

home.addEventListener("click", () => chrome.tabs.create({ url: `${SITE}/` }));

run.addEventListener("click", async () => {
  run.disabled = true;
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  const id = videoIdFromUrl(tab?.url || "");
  if (!id) {
    await chrome.tabs.create({ url: `${SITE}/` });
    window.close();
    return;
  }
  chrome.runtime.sendMessage({ type: "fetchTranscript", videoId: id }, async (res) => {
    let dest = `${SITE}/?v=${id}&mode=text`;
    if (res?.ok) {
      try {
        const r = await fetch(`${SITE}/api/ingest`, {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(res.doc),
        });
        if (r.ok) dest = `${SITE}/?v=${id}&mode=text`;
      } catch {
        /* open anyway */
      }
    }
    await chrome.tabs.create({ url: dest });
    window.close();
  });
});
