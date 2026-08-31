(function () {
  const root = document.querySelector("[data-fetch-failed]");
  if (!root || root.dataset.rescue === "1") return;
  root.dataset.rescue = "1";
  const status = root.querySelector("[data-rescue-status]");
  const vid = root.dataset.vid;
  if (!vid || !/^[\w-]{11}$/.test(vid)) return;
  if (status) {
    status.hidden = false;
    status.textContent = "Extension: fetching captions with this browser…";
  }
  chrome.runtime.sendMessage(
    {
      type: "fetchTranscript",
      videoId: vid,
      lang: root.dataset.lang || "",
      tlang: root.dataset.tlang || "",
    },
    async (res) => {
      if (!res?.ok) {
        if (status) {
          status.textContent = res?.error
            || "This browser could not download captions either. The video may have none.";
        }
        return;
      }
      try {
        const r = await fetch("/api/ingest", {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(res.doc),
        });
        if (!r.ok) throw new Error("ingest");
        location.reload();
      } catch {
        if (status) status.textContent = "Got captions but could not store them on YouTubeForge.";
      }
    }
  );
})();
