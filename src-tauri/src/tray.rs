//! System Tray + window positioning.
//!
//! The widget "lives" in the taskbar via a tray icon. A left-click toggles the
//! compact window; the tooltip shows the current countdown. On show, the window
//! is (re)positioned near the taskbar / tray so it feels docked to the bar.

use tauri::{AppHandle, Emitter, Manager, PhysicalPosition};

use crate::{config, AppState, MAIN_WINDOW};

/// Build the tray icon (left-click toggles the main window).
pub fn build(app: &tauri::App) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItem};

    let show = MenuItem::with_id(app, "show", "Afficher / Masquer", true, None::<&str>)?;
    let dock = MenuItem::with_id(app, "dock", "Docker à la barre", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quitter", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &dock, &quit])?;

    let tray = tauri::tray::TrayIconBuilder::with_id("salaat-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Miqati")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                button_state: tauri::tray::MouseButtonState::Up,
                ..
            } = event
            {
                toggle_main(tray.app_handle());
            }
        })
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => toggle_main(app),
            "dock" => reset_dock(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    let _ = tray;

    // Dock the main window near the taskbar right away on startup.
    position_main_window(app.handle());

    Ok(())
}

/// Show/hide the main window with a fade, repositioning it near the bar when
/// shown. The frontend plays the fade and calls [`hide_window`] once it has
/// finished; a generation counter invalidates stale hide fallbacks, so a fast
/// tray double-click can never hide a freshly re-shown widget.
fn toggle_main(app: &AppHandle) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else {
        return;
    };

    let state = app.state::<AppState>();
    let mut gen = state.hide_gen.lock().unwrap();
    *gen += 1;
    let my_gen = *gen;
    drop(gen);

    if window.is_visible().unwrap_or(false) {
        let _ = window.emit_to(MAIN_WINDOW, "animate-out", ());
        // Fallback if the frontend never answers (JS not loaded yet, …).
        let app = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(400));
            let state = app.state::<AppState>();
            if *state.hide_gen.lock().unwrap() == my_gen {
                if let Some(w) = app.get_webview_window(MAIN_WINDOW) {
                    let _ = w.hide();
                }
            }
        });
    } else {
        let _ = window.show();
        position_main_window(app);
        let _ = window.emit_to(MAIN_WINDOW, "animate-in", ());
    }
}

/// Forget the dragged position and re-dock the widget against the bar
/// (tray menu: « Docker à la barre »).
fn reset_dock(app: &AppHandle) {
    {
        let state = app.state::<AppState>();
        let mut cfg = state.cfg.lock().unwrap();
        cfg.window_position = None;
        let _ = config::save(&cfg);
    }
    position_main_window(app);
}

/// Position the main window: use the user-saved position if any, otherwise
/// dock it near the taskbar (real bar position on Windows, bottom-right
/// fallback elsewhere).
pub fn position_main_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else {
        return;
    };

    // A dragged position (saved in the config) always wins.
    let cfg = app.state::<crate::AppState>().cfg.lock().unwrap().clone();
    if let Some(config::WindowPosition { x, y }) = cfg.window_position {
        let _ = window.set_position(tauri::LogicalPosition::new(x, y));
        return;
    }

    #[cfg(target_os = "windows")]
    if crate::win32::position_near_taskbar(&window) {
        return;
    }

    position_bottom_right(&window);
}

/// Generic fallback: bottom-right corner of the window's monitor, just above
/// the bar. Uses physical pixels on both sides (no DPI division — that was the
/// old bug beyond 100 % scaling).
fn position_bottom_right(window: &tauri::WebviewWindow) {
    if let Some(monitor) = window.current_monitor().ok().flatten() {
        let size = monitor.size();
        let win = window.outer_size().unwrap_or_default();
        let inset = 12.0f64;

        let x = (size.width as f64) - (win.width as f64) - inset;
        let y = (size.height as f64) - (win.height as f64) - inset;
        let _ = window.set_position(PhysicalPosition::new(x.max(0.0) as i32, y.max(0.0) as i32));
    }
}

/// Hide the main window (called by the frontend once the fade-out has played).
#[tauri::command]
pub fn hide_window(window: tauri::WebviewWindow) -> Result<(), String> {
    let _ = window.hide();
    Ok(())
}

/// Refresh the tray tooltip (called from the frontend with the countdown).
#[tauri::command]
pub fn update_tray(app: tauri::AppHandle, tooltip: String) -> Result<(), String> {
    update_tooltip(&app, &tooltip);
    Ok(())
}

/// Update the tray icon and tooltip.
pub fn update_tooltip(app: &AppHandle, text: &str) {
    if let Some(tray) = app.tray_by_id("salaat-tray") {
        let _ = tray.set_tooltip(Some(text));
    }
}
