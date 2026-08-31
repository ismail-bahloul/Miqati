//! Miqati — Tauri backend.
//!
//! Thin Windows/UI layer. All calculation lives in [`salaat_core`].

mod commands;
mod config;
mod tray;
#[cfg(target_os = "windows")]
mod win32;

use std::sync::Mutex;
use tauri::Manager;

/// Label of the widget (main) window, shared across modules.
pub const MAIN_WINDOW: &str = "main";

/// Application-global, mutable per-app state shared across commands.
pub struct AppState {
    /// Location + settings, read from disk and cached.
    pub cfg: Mutex<config::PrayerConfig>,
    /// Generation counter for show/hide requests (see `tray::toggle_main`).
    pub hide_gen: Mutex<u64>,
}

pub fn run() {
    let cfg = config::load();
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(AppState {
            cfg: Mutex::new(cfg),
            hide_gen: Mutex::new(0),
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::get_config,
            commands::set_config,
            commands::open_settings,
            commands::save_window_position,
            commands::detect_location,
            commands::quit_app,
            tray::hide_window,
            tray::update_tray
        ])
        .setup(|app| {
            // Keep the OS autostart entry in sync with the saved setting.
            use tauri_plugin_autostart::ManagerExt;
            let autostart = app.state::<AppState>().cfg.lock().unwrap().autostart;
            let manager = app.autolaunch();
            if autostart {
                let _ = manager.enable();
            } else {
                let _ = manager.disable();
            }

            tray::build(app)?;

            // Pre-create the settings window hidden. Creating it lazily from a
            // command left the WebView2 child with a 0×0 size (blank window);
            // creating it here, on the main thread with its final size, avoids
            // that init race. `open_settings` only shows it.
            let _ = tauri::WebviewWindowBuilder::new(
                app,
                "settings",
                tauri::WebviewUrl::App("settings.html".into()),
            )
            .title("Réglages — Miqati")
            .inner_size(420.0, 660.0)
            .min_inner_size(380.0, 620.0)
            .max_inner_size(480.0, 740.0)
            .resizable(true)
            .maximizable(false)
            .visible(false)
            .build();

            // Intercept the title-bar X: hide the window instead of destroying
            // it, so reopening it stays cheap and the WebView2 keeps its size.
            if let Some(settings) = app.get_webview_window("settings") {
                let settings_for_event = settings.clone();
                settings.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = settings_for_event.hide();
                    }
                });
            }

            // Apply the saved always-on-top preference to the widget window.
            let always_on_top = app.state::<AppState>().cfg.lock().unwrap().always_on_top;
            if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
                let _ = window.set_always_on_top(always_on_top);
            }

            // Start hidden in the tray if requested (tray click still shows it).
            if app.state::<AppState>().cfg.lock().unwrap().start_hidden {
                if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
                    let _ = window.hide();
                }
            }

            #[cfg(target_os = "windows")]
            {
                if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
                    win32::make_no_activate(&window);
                }
                win32::spawn_fullscreen_watcher(app.handle().clone());
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running salaat-widget");
}
