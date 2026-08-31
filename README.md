# YouTubeForge

Free YouTube transcripts, audio, SRT, translation, and summaries. Built with [Resuma](https://resuma-docs.fly.dev/).

A cleaner competitor to [YouTubeToTranscript](https://youtubetotranscript.com/): searchable lines, real SRT/VTT downloads, shareable URLs, translation, and a free HTTP API. No account, no cookie wall.

Live: [youtubetotext.fly.dev](https://youtubetotext.fly.dev)

## What it does

YouTubeForge family (one app, several SEO URLs):

| Landing | App |
|---|---|
| `/youtube-to-text` | Transcript |
| `/youtube-to-audio` | Audio download (`GET /api/audio?v=`) |
| `/youtube-translator` | Caption `tlang` |
| `/youtube-summary` | Chapter recap + prompts |
| `/youtube-to-srt` | SRT/VTT |

Paste once on `/`. Tools for the same video (audio, translate, summary, SRT) sit in the transcript column. Shareable pages stay at `/v/{id}`.

- Paste a YouTube URL (watch, shorts, `youtu.be`, or a raw video id)
- Search, trim, copy, download **TXT, SRT, VTT, Markdown, JSON**
- Download audio when YouTube exposes a plain audio URL
- `GET /api/transcript?v=…&fmt=json|txt|srt|vtt|md`

The public API and video loads are rate-limited per IP so scrapers cannot drain YouTube captions through this app. Optional Cloudflare Turnstile: set `TURNSTILE_SITE_KEY` and `TURNSTILE_SECRET` on Fly.

## Google AdSense

Slots stay reserved (leaderboard / infeed / rectangle). Placeholders until you set a publisher ID and unit IDs.

```bash
# local
ADSENSE_CLIENT=ca-pub-xxxxxxxxxxxxxxxx
ADSENSE_SLOT_LEADERBOARD=1234567890
ADSENSE_SLOT_INFEED=1234567891
ADSENSE_SLOT_RECTANGLE=1234567892

# optional per placement, e.g. home-hero → ADSENSE_SLOT_HOME_HERO
# fly
fly secrets set ADSENSE_CLIENT=ca-pub-xxxxxxxxxxxxxxxx \
  ADSENSE_SLOT_LEADERBOARD=… ADSENSE_SLOT_INFEED=… ADSENSE_SLOT_RECTANGLE=…
```

With `ADSENSE_CLIENT` set, the app serves `/ads.txt` and loads `adsbygoogle.js`. Create matching display units in AdSense (responsive). Rectangle units fill when the download dialog opens.

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
