<p align="center">
  <img src="docs/logo.svg" width="96" alt="Bloom logo">
</p>

<h1 align="center">bloom</h1>

<p align="center"><b>Live wallpapers that don't eat your Mac.</b><br>
Any video, behind your icons, alive — in under 40 MB of RAM.</p>

<p align="center"><a href="https://keen7-bloom.github.io/bloom/">Website</a> ·
<a href="https://github.com/keen7-Bloom/bloom/releases">Download</a></p>

---

## Why

Wallpaper Engine idles at 450–900 MB because it ships a whole browser. Bloom uses the
webview your OS already has (settings UI only) and your GPU's native video decoder for
the wallpaper itself. **Measured: 39 MB with a 1080p loop on an M1 MacBook.** When your
windows cover the desktop, macOS suspends Bloom entirely — zero frames, 0.0% CPU.

## Features (v0.1)

- Any MP4/WebM as a live wallpaper — drag it in, done
- Built-in "Garden" scene
- Menu bar app: no dock icon, no clutter
- Pause / resume from the tray
- Remembers your wallpaper across restarts
- No account, no telemetry, no network access

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
