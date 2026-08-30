//! Entry point and Tauri command surface for the widget.
//!
//! This is the thin Windows/UI layer. All calculation logic lives in the
//! `salaat-core` crate so it stays pure, testable and platform-agnostic.

pub fn run() {
    // Placeholder backend. The full Tauri setup (window, tray, commands)
    // is wired in by the next step.
    println!("salaat-widget backend starting (scaffold)");
}
