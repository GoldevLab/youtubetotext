# Chrome extension (YouTubeForge)

Load unpacked: chrome://extensions → Developer mode → Load unpacked → this folder.

Publish: zip this folder (not the repo root). Chrome Web Store listing copy is in STORE.txt. Privacy policy URL: https://youtubetotext.fly.dev/privacy

After you buy a domain:
1. Fly: `SITE_URL=https://YOUR.DOMAIN`
2. Edit `config.js` (`SITE`)
3. Edit `manifest.json` (`homepage_url`, `host_permissions`, `content_scripts.matches` for the site)
4. Fly: `CHROME_STORE_URL=https://chromewebstore.google.com/detail/...` once the listing is live
5. AdSense: add the new domain, keep `ADSENSE_CLIENT` / `ADSENSE_SLOT*` as secrets — never inside this extension
