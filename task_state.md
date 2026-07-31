# Bloom — project state

**Last updated:** July 31, 2026 (optimization + updater pushed, not yet released)
**Repo:** https://github.com/keen7-Bloom/bloom (public, GPL-3.0)
**Site:** https://keen7-bloom.github.io/bloom/
**Support:** bloomappsupportapp@gmail.com
**Local path:** `~/Projects/bloom`

---

## Right now, in one paragraph

**v0.4.0 shipped** with optimization + self-updater + Windows + macOS updater signing
all confirmed working in CI. But its Ubuntu job died mid-upload with a `404 Not Found`
on delete-a-release-asset — Linux `.deb`/`.rpm`/`.AppImage` uploaded fine, but
`latest.json` never got its `linux-x86_64` key, so Linux users can install v0.4.0 and
then never self-update. **v0.4.1** (in flight) fixes it: `max-parallel: 1` on the build
matrix removes the race where multiple jobs concurrently delete-then-reupload the same
`latest.json`. Site update is *held back* until v0.4.1 CI produces real assets — the
previous release cycle taught us that pushing download links before assets exist gives
users 404s. The site currently still points at v0.4.0 downloads (which do work).

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

## Optimization pass (July 31, 2026) — pushed to main, commit `87a67b3`

Applied to all three platforms. macOS numbers are hard-measured from a local
`--target aarch64-apple-darwin` release build; Windows/Linux effects are reasoned,
not yet measured.

| | Before | After |
|---|---|---|
| Rust binary | 10.80 MB | **2.43 MB** (−77.5%) |
| macOS dmg | 3.16 MB | **1.50 MB** (−52.6%) |
| Crates in Cargo.lock | 479 | 439 |
| npm runtime deps | 2 | **0** |
| Shipped frontend | 4 files | 1 file, 5.8 KB |

**Binary size.** `Cargo.toml` had no `[profile.release]` at all — it was building on
stock defaults. Added `opt-level="z"`, `lto="fat"`, `codegen-units=1`,
`panic="abort"`, `strip=true`. Nothing Bloom does in Rust is hot (create a window,
build a tray menu, poll power every 15s), so trading CPU optimisation for size costs
nothing real. This single change is most of the 77%.

**Dead weight removed.** `tauri-plugin-opener` was registered in `lib.rs` and never
called anywhere. `serde`/`serde_json` were declared and never imported (no
`#[tauri::command]` exists). `index.html` + `src/` were untouched Tauri scaffold —
its `main.ts` calls `invoke("greet")`, a command that doesn't exist in `lib.rs` —
and were shipping inside every installer. They're now excluded from the Vite input
rather than deleted, so the files remain in the repo but stop being bundled.
Capabilities dropped `opener:default` and the nonexistent `"main"` window.

**`bundle.targets` is now explicit** instead of `"all"`. Note `app` is in the list and
must stay: on macOS the updater artifact *is* `bloom.app.tar.gz`, and building only
`dmg` makes the bundler warn *"configured to create updater artifacts but no
updater-enabled targets were built"* and silently produce none. The updater-capable
targets are `app`, `appimage`, `msi`, `nsis`. Those `.tar.gz` files were never stray
Linux builds — they were unsigned updater bundles nothing consumed. (Don't try to
document this inside `tauri.conf.json`: the schema rejects unknown keys, so a
`_comment` field fails the build outright.)

**Wallpaper renderer** (`wallpaper.html`) — four fixes, all verified in a browser
harness with a stubbed `window.__TAURI__`:

1. *Gradients were rebuilt every frame.* `createRadialGradient` ran 7× per frame —
   ~420 throwaway gradient objects a second, forever. Now built once per orb at the
   origin and positioned with a transform. Also replaced `arc()`+`fill()` with
   `fillRect()`, since the gradient is transparent at its edge anyway, which drops
   path construction too.
2. *Duplicate animation loops.* `requestAnimationFrame` was called directly from both
   `showGarden()` and `apply()` with no cancel, so choosing "Garden Scene" twice left
   two self-rescheduling loops running in parallel, each doing a full render. Verified:
   three consecutive `showGarden()` calls now hold at 60 RAF/s instead of ~240.
3. *Canvas backing store was never released.* Switching to video only set
   `display:none`, which does **not** free the pixels — on a 4K screen that is tens of
   MB sitting idle behind the video for the whole session. Now zeroed on switch and
   rebuilt on return. This is the change most likely to matter for the Windows number.
4. *Render scale and frame rate.* The garden is nothing but huge soft gradients with no
   detail finer than a few hundred px, so it now renders at 0.5× CSS pixels (quarter
   the pixels) and throttles to 30fps — the orbs drift at ~6e-5 rad/ms, so 60fps was
   drawing indistinguishable frames. RAF still ticks at 60 deliberately: it stays
   vsync-aligned and preserves the macOS occlusion suspend we get for free.

**Still unmeasured:** the actual runtime RAM delta. A controlled before/after with the
desktop genuinely visible (and CPU checked, per the earlier mistake) has not been run.
Do not put a new RAM number on the site until it has.

## Auto-updater (July 31, 2026) — pushed to main, commit `f63cdef`, NOT yet released

`tauri-plugin-updater` v2.10.1. Verified end to end in a local release build (signed
with the real key, both GitHub secrets set): reports *"Finished 1 updater signature"*
instead of the old *"Signature not found for the updater JSON. Skipping upload."*

**This code exists in the repo but has not shipped to a single user yet.** The release
workflow only runs on `v*` tags, and no tag has been cut since `v0.3.0` (which predates
all of today's work). Nothing has been verified in CI — only locally on macOS. Cutting
`v0.4.0` is the next concrete step and will be the first real test of the updater, the
new `[profile.release]` flags, and the site fixes together, on all three platforms.

- Plugin registered in `lib.rs`; `updater:default` added to capabilities.
- **Check silently, install on click.** A background check runs at startup but installs
  nothing — finding an update only relabels the tray item to "Install update v0.4.0…".
  The tray item *is* the prompt, which is how a window-less app asks permission. Clicking
  it with nothing staged does a manual check instead. The staged `Update` lives in a
  `PendingUpdate(Mutex<Option<Update>>)` in app state; the guard is dropped before the
  await, since a `MutexGuard` cannot be held across one. `download_and_install` consumes
  the update, so a failed install relabels to "Retry update v…" and that click re-checks
  from scratch rather than reusing a consumed value.
- `createUpdaterArtifacts: true`, endpoint
  `https://github.com/keen7-Bloom/bloom/releases/latest/download/latest.json`.
- Workflow passes `TAURI_SIGNING_PRIVATE_KEY` / `_PASSWORD` to `tauri-action`, and its
  release body no longer claims macOS+Windows only.
- `serde_json` had to come back — adding a `plugins` block to `tauri.conf.json` makes
  `generate_context!` emit code referencing it. Removing it earlier was correct then.

**Size cost:** binary 2.43 MB → 3.43 MB (rustls/zip/tar come along). Still 68% under
the original 10.80 MB. dmg 1.50 MB → 2.26 MB, versus 3.16 MB originally.

**Signing key:** generated July 31 at `~/.tauri/bloom.key`, key id `40BE4C4B5D10052E`,
public half is in `tauri.conf.json`. An earlier key was discarded after its password was
pasted into a chat transcript; nothing had depended on it. The throwaway key used to test
the pipeline has been destroyed.

**Both GitHub secrets are set** (`TAURI_SIGNING_PRIVATE_KEY` at 11:32 UTC,
`TAURI_SIGNING_PRIVATE_KEY_PASSWORD` at 11:33 UTC, July 31). Confirmed via
`gh secret list`. Signing verified working in v0.4.0 CI on macOS aarch64 + x86_64 and
Windows (msi + nsis). Linux was signed too but never made it into `latest.json` — see
Right Now paragraph. v0.4.1 fixes that.

**Won't fix:** Tauri's update signature is not Apple/Microsoft code signing — Gatekeeper
and SmartScreen warnings remain. On Linux only the AppImage can self-update; `.deb`/`.rpm`
belong to the package manager. Anyone already on a build without the updater is stranded
on manual updates forever, which is why this went in before real users arrived.

## Website fixes (July 31, 2026) — pushed to main, commit `f63cdef`, live on Pages

Narrow pass only — checked against the code, three claims were false, one was stale.
Nothing else on the site was touched (design, layout, and voice are unchanged, on
request):

- `per-monitor` claimed a feature that doesn't exist (`lib.rs` only ever calls
  `primary_monitor()`). Relabeled `per-monitor.next`, described as planned for v0.4
  rather than removed.
- "a small starter pack ships in the box" — false, there's no bundled-video resources
  dir. Reworded to say there isn't one yet.
- "every release ships an Activity Monitor screenshot from real hardware" — false, and
  the same wrong claim had already been caught and cut from a Reddit draft earlier, just
  never fixed on the site itself. Reworded to what's actually true: every number on the
  page names the machine it came from.
- "Growing in v0.2" for audio-reactive — stale, audio-reactive is v0.4 on the roadmap.

## Measured performance — CORRECTED July 31, 2026

**Every number published before v0.4.1 was wrong.** They counted only the `bloom`
process. Bloom actually runs four: `bloom`, `com.apple.WebKit.GPU`,
`com.apple.WebKit.WebContent`, `com.apple.WebKit.Networking`. The WebKit ones have no
"bloom" in their process name, so every `grep -i bloom` silently dropped them — and
they are the larger part. `WebKit.GPU` alone is 213 MB on 4K.

M1 MacBook Air, `phys_footprint` summed across all four (the metric Activity Monitor's
"Memory" column shows). Reproduce with `scripts/measure-memory.sh`.

| Condition | Memory | CPU | Old claim |
|---|---|---|---|
| Garden scene, visible | **171 MB** | > 0 | (unlisted) |
| 1080p video, visible | **184 MB** | > 0 | 39 MB — 4.7× under |
| 4K video, visible | **308 MB** | > 0 | 57–70 MB — 4.8× under |
| Desktop covered (4K loaded) | 308 MB footprint / **52 MB resident** | **0.0%** | ~13–27 MB |

**The occlusion win is real, but it's a CPU and resident-RAM win, not a footprint win.**
Covering the desktop takes CPU to a verified 0.0% and lets macOS compress Bloom's pages
to ~52 MB actually held in RAM — but Activity Monitor keeps reporting ~308 MB, because
that column includes compressed pages. Both are true; say which you mean.

**Measurement gotchas that already burned us twice:**
- `ps -o %cpu` on macOS is the average over the process's whole *lifetime*, not now. An
  idle-but-formerly-busy process reads non-zero forever. The first version of
  `measure-memory.sh` used it and cheerfully reported "AWAKE" for a suspended app. Use
  `top -l 2` and discard the first sample.
- RSS and `phys_footprint` diverge by 6× when suspended. Name the metric or the number
  is meaningless.
- Check CPU *before* trusting any RAM figure. A suspended webview reads low for the
  wrong reason.

**Unmeasured:** Windows. Uses WebView2 (Chromium), architecturally different from
WKWebView, so none of the above transfers. One friend reported ~800 MB on a possibly-4K
file, never confirmed. Given macOS 4K is 308 MB, that report is now *more* plausible,
not less — Chromium should be heavier than WKWebView, not lighter. Site shows
"measuring…" rather than guessing.

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

**Same race, different asset (v0.4.0).** Ubuntu job uploaded all three Linux packages
and signatures, then died with `##[error]Not Found - .../delete-a-release-asset`
mid-way through writing `latest.json`. The three-phase split from v0.1.0 stopped jobs
racing on the *release*, but they still race on any *asset* they all write — chief
among them the merged updater manifest. Fix: `max-parallel: 1` on the build matrix in
`.github/workflows/release.yml`. Serial matrix runs ~3× longer on tagged releases, which
is fine — releases are rare and correctness beats latency. Anyone on v0.4.0 Linux is
stranded on manual updates until they re-install from v0.4.1.

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

**v0.3.0 CI — DONE, all six jobs green** (run 30619670053, July 31 09:22–09:30 UTC).
The Ubuntu job built in 6m2s with zero Rust warnings; the apt dependency list is
confirmed correct on ubuntu-22.04. Release published with nine assets:

| Platform | Assets | Size |
|---|---|---|
| macOS | `bloom_0.3.0_aarch64.dmg`, `bloom_0.3.0_x64.dmg` | 3.3 / 3.4 MB |
| Windows | `bloom_0.3.0_x64-setup.exe`, `bloom_0.3.0_x64_en-US.msi` | 2.2 / 3.3 MB |
| Linux | `bloom_0.3.0_amd64.deb`, `bloom-0.3.0-1.x86_64.rpm` | 4.4 MB each |
| Linux | `bloom_0.3.0_amd64.AppImage` | **82 MB** |

The AppImage is 25× the dmg because `linuxdeploy-plugin-gtk` bundles GTK, WebKitGTK
and gstreamer for distro portability; the deb/rpm declare them as system deps instead.
Not a bug, but it sits badly next to "ultra-lightweight" on the site — lead with
deb/rpm and offer the AppImage as the portable fallback.

v0.3.0 itself has run in CI on Ubuntu and produced valid packages, but nothing has run
this build on a real, physical Linux machine yet.

**Immediate:**
1. ~~Confirm the Linux job built.~~ Done — green.
2. Get someone with a real Linux machine to try it. Tester brief written (distro/DE/
   session-type questions, FUSE-2 gotcha, multi-process RAM command). X11 session only —
   Wayland has no equivalent to the desktop window-type hint, so it will float on top.
   Bloom prints a startup warning when it detects Wayland.
   **Suspected bug to watch for:** `lib.rs` builds the window (which shows it), *then*
   calls `set_desktop_underlay(true)`. On X11 the plugin's implementation is
   `gtk_window.set_type_hint(WindowTypeHint::Desktop)`, and most window managers read
   `_NET_WM_WINDOW_TYPE` at map time and never re-read it. So the hint may arrive too
   late and be ignored even on X11. If a tester reports "floats on top" *on an X11
   session*, this is the cause — fix is to build with `.visible(false)`, set the
   underlay, then `.show()`. Still unfixed; needs a real tester's answer first.
3. **Fixed in the workflow, not yet on the live release.** `.github/workflows/release.yml`
   no longer hardcodes "macOS and Windows" into the body — it now says "macOS, Windows,
   and Linux below, Linux is experimental." But that only applies to releases cut *after*
   this change; the live `v0.3.0` release page still has the old wrong text, since editing
   a published release wasn't done without asking. Draft replacement body still sitting in
   scratch, never applied.
4. Get the Windows RAM number. Need Task Manager screenshot + video resolution + whether
   hardware GPU scheduling is on. If it's genuinely ~800 MB, suspect software decode
   fallback in WebView2 — that's a real bug, not just WebView2 overhead. Also worth
   re-checking after the optimization pass below, which may have moved this number.
5. **Closed a different way than planned.** The two `.tar.gz` files
   (`bloom_aarch64.app.tar.gz`, `bloom_x64.app.tar.gz`) were never deleted — instead the
   updater now actually consumes them. `bundle.targets` includes `"app"` specifically so
   these get produced *and* signed. They'll keep appearing in every release from here on,
   but now they're load-bearing instead of confusing dead weight. See the updater section
   above.
6. Commit a real Activity Monitor screenshot so the README's numbers have receipts. Still
   not done. The site copy claiming this was shipping was fixed today (see Website below)
   by rewording the claim rather than by finally taking the screenshot.

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
