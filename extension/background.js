import { fetchTranscript } from "./youtube-fetch.js";

chrome.runtime.onMessage.addListener((msg, _sender, sendResponse) => {
  if (msg?.type !== "fetchTranscript") return;
  fetchTranscript(msg.videoId, msg.lang || "", msg.tlang || "")
    .then((doc) => sendResponse({ ok: true, doc }))
    .catch((e) => sendResponse({ ok: false, error: e.message || String(e) }));
  return true;
});
