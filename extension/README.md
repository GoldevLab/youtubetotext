# Chrome extension (YouTubeForge)

Load unpacked: chrome://extensions → Developer mode → Load unpacked → this folder.

Publish: zip this folder (not the repo root). Chrome Web Store listing copy is in STORE.txt. Privacy policy URL: https://forgeyt.com/privacy

Domain is `forgeyt.com`. Keep `config.js` (`SITE`) and `manifest.json` host permissions in sync with Fly `SITE_URL`.
After Chrome Web Store publish: Fly `CHROME_STORE_URL=https://chromewebstore.google.com/detail/...`
AdSense: add `forgeyt.com` in AdSense, keep `ADSENSE_CLIENT` / `ADSENSE_SLOT*` as Fly secrets — never inside this extension.
