//! Salaat Widget — Tauri backend.
//!
//! Thin Windows/UI layer. All calculation lives in [`salaat_core`].

mod commands;
mod config;
mod tray;

use std::sync::Mutex;

/// Application-global, mutable per-app state shared across commands.
pub struct AppState {
    /// Location + settings, read from disk and cached.
    pub cfg: Mutex<config::PrayerConfig>,
}

pub fn run() {
    let cfg = config::load();
    tauri::Builder::default()
        .manage(AppState {
            cfg: Mutex::new(cfg),
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::open_settings,
            commands::quit_app,
            tray::update_tray
        ])
        .setup(|app| {
            tray::build(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running salaat-widget");
}
