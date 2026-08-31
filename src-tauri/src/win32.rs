//! Windows-only integration: no-activate HUD window, real taskbar-aware
//! positioning and fullscreen auto-hide.
//!
//! This module is only compiled on Windows (see the inner `cfg` attribute);
//! on other platforms the widget falls back to the generic positioning code
//! in [`crate::tray`].

#![cfg(target_os = "windows")]

use windows_sys::Win32::Foundation::{HWND, RECT};

use tauri::Manager;

use crate::MAIN_WINDOW;

/// Extract the native Win32 `HWND` from a Tauri window, via `raw-window-handle`
/// (avoids depending on the `windows` crate type used internally by Tauri).
fn native_hwnd(window: &tauri::WebviewWindow) -> Option<HWND> {
    use raw_window_handle::HasWindowHandle;
    let handle = window.window_handle().ok()?;
    match handle.as_raw() {
        raw_window_handle::RawWindowHandle::Win32(h) => {
            Some(h.hwnd.get() as *mut core::ffi::c_void)
        }
        _ => None,
    }
}

/// Raise the window above other top-level windows without activating it and
/// without making it permanently topmost. Needed when always-on-top is off, so
/// the tray "show" still brings the widget in front of other windows (otherwise
/// it would stay behind and seem "lost").
pub fn bring_to_front(window: &tauri::WebviewWindow) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    };
    let Some(hwnd) = native_hwnd(window) else {
        return;
    };
    unsafe {
        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
        SetWindowPos(
            hwnd,
            HWND_NOTOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
    }
}

/// Real taskbar rectangle via `ABM_GETTASKBARPOS`. Returns `(edge, [left, top,
/// right, bottom])` in physical pixels; `edge` is one of `ABE_*`. `None` when
/// the taskbar rect could not be obtained.
pub fn taskbar_rect() -> Option<(u32, [i32; 4])> {
    use windows_sys::Win32::UI::Shell::{SHAppBarMessage, ABM_GETTASKBARPOS, APPBARDATA};

    let mut data: APPBARDATA = unsafe { core::mem::zeroed() };
    data.cbSize = core::mem::size_of::<APPBARDATA>() as u32;
    if unsafe { SHAppBarMessage(ABM_GETTASKBARPOS, &mut data) } == 0 {
        return None;
    }
    let rc = data.rc;
    Some((data.uEdge, [rc.left, rc.top, rc.right, rc.bottom]))
}

/// Make the window a true HUD: it never takes focus or activates, and it is
/// hidden from Alt-Tab (`WS_EX_TOOLWINDOW`).
pub fn make_no_activate(window: &tauri::WebviewWindow) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongW, SetWindowLongW, GWL_EXSTYLE, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    };
    let Some(hwnd) = native_hwnd(window) else {
        return;
    };
    unsafe {
        let style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        if style == 0 {
            return; // GetWindowLongW failed; leave the window as-is.
        }
        SetWindowLongW(
            hwnd,
            GWL_EXSTYLE,
            (style as u32 | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW) as i32,
        );
    }
}

/// Position the window against the real taskbar, using
/// `SHAppBarMessage(ABM_GETTASKBARPOS)`. Handles bars on any edge
/// (bottom/top/left/right). Returns `false` when the position could not be
/// obtained, so the caller can fall back to generic positioning.
pub fn position_near_taskbar(window: &tauri::WebviewWindow) -> bool {
    use windows_sys::Win32::UI::Shell::{ABE_LEFT, ABE_RIGHT, ABE_TOP};

    let Some((edge, [left, _, right, bottom])) = taskbar_rect() else {
        return false;
    };
    let win = window.outer_size().unwrap_or_default();
    let inset = 8.0f64;

    // Dock the widget on the same side as the taskbar, aligned with its right
    // edge. On the usual bottom bar it sits ON the taskbar strip (overlapping
    // it) rather than floating above it, so it never covers app pages; its
    // height should match the taskbar height (set by the frontend).
    let (x, y) = match edge {
        ABE_TOP => (right as f64 - win.width as f64, bottom as f64 + inset),
        ABE_LEFT => (right as f64 + inset, bottom as f64 - win.height as f64),
        ABE_RIGHT => (
            left as f64 - win.width as f64 - inset,
            bottom as f64 - win.height as f64,
        ),
        // ABE_BOTTOM (and anything unknown): on the bar, right-aligned.
        _ => (
            right as f64 - win.width as f64,
            bottom as f64 - win.height as f64,
        ),
    };

    let _ = window.set_position(tauri::PhysicalPosition::new(
        x.max(0.0) as i32,
        y.max(0.0) as i32,
    ));
    true
}

/// Spawn a background poller that hides the widget while a fullscreen app is
/// in the foreground (games, video…), and restores it when leaving fullscreen.
pub fn spawn_fullscreen_watcher(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let mut hidden_by_fullscreen = false;
        loop {
            std::thread::sleep(std::time::Duration::from_millis(750));
            let Some(window) = app.get_webview_window(MAIN_WINDOW) else {
                continue;
            };
            if foreground_is_fullscreen() {
                if window.is_visible().unwrap_or(false) {
                    let _ = window.hide();
                    hidden_by_fullscreen = true;
                }
            } else if hidden_by_fullscreen {
                let _ = window.show();
                hidden_by_fullscreen = false;
            }
        }
    });
}

/// Keep the widget above the taskbar. The Windows taskbar (and its popups:
/// Start menu, flyouts…) is itself a topmost window that jumps to the front
/// when you interact with it, which would otherwise draw over the widget. A
/// light poller re-raises our window to topmost (without activating it) so it
/// stays visible above the bar at all times.
pub fn spawn_topmost_keeper(app: tauri::AppHandle) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    };
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(500));
        let Some(window) = app.get_webview_window(MAIN_WINDOW) else {
            continue;
        };
        if !window.is_visible().unwrap_or(false) {
            continue;
        }
        if let Some(hwnd) = native_hwnd(&window) {
            unsafe {
                SetWindowPos(
                    hwnd,
                    HWND_TOPMOST,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                );
            }
        }
    });
}

/// True when the foreground window covers an entire monitor (the usual
/// fullscreen heuristic: its rect equals the monitor rect).
fn foreground_is_fullscreen() -> bool {
    use windows_sys::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowRect};

    unsafe {
        let fg = GetForegroundWindow();
        if fg.is_null() {
            return false;
        }
        let mut wnd: RECT = core::mem::zeroed();
        if GetWindowRect(fg, &mut wnd) == 0 {
            return false;
        }
        let monitor = MonitorFromWindow(fg, MONITOR_DEFAULTTONEAREST);
        let mut info: MONITORINFO = core::mem::zeroed();
        info.cbSize = core::mem::size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(monitor, &mut info) == 0 {
            return false;
        }
        wnd.left == info.rcMonitor.left
            && wnd.top == info.rcMonitor.top
            && wnd.right == info.rcMonitor.right
            && wnd.bottom == info.rcMonitor.bottom
    }
}
