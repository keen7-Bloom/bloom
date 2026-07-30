<p align="center">
  <img src="docs/logo.svg" width="96" alt="Bloom logo">
</p>

<h1 align="center">bloom</h1>

<p align="center">A small macOS app that plays a looping video behind your desktop icons.</p>

<p align="center"><a href="https://keen7-bloom.github.io/bloom/">Website</a> ·
<a href="https://github.com/keen7-Bloom/bloom/releases">Download</a></p>

---

## What it does

Bloom sets a video file (MP4 or WebM) as an animated desktop background. It runs as a
menu bar app on macOS, using Tauri's webview for the settings UI and the system's video
decoder for playback, so it uses relatively little memory (around 39 MB with a 1080p
loop, measured on an M1 MacBook). It also stops rendering when the desktop isn't visible.

## Features (v0.1)

- Set any local MP4/WebM as a live wallpaper
- One built-in scene ("Garden")
- Runs from the menu bar, no dock icon
- Pause / resume from the tray menu
- Remembers your last wallpaper between launches
- No account, no analytics, no network requests

## Install

**macOS** (11+, Apple Silicon & Intel): grab the `.dmg` from
[Releases](https://github.com/keen7-Bloom/bloom/releases).
Unsigned beta — right-click → Open the first time.

**Windows**: on the way (v0.2).

## Build from source

```bash
npm install
npm run tauri dev    # develop
npm run tauri build  # produce installers
```

## License

[GPL-3.0](LICENSE) — free forever, and every fork stays free too.

Made in Türkiye 🌱
