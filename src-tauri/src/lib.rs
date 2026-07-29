use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_desktop_underlay::DesktopUnderlayExt;
use tauri_plugin_dialog::DialogExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_desktop_underlay::init())
        .setup(|app| {
            // Menu bar app only — no Dock icon, no app switcher entry.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

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
            let quit = MenuItem::with_id(app, "quit", "Quit Bloom", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&choose, &garden, &toggle, &quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .icon_as_template(false)
                .menu(&menu)
                .show_menu_on_left_click(true)
                .tooltip("Bloom — live wallpaper")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
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
