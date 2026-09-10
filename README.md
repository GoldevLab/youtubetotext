# YouTubeForge

Free YouTube transcripts, audio, SRT, translation, and summaries. Built with [Resuma](https://resuma-docs.fly.dev/).

A cleaner competitor to [YouTubeToTranscript](https://youtubetotranscript.com/): searchable lines, real SRT/VTT downloads, shareable URLs, and translation. No account, no cookie wall.

Live: [forgeyt.com](https://forgeyt.com) (also [youtubetotext.fly.dev](https://youtubetotext.fly.dev))

## What it does

Paste a YouTube URL on `/`. The result lives at `/?v={videoId}&mode=text` (noindex). SEO landings keep their own URLs and the same paste form:

| URL | Job |
|---|---|
| `/youtube-to-text` | Transcript |
| `/youtube-to-audio` | Audio / MP3 |
| `/youtube-translator` | Caption translation |
| `/youtube-summary` | Chapter recap |
| `/youtube-to-srt` | SRT / VTT |
| `/privacy` | Privacy / AdSense |
| `/terms` | Terms (captions, MP3, video, API) |
| `/extension` | Chrome extension |
| `/pricing` | Free web + Basic/Pro API (Lemon Squeezy) |
| `/developers` | API docs |

`/v/{id}` still redirects to `/?v=`. Canonical origin: `SITE_URL=https://forgeyt.com`. Legacy Spanish slugs (`/youtube-a-texto`, `/youtube-a-mp3`, `/youtube-traductor`, `/youtube-resumen`, `/youtube-a-srt`) 301 to the English landings.

- Paste a YouTube URL (watch, shorts, `youtu.be`, or a raw video id)
- Search, trim, copy, download **TXT, SRT, VTT, Markdown, JSON**
- Download audio when YouTube exposes a plain audio URL
- Optional **paid API** (`x-api-key`) via Lemon Squeezy subscriptions

Website `/api/*` routes power the UI (same-site). Paid keys unlock the same endpoints without proof-of-work. Downloads on the free web still need a short-lived HMAC ticket after local SHA-256 PoW. Shared `/?v=` transcript links stay open.

## Optional env (do not invent values)

Set these on Fly as secrets. Leave them unset locally unless you have real IDs.

| Env | Effect |
|---|---|
| `SITE_URL` | Canonical origin |
| `ADSENSE_CLIENT` / `ADSENSE_SLOT*` | Live ads + `/ads.txt` |
| `FORGE_API_KEYS` or `API_KEY` | Ops escape hatch for internal tooling |
| `LEMON_API_KEY` | Lemon Squeezy API key |
| `LEMON_STORE_ID` | Store id (numeric string) |
| `LEMON_VARIANT_BASIC` / `LEMON_VARIANT_PRO` | Subscription variant ids |
| `LEMON_WEBHOOK_SECRET` | Webhook signing secret → `POST /api/billing/webhook` |
| `CONTACT_EMAIL` | Shown on `/privacy` |
| `CHROME_STORE_URL` | Store button on `/extension` |
| `GSC_VERIFICATION` | Search Console HTML meta tag (paste the content= value) |
| `GA4_ID` (`G-…`) or `PLAUSIBLE_DOMAIN` | Analytics |
| `META_PIXEL_ID` (digits only) | Meta Pixel (deferred; PageView + ViewContent on `?v=`) |
| `PRIVATE_PIXEL_TOKEN` | Meta Conversions API token — mirrors ViewContent server-side with the same `event_id` as the Pixel |
| `META_TEST_EVENT_CODE` (optional) | Events Manager test code while verifying CAPI |
| `GATE_SECRET` (optional) | 32-byte hex HMAC key for download tickets; auto-generated under `RESUMA_DATA_DIR` if unset |

### Lemon Squeezy setup

1. Create two subscription products/variants: **Basic $9/mo**, **Pro $29/mo**.
2. Set the secrets above on Fly.
3. Webhook URL: `https://forgeyt.com/api/billing/webhook` — events: `subscription_created`, `subscription_updated`, `subscription_expired`, `subscription_cancelled`, `subscription_paused`, `subscription_resumed`, `order_created`.
4. Checkout: `/api/billing/checkout?plan=basic|pro` → Lemon → `/developers/welcome?t=…` reveals the key once.

**You do in Google / Meta (not in git):**
1. [Search Console](https://search.google.com/search-console) → add property `https://forgeyt.com` → verify (DNS or put the token in `GSC_VERIFICATION`) → Sitemaps → submit `https://forgeyt.com/sitemap.xml`.
2. [AdSense](https://www.google.com/adsense/) → Sites → add `forgeyt.com` → create a responsive Display unit → set `ADSENSE_CLIENT` + `ADSENSE_SLOT` as Fly secrets (then `/ads.txt` stops 404).
3. [Meta Events Manager](https://business.facebook.com/events_manager) → Connect data source → Web → Pixel → copy Pixel ID → `fly secrets set META_PIXEL_ID=123456789012345`. For Conversions API (recommended): generate a token under Settings → Conversions API → `fly secrets set PRIVATE_PIXEL_TOKEN=…`. Pixel + CAPI share `event_id` on ViewContent (`?v=`) so Meta dedupes. Optional: `META_TEST_EVENT_CODE` while testing. Verify with Meta Pixel Helper / Test Events.

`youtubetotext.fly.dev` page traffic redirects to `forgeyt.com` (probes `/health` stay on Fly).

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
