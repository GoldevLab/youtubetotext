# YouTubeToText

Free **YouTube transcript** tool — **100% Rust** with [Resuma](https://resuma-docs.fly.dev/).

A cleaner competitor to [YouTubeToTranscript](https://youtubetotranscript.com/): searchable lines, real SRT/VTT downloads, shareable URLs, translation, and a free HTTP API. No account, no cookie wall.

Live: [youtubetotext.fly.dev](https://youtubetotext.fly.dev)

## What it does

- Paste a YouTube URL (watch, shorts, `youtu.be`, or a raw video id)
- Read the YouTube transcript next to a privacy-friendly player (YouTube loads on play)
- Search, trim intro/outro (chips or click two lines), copy plain / timed / Markdown
- Edit caption typos before copy or download
- Download **TXT, SRT, VTT, Markdown, JSON**
- Switch caption tracks and auto-translate (YouTube `tlang`)
- Chrome/Firefox extension when YouTube blocks server IPs
- `GET /api/transcript?v=…&fmt=json|txt|srt|vtt|md`

## Development

```bash
cd letrato
cargo run
```

Open http://127.0.0.1:3000 (or `RESUMA_ADDR=127.0.0.1:3010 cargo run`)

Do not put `/robots.txt`, `/sitemap.xml`, `/favicon.svg`, or `/og.svg` in `public/` — Flow already serves them.

## Deploy (Fly.io)

One shared-cpu Machine in Dallas (`dfw`), autostop when idle. Pushes to `main` deploy via GitHub Actions (`.github/workflows/fly.yml`).

```bash
fly apps create youtubetotext
fly tokens create deploy -x 999999h
# GitHub → Settings → Secrets → Actions → FLY_API_TOKEN
fly deploy --remote-only --ha=false
```

App name: `youtubetotext` → `https://youtubetotext.fly.dev`

CSP is off (`RESUMA_CSP=0`) so the YouTube embed can load after you press Play.
