# YouTubeForge

Free YouTube transcripts, audio, SRT, translation, and summaries. Built with [Resuma](https://resuma-docs.fly.dev/).

A cleaner competitor to [YouTubeToTranscript](https://youtubetotranscript.com/): searchable lines, real SRT/VTT downloads, shareable URLs, translation, and a free HTTP API. No account, no cookie wall.

Live: [youtubetotext.fly.dev](https://youtubetotext.fly.dev)

## What it does

Paste a YouTube URL on `/`. The result lives at `/?v={videoId}&mode=text` (noindex). SEO landings keep their own URLs and the same paste form:

| URL | Job |
|---|---|
| `/youtube-to-text` | Transcript |
| `/youtube-to-audio` | Audio / MP3 |
| `/youtube-translator` | Caption translation |
| `/youtube-summary` | Chapter recap |
| `/youtube-to-srt` | SRT / VTT |
| `/youtube-a-texto` | Transcripción (ES) |
| `/youtube-a-mp3` | Audio / MP3 (ES) |
| `/youtube-traductor` | Traducir subtítulos (ES) |
| `/youtube-resumen` | Resumen (ES) |
| `/youtube-a-srt` | SRT / VTT (ES) |
| `/privacy` | Privacy / AdSense |
| `/terms` | Terms (captions, MP3, video) |
| `/pricing` | API key / higher limits |
| `/api` | API docs |
| `/extension` | Chrome extension |

`/v/{id}` still redirects to `/?v=`. Custom domain later: set `SITE_URL` (Fly already has the fly.dev value).

- Paste a YouTube URL (watch, shorts, `youtu.be`, or a raw video id)
- Search, trim, copy, download **TXT, SRT, VTT, Markdown, JSON**
- Download audio when YouTube exposes a plain audio URL
- `GET /api/transcript?v=…&fmt=json|txt|srt|vtt|md`

The public API and video loads are rate-limited per IP so scrapers cannot drain YouTube captions through this app. Optional Cloudflare Turnstile: set `TURNSTILE_SITE_KEY` and `TURNSTILE_SECRET` on Fly.

Optional `X-Api-Key` or `Authorization: Bearer` matching `FORGE_API_KEYS` (comma-separated, each ≥16 chars) or `API_KEY` raises the per-minute caps (about 240 transcript / 80 audio / 40 video). Request a key from the mailbox in `CONTACT_EMAIL`, or GitHub issues if that env is unset. No Stripe checkout on the site.

## Optional env (do not invent values)

Set these on Fly as secrets. Leave them unset locally unless you have real IDs.

| Env | Effect |
|---|---|
| `SITE_URL` | Canonical origin |
| `ADSENSE_CLIENT` / `ADSENSE_SLOT*` | Live ads + `/ads.txt` |
| `FORGE_API_KEYS` or `API_KEY` | Higher API limits |
| `CONTACT_EMAIL` | Shown on `/privacy` and `/pricing` |
| `CHROME_STORE_URL` | Store button on `/extension` |
| `GSC_VERIFICATION` | Search Console meta |
| `GA4_ID` (`G-…`) or `PLAUSIBLE_DOMAIN` | Analytics |
| `TURNSTILE_SITE_KEY` / `TURNSTILE_SECRET` | Home paste captcha |

Search Console sitemap submit and AdSense approval stay in your Google accounts — the app only exposes `/sitemap.xml` and `/ads.txt`.

## Google AdSense

Live units (keep it to these): `home-faq` and `landing-mid` after the article, `workspace-player` + `workspace-cues` on the result, `workspace-video-dl` / `home-video-dl` in the download dialog. No ads on 404, loading, or error. Placeholders until you set a publisher ID and at least one unit ID. Do not commit these values.

```bash
# local — one client + one responsive display unit is enough to fill every slot
export ADSENSE_CLIENT=ca-pub-xxxxxxxxxxxxxxxx
export ADSENSE_SLOT=1234567890

# or one unit per size
export ADSENSE_SLOT_LEADERBOARD=1234567890
export ADSENSE_SLOT_INFEED=1234567891
export ADSENSE_SLOT_RECTANGLE=1234567892

# optional per placement, e.g. home-faq → ADSENSE_SLOT_HOME_FAQ

# Fly (secrets, not fly.toml)
fly secrets set ADSENSE_CLIENT=ca-pub-xxxxxxxxxxxxxxxx ADSENSE_SLOT=1234567890
```

In AdSense: create **Display ads → Responsive**. Copy `ca-pub-…` and the numeric slot. After deploy, `/ads.txt` is served automatically. Rectangle units also fill when the download dialog opens.

CSP stays on as report-only so AdSense iframes and the YouTube player are not blocked (Resuma 1.3.1 has no `frame-src` yet).

## Development

```bash
cd youtubeText
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
