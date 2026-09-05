function videoIdFromLocation() {
  try {
    const u = new URL(location.href);
    const v = u.searchParams.get("v");
    if (v && /^[\w-]{11}$/.test(v)) return v;
    const parts = u.pathname.split("/").filter(Boolean);
    if (parts[0] === "shorts" && /^[\w-]{11}$/.test(parts[1] || "")) return parts[1];
    if (parts[0] === "embed" && /^[\w-]{11}$/.test(parts[1] || "")) return parts[1];
    if (parts[0] === "live" && /^[\w-]{11}$/.test(parts[1] || "")) return parts[1];
  } catch {
    /* ignore */
  }
  return null;
}

function mount() {
  const id = videoIdFromLocation();
  const host = document.getElementById("top-level-buttons-computed")
    || document.querySelector("ytd-watch-metadata")
    || document.querySelector("#above-the-fold")
    || document.body;
  if (!id || !host) return;
  let btn = document.getElementById("ytt-yt-btn");
  if (!btn) {
    btn = document.createElement("button");
    btn.id = "ytt-yt-btn";
    btn.type = "button";
    btn.textContent = "Get transcript";
    btn.addEventListener("click", () => {
      btn.disabled = true;
      chrome.runtime.sendMessage({ type: "fetchTranscript", videoId: id }, async (res) => {
        btn.disabled = false;
        if (!res?.ok) {
          btn.textContent = res?.error || "No captions";
          setTimeout(() => { btn.textContent = "Get transcript"; }, 2400);
          return;
        }
        const origins = ["http://127.0.0.1:3010", "https://youtubetotext.fly.dev"];
        let dest = `https://youtubetotext.fly.dev/?v=${id}&mode=text`;
        for (const origin of origins) {
          try {
            const r = await fetch(`${origin}/api/ingest`, {
              method: "POST",
              headers: { "content-type": "application/json" },
              body: JSON.stringify(res.doc),
            });
            if (r.ok) {
              dest = `${origin}/?v=${id}&mode=text`;
              break;
            }
          } catch {
            /* next */
          }
        }
        window.open(dest, "_blank", "noopener,noreferrer");
      });
    });
    host.prepend(btn);
  }
}

document.addEventListener("yt-navigate-finish", mount);
mount();
setInterval(mount, 2500);
