<p align="center">
  <img src="assets/banner.png" alt="Miqati" width="100%">
</p>

<p align="center">
  <strong>Miqati</strong> — your prayer times, always within reach on the Windows taskbar.
</p>

---

A compact, offline-first widget that keeps your prayer times and the countdown to the next salah visible on top of the Windows taskbar, without ever stealing focus.

## Features

- Always-on-top HUD widget, docked to the taskbar.
- Live countdown to the next prayer.
- Detail view with today's times and the Hijri date.
- Fully offline — times are computed locally.
- Automatic location detection (city, timezone, calculation method).
- Multilingual interface (English, French, Arabic).
- Optional start with Windows.

## Installation

Download the latest installer from the [Releases](https://github.com/ismail-bahloul/Miqati/releases) page and run it. No administrator rights are required.

> The app is not code-signed yet — on first launch Windows may ask you to click "More info" then "Run anyway".

## Usage

- Click the widget to toggle the detail view.
- Drag it to move it; the tray menu can snap it back to the taskbar.
- The tray icon shows or hides the widget, and quits the app.

## Building

Requirements: [Rust](https://rustup.rs), a Windows toolchain, and the [WebView2 runtime](https://developer.microsoft.com/microsoft-edge/webview2/).

```bash
cargo build --release
cargo tauri build
```

## License

To be defined. © Ismail Bahloul
