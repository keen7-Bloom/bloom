use tauri::menu::{CheckMenuItem, Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_desktop_underlay::DesktopUnderlayExt;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_updater::UpdaterExt;

/// An update that has been found but not yet installed.
///
/// Bloom has no window to prompt in, so the tray menu item *is* the prompt: finding an
/// update only relabels it to "Install update v…". Nothing is downloaded or replaced
/// until the user picks it.
#[derive(Default)]
struct PendingUpdate(std::sync::Mutex<Option<tauri_plugin_updater::Update>>);

/// Ask GitHub whether a newer release exists. Never installs.
async fn check_for_update(app: tauri::AppHandle, item: MenuItem<tauri::Wry>) {
    let updater = match app.updater() {
        Ok(u) => u,
        Err(e) => {
            eprintln!("bloom: updater unavailable: {e}");
            return;
        }
    };

    match updater.check().await {
        Ok(Some(update)) => {
            let version = update.version.clone();
            eprintln!("bloom: update available: v{version}");
            let _ = item.set_text(format!("Install update v{version}…"));
            *app.state::<PendingUpdate>().0.lock().unwrap() = Some(update);
        }
        Ok(None) => eprintln!("bloom: already up to date"),
        Err(e) => eprintln!("bloom: update check failed: {e}"),
    }
}

/// Install the update the user just clicked, then relaunch into it.
async fn install_update(app: tauri::AppHandle, item: MenuItem<tauri::Wry>) {
    // Take the update out and drop the lock before awaiting — a MutexGuard must not
    // be held across an await point.
    let pending = {
        let state = app.state::<PendingUpdate>();
        let mut guard = state.0.lock().unwrap();
        guard.take()
    };

    let Some(update) = pending else {
        // Nothing staged, so this click is a manual "check now".
        check_for_update(app, item).await;
        return;
    };

    let version = update.version.clone();
    let _ = item.set_text("Installing update…");
    if let Err(e) = update.download_and_install(|_, _| {}, || {}).await {
        eprintln!("bloom: update to v{version} failed: {e}");
        // download_and_install consumed the update, so nothing is staged any more.
        // Relabel to invite a retry — that click re-checks and re-stages from scratch.
        let _ = item.set_text(format!("Retry update v{version}…"));
        return;
    }
    app.restart();
}

/// True when the machine is running on battery power.
#[cfg(target_os = "macos")]
fn on_battery() -> bool {
    std::process::Command::new("pmset")
        .args(["-g", "batt"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("'Battery Power'"))
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
fn on_battery() -> bool {
    use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
    unsafe {
        let mut status: SYSTEM_POWER_STATUS = std::mem::zeroed();
        // ACLineStatus: 0 = battery, 1 = AC, 255 = unknown
        GetSystemPowerStatus(&mut status) != 0 && status.ACLineStatus == 0
    }
}

/// Reads sysfs: any mains adapter reporting `online = 0` means we're on battery.
/// Desktops with no adapter and no battery report false.
#[cfg(target_os = "linux")]
fn on_battery() -> bool {
    let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") else {
        return false;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let kind = std::fs::read_to_string(path.join("type")).unwrap_or_default();
        if kind.trim() == "Mains" {
            if let Ok(online) = std::fs::read_to_string(path.join("online")) {
                return online.trim() == "0";
            }
        }
    }
    false
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn on_battery() -> bool {
    false
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_desktop_underlay::init())
        .setup(|app| {
            // Menu bar app only — no Dock icon, no app switcher entry.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // The desktop window-type hint we rely on is X11-only. Wayland has no
            // equivalent protocol, so the wallpaper will float instead of sitting
            // behind icons. Say so rather than letting it look broken.
            #[cfg(target_os = "linux")]
            if std::env::var("XDG_SESSION_TYPE").as_deref() == Ok("wayland") {
                eprintln!(
                    "bloom: Wayland session detected. Placing a window behind desktop \
                     icons is X11-only, so the wallpaper will not sit behind your icons \
                     here. Log in with an X11/Xorg session for the intended behaviour."
                );
            }

            // One wallpaper window covering the primary monitor.
            let monitor = app
                .primary_monitor()?
                .expect("no primary monitor detected");
            let size = monitor.size();
            let pos = monitor.position();
            let scale = monitor.scale_factor();

            let wallpaper = WebviewWindowBuilder::new(
                app,
                "wallpaper",
                WebviewUrl::App("wallpaper.html".into()),
            )
            .title("Bloom")
            .decorations(false)
            .resizable(false)
            .maximizable(false)
            .minimizable(false)
            .closable(false)
            .shadow(false)
            .skip_taskbar(true)
            .position(pos.x as f64 / scale, pos.y as f64 / scale)
            .inner_size(size.width as f64 / scale, size.height as f64 / scale)
            .build()?;

            // The two calls that make Bloom exist:
            wallpaper.set_desktop_underlay(true)?; // behind desktop icons
            wallpaper.set_ignore_cursor_events(true)?; // clicks pass through

            // Menu bar tray.
            let choose = MenuItem::with_id(app, "choose", "Choose Video…", true, None::<&str>)?;
            let garden = MenuItem::with_id(app, "garden", "Garden Scene", true, None::<&str>)?;
            let toggle = MenuItem::with_id(app, "toggle", "Pause / Resume", true, None::<&str>)?;
            let battpause =
                CheckMenuItem::with_id(app, "battpause", "Pause on Battery", true, true, None::<&str>)?;
            let update = MenuItem::with_id(app, "update", "Check for Updates…", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit Bloom", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&choose, &garden, &toggle, &battpause, &update, &quit])?;

            app.manage(PendingUpdate::default());

            // Look for an update at startup, in the background so it never delays the
            // wallpaper. This only relabels the tray item — nothing installs until the
            // user clicks it.
            {
                let handle = app.handle().clone();
                let item = update.clone();
                tauri::async_runtime::spawn(check_for_update(handle, item));
            }

            // Watch power state; tell the wallpaper when it changes.
            {
                let app_handle = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    let mut last: Option<bool> = None;
                    loop {
                        let now = on_battery();
                        if last != Some(now) {
                            last = Some(now);
                            if let Some(w) = app_handle.get_webview_window("wallpaper") {
                                let _ = w.emit("bloom://power", if now { "battery" } else { "ac" });
                            }
                        }
                        std::thread::sleep(std::time::Duration::from_secs(15));
                    }
                });
            }

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .icon_as_template(false)
                .menu(&menu)
                .show_menu_on_left_click(true)
                .tooltip("Bloom — live wallpaper")
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "update" => {
                        tauri::async_runtime::spawn(install_update(app.clone(), update.clone()));
                    }
                    "battpause" => {
                        if let Some(w) = app.get_webview_window("wallpaper") {
                            let _ = w.emit("bloom://battery-pref", battpause.is_checked().unwrap_or(true));
                        }
                    }
                    "toggle" => {
                        if let Some(w) = app.get_webview_window("wallpaper") {
                            let _ = w.emit("bloom://toggle", ());
                        }
                    }
                    "garden" => {
                        if let Some(w) = app.get_webview_window("wallpaper") {
                            let _ = w.emit("bloom://garden", ());
                        }
                    }
                    "choose" => {
                        let app = app.clone();
                        app.clone()
                            .dialog()
                            .file()
                            .add_filter("Videos", &["mp4", "webm", "mov", "m4v"])
                            .pick_file(move |path| {
                                if let Some(path) = path {
                                    if let Some(w) = app.get_webview_window("wallpaper") {
                                        let _ = w.emit(
                                            "bloom://set-video",
                                            path.to_string(),
                                        );
                                    }
                                }
                            });
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
