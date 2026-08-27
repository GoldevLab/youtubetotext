import { fetchTranscript, videoIdFromUrl } from "./youtube-fetch.js";

const SITE = "https://youtubetotext.fly.dev";

chrome.runtime.onMessage.addListener((msg, _sender, sendResponse) => {
  if (msg?.type !== "fetchTranscript") return;
  fetchTranscript(msg.videoId, msg.lang || "", msg.tlang || "")
    .then((doc) => sendResponse({ ok: true, doc }))
    .catch((e) => sendResponse({ ok: false, error: e.message || String(e) }));
  return true;
});

chrome.action.onClicked.addListener(async (tab) => {
  const url = tab?.url || "";
  const id = videoIdFromUrl(url);
  if (!id) {
    await chrome.tabs.create({ url: `${SITE}/` });
    return;
  }
  try {
    const doc = await fetchTranscript(id, "", "");
    const origins = ["http://127.0.0.1:3010", "https://youtubetotext.fly.dev"];
    let opened = false;
    for (const origin of origins) {
      try {
        const r = await fetch(`${origin}/api/ingest`, {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(doc),
        });
        if (r.ok) {
          await chrome.tabs.create({ url: `${origin}/v/${id}` });
          opened = true;
          break;
        }
      } catch {
        /* try next origin */
      }
    }
    if (!opened) await chrome.tabs.create({ url: `https://youtubetotext.fly.dev/v/${id}` });
  } catch {
    await chrome.tabs.create({
      url: `${SITE}/v/${id}`,
    });
  }
});
