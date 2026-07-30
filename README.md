<p align="center">
  <img src="docs/logo.svg" width="96" alt="Bloom logo">
</p>

<h1 align="center">bloom</h1>

<p align="center">A small app for macOS and Windows that plays a looping video behind your desktop icons.</p>

<p align="center"><a href="https://keen7-bloom.github.io/bloom/">Website</a> ·
<a href="https://github.com/keen7-Bloom/bloom/releases">Download</a></p>

---

## What it does

Bloom sets a video file (MP4 or WebM) as an animated desktop background. It runs as a
menu bar / tray app, using Tauri's webview for the settings UI and the system's video
decoder for playback. It also stops rendering when the desktop isn't visible, and pauses
automatically on battery power.

**Memory usage:** on macOS this measures around 39 MB with a 1080p loop (M1 MacBook,
native WKWebView + native decode). On Windows, Tauri uses Microsoft's WebView2
(Chromium-based) instead, which is a different engine — we haven't published a verified
number for it yet. Don't assume the two platforms match; we'll update this once we have
real measurements from Windows hardware.

## Features (v0.2)

- Set any local MP4/WebM as a live wallpaper
- One built-in scene ("Garden")
- Runs from the menu bar / tray, no dock or taskbar icon
- Pause / resume from the tray menu
- Pauses automatically on battery power (macOS and Windows)
- Remembers your last wallpaper between launches
- No account, no analytics, no network requests

## Install

**macOS** (11+, Apple Silicon & Intel): grab the `.dmg` from
[Releases](https://github.com/keen7-Bloom/bloom/releases).
Unsigned beta — right-click → Open the first time.

**Windows** (10+): grab the `.exe` or `.msi` from
[Releases](https://github.com/keen7-Bloom/bloom/releases).
Unsigned beta — Windows SmartScreen may warn on first run; click "More info" → "Run anyway".
This build is new and less tested than macOS — please report anything odd.

## Build from source

```bash
npm install
npm run tauri dev    # develop
npm run tauri build  # produce installers
```

## Support

Found a bug or something's not working right? [Open an issue](https://github.com/keen7-Bloom/bloom/issues)
or email bloomappsupportapp@gmail.com.

## License

[GPL-3.0](LICENSE) — free forever, and every fork stays free too.

Made in Türkiye 🌱
