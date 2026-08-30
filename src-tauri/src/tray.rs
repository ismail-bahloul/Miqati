//! System Tray + window positioning.
//!
//! The widget "lives" in the taskbar via a tray icon. A left-click toggles the
//! compact window; the tooltip shows the current countdown. On show, the window
//! is (re)positioned near the taskbar / tray so it feels docked to the bar.

use tauri::{AppHandle, Manager, PhysicalPosition};

/// Ids of the tray and the main window.
const MAIN_WINDOW: &str = "main";

/// Build the tray icon (left-click toggles the main window).
pub fn build(app: &tauri::App) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItem};

    let show = MenuItem::with_id(app, "show", "Afficher / Masquer", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quitter", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    let tray = tauri::tray::TrayIconBuilder::with_id("salaat-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Salaat Widget")
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
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    let _ = tray;
    Ok(())
}

/// Show/hide the main window, repositioning it near the bar when shown.
fn toggle_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
            position_near_tray(&window);
        }
    }
}

/// Position the (compact) window at the bottom-right of the primary screen,
/// just above the taskbar area — visually "docked" to the tray zone.
fn position_near_tray(window: &tauri::WebviewWindow) {
    if let Some(monitor) = window.current_monitor().ok().flatten() {
        let size = monitor.size();
        let scale = monitor.scale_factor();
        let win = window.outer_size().unwrap_or_default();

        // Place in the bottom-right corner with a small inset above the bar.
        // (A precise taskbar-aware placement can be refined on Windows.)
        let x = ((size.width as f64) - (win.width as f64)) / scale - 12.0;
        let y = ((size.height as f64) - (win.height as f64)) / scale - 48.0;
        let _ = window.set_position(PhysicalPosition::new(x.max(0.0) as i32, y.max(0.0) as i32));
    }
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
