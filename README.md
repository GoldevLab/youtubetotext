# YouTubeForge

Free YouTube transcripts, audio, SRT, translation, and summaries. Built with [Resuma](https://resuma-docs.fly.dev/).

A cleaner competitor to [YouTubeToTranscript](https://youtubetotranscript.com/): searchable lines, real SRT/VTT downloads, shareable URLs, translation, and a free HTTP API. No account, no cookie wall.

Live: [youtubetotext.fly.dev](https://youtubetotext.fly.dev)

## What it does

Paste a YouTube URL on `/`. The result lives at `/?v={videoId}&mode=text` (audio, translate, summary, and SRT stay on that same page). Old landings like `/youtube-to-text` and `/v/{id}` redirect there.

- Paste a YouTube URL (watch, shorts, `youtu.be`, or a raw video id)
- Search, trim, copy, download **TXT, SRT, VTT, Markdown, JSON**
- Download audio when YouTube exposes a plain audio URL
- `GET /api/transcript?v=…&fmt=json|txt|srt|vtt|md`

The public API and video loads are rate-limited per IP so scrapers cannot drain YouTube captions through this app. Optional Cloudflare Turnstile: set `TURNSTILE_SITE_KEY` and `TURNSTILE_SECRET` on Fly.

## Google AdSense

Slots stay reserved (leaderboard / infeed / rectangle). Placeholders until you set a publisher ID and at least one unit ID. Do not commit these values.

```bash
# local — one client + one responsive display unit is enough to fill every slot
export ADSENSE_CLIENT=ca-pub-xxxxxxxxxxxxxxxx
export ADSENSE_SLOT=1234567890

# or one unit per size
export ADSENSE_SLOT_LEADERBOARD=1234567890
export ADSENSE_SLOT_INFEED=1234567891
export ADSENSE_SLOT_RECTANGLE=1234567892

# optional per placement, e.g. home-hero → ADSENSE_SLOT_HOME_HERO

# Fly (secrets, not fly.toml)
fly secrets set ADSENSE_CLIENT=ca-pub-xxxxxxxxxxxxxxxx ADSENSE_SLOT=1234567890
```

In AdSense: create **Display ads → Responsive**. Copy `ca-pub-…` and the numeric slot. After deploy, `/ads.txt` is served automatically. Rectangle units also fill when the download dialog opens.

CSP stays on as report-only so AdSense iframes and the YouTube player are not blocked (Resuma 1.3.1 has no `frame-src` yet).

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

CSP is report-only so YouTube thumbnails, the nocookie player, and AdSense iframes can load.
