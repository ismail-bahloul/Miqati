//! Windows-only integration: no-activate HUD window, real taskbar-aware
//! positioning and fullscreen auto-hide.
//!
//! This module is only compiled on Windows (see the inner `cfg` attribute);
//! on other platforms the widget falls back to the generic positioning code
//! in [`crate::tray`].

#![cfg(target_os = "windows")]

use windows_sys::Win32::Foundation::{HWND, RECT};

use tauri::Manager;

use crate::{AppState, MAIN_WINDOW};

/// Poll interval of the background window watcher.
const WATCH_INTERVAL: std::time::Duration = std::time::Duration::from_millis(300);

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

/// Position the window just above the real taskbar, using
/// `SHAppBarMessage(ABM_GETTASKBARPOS)`. Handles bars on any edge
/// (bottom/top/left/right). Returns `false` when the position could not be
/// obtained, so the caller can fall back to generic positioning.
///
/// The widget is anchored so its edge *touches* the inner edge of the taskbar
/// but never overlaps it: the taskbar is composited by the shell in a layer
/// above normal windows, so any overlap would be drawn over the widget and the
/// covered part would look "cut off".
pub fn position_near_taskbar(window: &tauri::WebviewWindow) -> bool {
    use windows_sys::Win32::UI::Shell::{ABE_LEFT, ABE_RIGHT, ABE_TOP};

    let Some((edge, [left, top, right, bottom])) = taskbar_rect() else {
        return false;
    };
    let win = window.outer_size().unwrap_or_default();
    let w = win.width as f64;
    let h = win.height as f64;
    let inset = 8.0f64;

    // Anchor against the bar's INNER edge (the side facing the workspace) so the
    // widget sits fully off the bar. On the usual bottom bar that is the bar's
    // top (`top`), not its bottom.
    let (x, y) = match edge {
        // Bar on top: the inner edge is its bottom.
        ABE_TOP => (right as f64 - w, bottom as f64 + inset),
        // Bar on the left: inner edge is its right.
        ABE_LEFT => (right as f64 + inset, bottom as f64 - h),
        // Bar on the right: inner edge is its left.
        ABE_RIGHT => (left as f64 - w - inset, bottom as f64 - h),
        // ABE_BOTTOM (and anything unknown): inner edge is the bar's top.
        _ => (right as f64 - w, top as f64 - h - inset),
    };

    // Clamp inside the monitor so a docked widget can never go off-screen.
    let _ = window.set_position(tauri::PhysicalPosition::new(
        x.max(0.0) as i32,
        y.max(0.0) as i32,
    ));
    true
}

/// Single background watcher for the widget window. It does two jobs on one
/// thread (they used to be two pollers):
///
/// 1. **Stay above the taskbar.** The Windows taskbar and its flyouts are
///    themselves topmost windows, and the shell raises them to the top of the
///    topmost band whenever the user touches the bar. Our window is
///    `WS_EX_NOACTIVATE`, so it can never be activated and win that race on its
///    own: it sinks under the bar and stays there. We detect the case and
///    re-raise it (see [`raise_above`]).
/// 2. **Hide while a fullscreen app is in the foreground** (games, video…) and
///    restore it on exit.
pub fn spawn_window_watcher(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let mut hidden_by_fullscreen = false;
        loop {
            std::thread::sleep(WATCH_INTERVAL);

            let Some(window) = app.get_webview_window(MAIN_WINDOW) else {
                continue;
            };

            if foreground_is_fullscreen() {
                // Hide unconditionally: the widget may have been shown (tray,
                // settings) *while* the fullscreen app was already running, and
                // a one-shot check would leave it on top of the video.
                if window.is_visible().unwrap_or(false) {
                    let _ = window.hide();
                }
                hidden_by_fullscreen = true;
                continue; // nothing to raise while hidden
            } else if hidden_by_fullscreen {
                let _ = window.show();
                hidden_by_fullscreen = false;
            }

            if !window.is_visible().unwrap_or(false) {
                continue;
            }
            // With always-on-top off the widget is meant to be coverable:
            // re-raising it here would fight the user's own setting.
            if !app
                .state::<AppState>()
                .cfg
                .lock()
                .map(|c| c.always_on_top)
                .unwrap_or(false)
            {
                continue;
            }
            let Some(hwnd) = native_hwnd(&window) else {
                continue;
            };
            // If a shell surface is in front of us, bring the widget back to the
            // top of the topmost band. `raise_above` stays inside the band, so
            // unlike the NOTOPMOST→TOPMOST pair it does not flicker.
            if shell_above(hwnd).is_some() {
                raise_above(hwnd);
            }
        }
    });
}

/// If a Windows shell surface (taskbar, Start menu, a flyout) sits *above* our
/// window in the z-order, return its handle.
///
/// `WindowFromPoint` cannot answer this: even with the shell visibly drawn on
/// top, it reports our own window at the probe point (the shell is composited
/// by the DWM outside the normal hit-test order). Walking the z-order chain
/// upwards from our window does reflect reality, so we look for a shell window
/// before leaving the topmost band.
fn shell_above(hwnd: HWND) -> Option<HWND> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetClassNameW, GetWindow, GetWindowLongW, GWL_EXSTYLE, GW_HWNDPREV, WS_EX_TOPMOST,
    };

    unsafe {
        let mut cur = hwnd;
        // Bound the walk: only the topmost band matters, and it is short.
        for _ in 0..64 {
            cur = GetWindow(cur, GW_HWNDPREV);
            if cur.is_null() {
                return None;
            }
            // Leaving the topmost band means nothing topmost is above us.
            let style = GetWindowLongW(cur, GWL_EXSTYLE) as u32;
            if style & WS_EX_TOPMOST == 0 {
                return None;
            }
            let mut buf = [0u16; 64];
            let len = GetClassNameW(cur, buf.as_mut_ptr(), buf.len() as i32);
            if len <= 0 {
                continue;
            }
            let class = String::from_utf16_lossy(&buf[..len as usize]);
            if matches!(
                class.as_str(),
                "Shell_TrayWnd"                          // primary taskbar
                | "Shell_SecondaryTrayWnd"              // taskbar on other monitors
                | "Windows.UI.Core.CoreWindow"          // Start / search
                | "XamlExplorerHostIslandWindow"        // flyouts (wifi, battery, calendar)
                | "TopLevelWindowForOverflowXamlIsland" // tray overflow / action centre
            ) {
                return Some(cur);
            }
        }
    }
    None
}

/// Bring the window to the top of its own z-order band, without activating it.
///
/// `HWND_TOP` re-orders within the window's current band: for our topmost
/// widget that puts it in front of the taskbar and its flyouts **without** ever
/// leaving the topmost band, so — unlike the NOTOPMOST→TOPMOST pair — there is
/// no intermediate repaint and therefore no flicker.
///
/// Note: passing another window's handle as `hWndInsertAfter` would be wrong
/// here — that handle *precedes* us in the z-order, which sinks the widget
/// *below* the taskbar instead of raising it.
fn raise_above(hwnd: HWND) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_TOP, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    };
    unsafe {
        SetWindowPos(
            hwnd,
            HWND_TOP,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
    }
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
        // The desktop and the shell also span the whole monitor, so a plain
        // geometry check would report "fullscreen" as soon as the user clicks
        // the desktop or the taskbar — hiding the widget for no reason. Those
        // surfaces are never a fullscreen app: exclude them by class.
        if is_shell_window(fg) {
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
        // Tolerate a few pixels: borderless-fullscreen players can leave a
        // hairline border, and DPI rounding is not always exact.
        const TOL: i32 = 2;
        (wnd.left - info.rcMonitor.left).abs() <= TOL
            && (wnd.top - info.rcMonitor.top).abs() <= TOL
            && (wnd.right - info.rcMonitor.right).abs() <= TOL
            && (wnd.bottom - info.rcMonitor.bottom).abs() <= TOL
    }
}

/// True when the window is a Windows shell surface (desktop, taskbar, Start
/// menu, flyouts) rather than a regular application window. These are drawn
/// across the whole monitor but are not "fullscreen apps".
fn is_shell_window(hwnd: HWND) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::GetClassNameW;
    unsafe {
        let mut buf = [0u16; 64];
        let len = GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
        if len <= 0 {
            return true; // no class: not something we should hide for
        }
        let class = String::from_utf16_lossy(&buf[..len as usize]);
        matches!(
            class.as_str(),
            "Progman"                                // desktop
            | "WorkerW"                             // desktop (wallpaper host)
            | "Shell_TrayWnd"                       // primary taskbar
            | "Shell_SecondaryTrayWnd"              // taskbar on other monitors
            | "Windows.UI.Core.CoreWindow"          // Start / search
            | "XamlExplorerHostIslandWindow"        // flyouts (wifi, battery, calendar)
            | "TopLevelWindowForOverflowXamlIsland" // tray overflow / action centre
            | "SysListView32"                       // desktop icons
        )
    }
}
