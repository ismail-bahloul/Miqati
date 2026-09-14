<p align="center">
  <img src="assets/banner.png" alt="Miqati" width="100%">
</p>

<p align="center">
  <strong>Miqati</strong> · your prayer times, always within reach on the Windows taskbar
</p>

---

A compact widget that keeps your prayer times and the countdown to the next salah visible on top of the Windows taskbar, without ever stealing focus. Times are computed locally; the only network use is the optional city detection, which can be turned off.

## Preview

<div align="center">
  <img src="assets/screenshot-compact.png" alt="Compact widget" width="280">
  <br><br>
  <img src="assets/screenshot-detail.png" alt="Detail view" width="280">
</div>

## Features

- Always-on-top HUD widget, docked to the taskbar.
- Live countdown to the next prayer.
- Detail view with today's times and the Hijri date.
- Prayer times are always computed **locally** — no account, no server, no
  tracking.
- Automatic location detection (city, timezone, calculation method) by IP. This
  is the only network request the app makes, and it can be turned off with
  **Offline mode** in the settings, after which nothing leaves your machine.
- Multilingual interface (English, French, Arabic).
- Optional start with Windows.

## Installation

Download the latest installer from the [Releases](https://github.com/ismail-bahloul/Miqati/releases) page and run it. No administrator rights are required.

> **Windows may warn you.** Miqati is not code-signed yet, so SmartScreen can
> show a "Windows protected your PC" prompt on first run: click **More info** →
> **Run anyway**. It only means Microsoft has not seen the file often enough to
> vouch for it.

## Usage

- Click the widget to toggle the detail view.
- Drag it to move it; the tray menu can snap it back to the taskbar.
- The tray icon shows or hides the widget, and quits the app.

## Verifying the download

Each release ships a `SHA256SUMS.txt`, and every installer carries a build
provenance attestation tying it to this repository's CI:

```bash
gh attestation verify <installer.exe> -R ismail-bahloul/Miqati
```

## Something wrong?

- **Windows warned me about the app** — expected while it is unsigned; see the
  note under Installation.
- **No notifications** — they only fire from the *installed* build (the
  installer creates the Start menu shortcut Windows needs for them), and only
  if reminders are enabled in the settings.
- **The time looks wrong for my city** — check the calculation method and the
  timezone under "Advanced"; the magnifier button re-detects your city and picks
  the method used there.
- **Anything else** — please
  [open an issue](https://github.com/ismail-bahloul/Miqati/issues). Naming your
  city, your Windows version and what you expected is enough to make it
  actionable; a screenshot helps.

## Building

Requirements: [Rust](https://rustup.rs), a Windows toolchain, and the [WebView2 runtime](https://developer.microsoft.com/microsoft-edge/webview2/).

```bash
cargo build --release
cargo tauri build
```

## License

Released under the [GPL-3.0](LICENSE) license. Free and open source. © Ismail Bahloul

The prayer-time engine (`salaat-core`) is a port of the GPL-3.0 [`salaatprayertime`](https://github.com/MazenMohamed203/salaatprayertime) KDE widget.
