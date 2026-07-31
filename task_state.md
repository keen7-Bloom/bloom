# Bloom — project state

**Last updated:** July 31, 2026
**Repo:** https://github.com/keen7-Bloom/bloom (public, GPL-3.0)
**Site:** https://keen7-bloom.github.io/bloom/
**Support:** bloomappsupportapp@gmail.com
**Local path:** `~/Projects/bloom`

---

## What Bloom is

A desktop app that plays a looping video file behind your desktop icons, on macOS and
Windows (Linux experimental). Menu bar / tray only — no dock icon, no main window.

The pitch is efficiency: most live wallpaper apps bundle their own Chromium and idle at
450–900 MB. Bloom uses the OS's existing webview plus native GPU video decode.

---

## Current objective

Get Bloom in front of real users and collect real feedback. The app works; distribution
is the bottleneck. Every "post it somewhere" channel has hit new-account gates.

---

## Architecture

**Stack:** Tauri v2 + Rust backend, vanilla HTML/CSS/JS frontend, Vite.

Chosen over Electron specifically for memory footprint. Electron bundles Chromium
(~100 MB before rendering anything); Tauri uses the platform webview — WKWebView on
macOS, WebView2 on Windows, WebKitGTK on Linux.

**Two windows, one of which is invisible to the user:**

| File | Role |
|---|---|
| `wallpaper.html` | The actual wallpaper renderer. Full-screen, borderless, click-through, sits at desktop level. |
| `index.html` | Vite entry / settings surface. Currently unused at runtime — all control is via the tray menu. |
| `src-tauri/src/lib.rs` | All native logic: window creation, tray menu, underlay, power polling. |

**How the wallpaper gets behind icons** — `tauri-plugin-desktop-underlay` v0.2.1:

- **macOS:** sets `NSWindow` level to `CGWindowLevelForKey(kCGDesktopWindowLevelKey) - 1`,
  plus collection behavior `CanJoinAllSpaces | Stationary | IgnoresCycle`.
- **Windows:** Win32 window layering.
- **Linux:** GTK `WindowTypeHint::Desktop` → X11 `_NET_WM_WINDOW_TYPE_DESKTOP`.

Plus `set_ignore_cursor_events(true)` so clicks pass through to the real desktop.

**Scene system:** `wallpaper.html` runs in one of two modes — `garden` (a canvas animation
of drifting brand-coloured orbs, the built-in default) or `video` (an HTML `<video>` element
fed a local file via Tauri's asset protocol). Mode and file path persist in `localStorage`
under `bloom.video`.

**Pause logic** — deliberately two independent inputs ANDed together:

```
should_run = userRunning && !(onBattery && battPref)
```

`userRunning` toggles from the tray. `onBattery` comes from a Rust thread polling power
state every 15s and emitting `bloom://power`. `battPref` is the "Pause on Battery" tray
checkbox, persisted in `localStorage` as `bloom.battpause` (defaults on).

Power detection per platform:
- macOS: shells out to `pmset -g batt`, looks for `'Battery Power'`
- Windows: Win32 `GetSystemPowerStatus`, checks `ACLineStatus == 0`
- Linux: reads `/sys/class/power_supply/*/type` for `Mains`, then its `online` file

**Free win:** when the desktop is fully occluded, macOS suspends the webview process on
its own — measured 0.0% CPU and ~1 MB. We didn't build that; the OS does it.

**Distribution:** GitHub Actions on tag push. Three-phase workflow — one job creates a
draft release, a matrix of platform jobs build and upload into it, a final job publishes.

---

## Measured performance

All figures from an M1 MacBook Air, July 2026, sampled with `ps` while the desktop was
actually visible.

| Condition | RAM | CPU |
|---|---|---|
| Idle / desktop occluded | ~13–27 MB | 0.0% |
| 1080p video playing | ~39 MB | low |
| 4K video playing | **57–70 MB** | 10–36% |

**Unmeasured:** Windows. It uses WebView2 (Chromium-based), architecturally different from
WKWebView, so macOS numbers do not transfer. One friend reported roughly 10% of 8 GB
(~800 MB) on a possibly-4K file — never confirmed with a screenshot. The website shows
"measuring…" for Windows rather than guessing.

---

## Bugs found and fixed

**Tray menu did nothing (v0.1.0).** Tauri v2 capabilities are per-window and
`capabilities/default.json` only listed `"main"`. Our window is named `"wallpaper"`, so it
silently had no permission to receive events. Fix: add `"wallpaper"` to the windows array.
Silent failure, no error anywhere — worth remembering.

**Windows CI job failed while actually succeeding (v0.1.0).** The build produced a valid
`.msi` and `.exe`, then died at upload: all three matrix jobs raced to create the same
release, two lost with `Not Found`. Fix: split into create-release → build matrix →
publish.

**Measured the wrong thing entirely.** Reported ~23 MB for 4K playback. It was 0.0% CPU
across every sample — the webview was suspended because the desktop was covered. The real
number under actual playback is 57–70 MB. Lesson: check CPU before trusting a RAM figure.

**Claimed a screenshot that didn't exist.** A draft Reddit post said "screenshots in the
repo" proving the RAM numbers. There is no Activity Monitor screenshot committed. Removed.
Still worth actually capturing one.

**Cosmetic, unfixed:** `bloom_aarch64.app.tar.gz` and `bloom_x64.app.tar.gz` on the
releases are macOS updater bundles, not Linux builds. The `.tar.gz` extension misleads —
it fooled us. Either delete them or wire up the updater that would use them.

---

## Distribution attempts

| Channel | Outcome |
|---|---|
| GitHub | Account auto-flagged as spam hours after creation. Cause: repo topics `wallpaper` / `live-wallpaper` on a brand-new account. Fixed via ticket #4615278 — made repo private, swapped topics to `tauri-app`/`desktop-app`/`rust`/`macos-app`/`windows-app`/`video-player`, de-hyped README and site copy. Cleared in ~5 min. |
| Product Hunt | **Live since July 31, 12:01am PT.** 7 followers as of last check. No comments beyond our own maker intro. Upvote count hidden during PH's anti-gaming randomization window. |
| Hacker News | Blocked twice. "Show HN" restricted site-wide; plain link submission rejected with "your account isn't able to submit this site" (new account). |
| r/macapps | Auto-removed. Needs verified email, 7-day-old account, 10 karma **from that subreddit specifically**. |
| r/opensource | Auto-removed. Karma requirement, "exceptions will not be made." |

Reddit is not blocked by content — it's blocked by the account having ~0 karma. No appeal
process; the fix is genuinely to go comment on other people's posts for a while.

Reddit is also unreachable from the assistant's browser tools (policy block), so those
posts have to be made manually.

---

## Real usage, honestly

- GitHub stars: 0. Forks: 0. Repo page views: 1.
- Downloads: 1 Mac dmg, 4 Windows exe — **all accounted for** as the developer, one friend,
  and CI/curl link verification. No confirmed stranger has downloaded Bloom.
- Support inbox: zero real messages. Only automated Google/Product Hunt mail.
- Product Hunt followers: 7. This is the only confirmed outside interest.

Nobody has independently verified the app works, except one friend on Windows who
confirmed the installer ran and the wallpaper played.

---

## Brand

- Wordmark `bloom`, lowercase, the two `o`s in coral `#FF4D6D`
- Mark: vesica — two overlapping outlined circles, filled lens where they meet
- Palette: coral `#FF4D6D` → amber `#FFB13C` → green `#2BE5A7` on ink `#07090C`
- Type: Geist (site), Futura (original lockup)
- Site design: macOS desktop metaphor — fake menu bar with live clock, draggable windows,
  Activity Monitor table, terminal-styled FAQ. Default scene is Noir; Garden/Ember/Tide
  switchable live. Tagline: *"Beauty that costs nothing to run."*

---

## Next up

**In flight right now:** v0.3.0 CI build — first Linux attempt. Four jobs running
(2× macOS, Windows, ubuntu-22.04). Unknown whether the Ubuntu job compiles; the apt
dependency list (`libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`,
`patchelf`, `libgtk-3-dev`) is unverified.

**Immediate:**
1. Confirm the Linux job built. If it failed, the dependency list is the first suspect.
2. Get someone with a real Linux machine to try the `.AppImage`. X11 session only —
   Wayland has no equivalent to the desktop window-type hint, so it will float on top.
   Bloom prints a startup warning when it detects Wayland.
3. Get the Windows RAM number. Need Task Manager screenshot + video resolution + whether
   hardware GPU scheduling is on. If it's genuinely ~800 MB, suspect software decode
   fallback in WebView2 — that's a real bug, not just WebView2 overhead.
4. Delete the two misleading `.tar.gz` assets from releases.
5. Commit a real Activity Monitor screenshot so the README's numbers have receipts.

**Distribution, when ready:**
- Build ~10 Reddit karma by commenting normally, then post to r/SideProject,
  r/opensource, r/unixporn (screenshot-first — the app is visual and text undersells it).
- Untried and ungated: Bluesky, Mastodon (fosstodon), IndieHackers, Tauri Discord
  #showcase, AlternativeTo (list as a Wallpaper Engine alternative), `awesome-tauri` PR.

**Product roadmap (v0.4+):** audio-reactive wallpapers, playlists / scheduled rotation,
per-monitor scenes, start-at-login, a starter pack of bundled loops.

**Deliberately not doing:**
- **FreeBSD.** No GitHub Actions runners exist, Tauri doesn't target it, the underlay
  plugin has no FreeBSD path. Would mean building a toolchain from scratch for
  approximately zero users.
- **Code signing.** Apple Developer Program is $99/yr; both installers ship unsigned, so
  macOS needs right-click → Open and Windows shows SmartScreen. Revisit if adoption
  justifies it.

**Monetization — currently blocked.** Turkey is excluded from both Stripe and PayPal, and
Ko-fi / Buy Me a Coffee / Gumroad all pay out exclusively through those. Payoneer is the
usual workaround for Turkish freelancers but its compatibility with these platforms is
unconfirmed. Any account also needs to belong to someone who can legally receive funds —
a family conversation, not a technical one. Realistic near-term income is freelance work
(Bionluk pays TRY to a Turkish bank directly, no international rails), using Bloom as
portfolio proof.

---

## Gotchas for future sessions

- Rust lives at `/opt/homebrew/opt/rustup/bin` (Homebrew rustup proxies), added to
  `~/.zshrc`. `cargo` is not on the default PATH in fresh shells.
- `vite.config.ts` needs both `index.html` and `wallpaper.html` as rollup inputs, or the
  wallpaper page won't exist in production builds.
- `tauri.conf.json` has `assetProtocol.scope: ["**"]` so the webview can load video files
  from anywhere on disk.
- The app icon is generated from `docs/logo.svg` on an ink rounded-rect via
  `npm run tauri icon`.
- Version must be bumped in three places: `tauri.conf.json`, `src-tauri/Cargo.toml`,
  `package.json`.
- GitHub Pages serves from `main` branch, `/docs` folder. Going private removes the Pages
  config entirely — it must be recreated via API, not just re-enabled.
