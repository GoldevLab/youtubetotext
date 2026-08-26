const WEB_KEY = "AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8";
const ANDROID_KEY = "AIzaSyA8eiZmM1FaDVz4ve3x5lWK4ntUNQ2-7oc";
const PLAYER = "https://www.youtube.com/youtubei/v1/player?prettyPrint=false";

const CLIENTS = [
  {
    name: "IOS",
    version: "20.10.4",
    ua: "com.google.ios.youtube/20.10.4 (iPhone16,2; U; CPU iOS 18_3_2 like Mac OS X; en_US)",
    header: "5",
    key: WEB_KEY,
    extra: { deviceMake: "Apple", deviceModel: "iPhone16,2", osName: "iPhone", osVersion: "18.3.2.22D82" },
  },
  {
    name: "ANDROID",
    version: "20.10.38",
    ua: "com.google.android.youtube/20.10.38 (Linux; U; Android 14) gzip",
    header: "3",
    key: ANDROID_KEY,
    extra: { androidSdkVersion: 34, osName: "Android", osVersion: "14" },
  },
];

function setQuery(url, key, value) {
  const amp = `&${key}=`;
  const qst = `?${key}=`;
  const i = url.indexOf(amp) >= 0 ? url.indexOf(amp) : url.indexOf(qst);
  if (i >= 0) {
    const valAt = i + amp.length;
    const endRel = url.slice(valAt).indexOf("&");
    const end = endRel >= 0 ? valAt + endRel : url.length;
    return url.slice(0, valAt) + value + url.slice(end);
  }
  return url + (url.includes("?") ? "&" : "?") + `${key}=${value}`;
}

function parseJson3(data) {
  const cues = [];
  for (const ev of data.events || []) {
    if (!ev.segs) continue;
    const text = ev.segs.map((s) => s.utf8 || "").join("").replace(/\s+/g, " ").trim();
    if (!text) continue;
    cues.push({
      start_ms: Number(ev.tStartMs) || 0,
      duration_ms: Number(ev.dDurationMs) || 0,
      text,
    });
  }
  return cues;
}

function parseVtt(body) {
  const cues = [];
  const blocks = body.replace(/^\uFEFF/, "").split(/\n\n+/);
  for (const block of blocks) {
    const lines = block.split("\n").map((l) => l.trim()).filter(Boolean);
    const time = lines.find((l) => l.includes("-->"));
    if (!time) continue;
    const [start] = time.split("-->").map((s) => s.trim());
    const text = lines.filter((l) => l !== time && !/^\d+$/.test(l)).join(" ").trim();
    if (!text) continue;
    const parts = start.replace(",", ".").split(":");
    let ms = 0;
    if (parts.length === 3) {
      ms = (+parts[0] * 3600 + +parts[1] * 60 + parseFloat(parts[2])) * 1000;
    } else if (parts.length === 2) {
      ms = (+parts[0] * 60 + parseFloat(parts[1])) * 1000;
    }
    cues.push({ start_ms: Math.round(ms), duration_ms: 0, text });
  }
  return cues;
}

function pickTrack(tracks, lang) {
  if (!tracks.length) return null;
  const wanted = (lang || "").replace("|asr", "");
  const wantAsr = (lang || "").endsWith("|asr");
  if (wanted) {
    const match = tracks.find((t) => {
      const code = t.languageCode || "";
      const asr = t.kind === "asr";
      if (wantAsr) return asr && code.toLowerCase() === wanted.toLowerCase();
      return code.toLowerCase() === wanted.toLowerCase() && !asr;
    }) || tracks.find((t) => (t.languageCode || "").toLowerCase().startsWith(wanted.toLowerCase()));
    if (match) return match;
  }
  return tracks.find((t) => t.kind !== "asr" && (t.languageCode || "").startsWith("en"))
    || tracks.find((t) => t.kind !== "asr")
    || tracks[0];
}

async function player(videoId, client, hl) {
  const url = `${PLAYER}&key=${client.key}`;
  const res = await fetch(url, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "User-Agent": client.ua,
      "X-YouTube-Client-Name": client.header,
      "X-YouTube-Client-Version": client.version,
    },
    body: JSON.stringify({
      context: {
        client: {
          clientName: client.name,
          clientVersion: client.version,
          hl,
          gl: "US",
          userAgent: client.ua,
          ...client.extra,
        },
      },
      videoId,
      contentCheckOk: true,
      racyCheckOk: true,
      params: "CgIQBg==",
    }),
  });
  if (!res.ok) throw new Error(`player ${res.status}`);
  return res.json();
}

async function timedtext(baseUrl) {
  for (const fmt of ["json3", "vtt", "srv3"]) {
    const url = setQuery(baseUrl, "fmt", fmt);
    const res = await fetch(url, { headers: { Accept: "*/*" } });
    if (res.status === 429) throw new Error("YouTube rate-limited caption files from this network.");
    if (!res.ok) continue;
    const body = await res.text();
    if (!body.trim()) continue;
    if (fmt === "json3" || body.trim().startsWith("{")) {
      try {
        const cues = parseJson3(JSON.parse(body));
        if (cues.length) return cues;
      } catch {
        /* try next */
      }
    }
    if (body.startsWith("WEBVTT") || fmt === "vtt") {
      const cues = parseVtt(body);
      if (cues.length) return cues;
    }
  }
  throw new Error("YouTube sent an empty caption file.");
}

export async function fetchTranscript(videoId, lang, tlang) {
  if (!/^[\w-]{11}$/.test(videoId)) throw new Error("Not a YouTube video id.");
  let lastErr = "Could not read this video.";
  for (const hl of ["en", "es"]) {
    for (const client of CLIENTS) {
      try {
        const data = await player(videoId, client, hl);
        const details = data.videoDetails || {};
        const tracks = data.captions?.playerCaptionsTracklistRenderer?.captionTracks || [];
        const track = pickTrack(tracks, lang);
        if (!track?.baseUrl) {
          lastErr = "This video has no captions.";
          continue;
        }
        let url = track.baseUrl;
        if (tlang && tlang.toLowerCase() !== (track.languageCode || "").toLowerCase()) {
          url = setQuery(url, "tlang", tlang);
        }
        const cues = await timedtext(url);
        return {
          video_id: videoId,
          title: details.title || videoId,
          author: details.author || "",
          duration_secs: Number(details.lengthSeconds) || 0,
          lang: track.languageCode || "",
          kind: track.kind || "",
          tlang: tlang || "",
          cues,
        };
      } catch (e) {
        lastErr = e.message || String(e);
      }
    }
  }
  throw new Error(lastErr);
}

export function videoIdFromUrl(raw) {
  try {
    const u = new URL(raw);
    const v = u.searchParams.get("v") || u.searchParams.get("vi");
    if (v && /^[\w-]{11}$/.test(v)) return v;
    const parts = u.pathname.split("/").filter(Boolean);
    const host = u.hostname.replace(/^www\./, "");
    if (host === "youtu.be" && /^[\w-]{11}$/.test(parts[0] || "")) return parts[0];
    const i = parts.findIndex((p) => ["embed", "shorts", "live", "v", "watch"].includes(p));
    const id = i >= 0 ? (parts[i + 1] || "").slice(0, 11) : "";
    if (/^[\w-]{11}$/.test(id)) return id;
  } catch {
    /* ignore */
  }
  return null;
}
