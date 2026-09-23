<p align="center">
  <img src="assets/banner.png" alt="Miqati" width="100%">
</p>

<p align="center">
  Prayer times and a live countdown to the next salah, right on your Windows taskbar.
</p>

<p align="center">
  <a href="https://github.com/ismail-bahloul/Miqati/releases/latest/download/Miqati_x64-setup.exe"><b>Download Miqati for Windows</b></a>
</p>

<p align="center">
  <sub>Windows 10 and 11, no administrator rights needed. Free and open source.</sub>
</p>

Miqati is a small widget that lives on your taskbar. It shows the next prayer and how long
is left before it, so you always know where you stand without reaching for your phone.

Click it to open today's times and the Hijri date. It never takes the focus away from
whatever you are doing, and it calculates everything on your own computer: no account, no
server, no tracking.

## Preview

<div align="center">
  <img src="assets/screenshot-compact.png" alt="The widget on the taskbar" width="280">
  <br><br>
  <img src="assets/screenshot-detail.png" alt="Today's times in the expanded view" width="280">
</div>

## Install

1. Click **Download Miqati for Windows** at the top. It fetches the installer in one go. If you would rather read the release notes first, the [latest release](https://github.com/ismail-bahloul/Miqati/releases/latest) page has the same file.
2. Run the installer you downloaded. Windows may ask you to confirm.
3. Miqati starts right away and puts an icon in the notification area, next to the clock.

There is nothing to set up before you start: on first launch Miqati works out your city from
your connection and picks the calculation method used there. You can change any of it later
in the settings.

> **Windows may warn you the first time.** SmartScreen shows a "Windows protected your PC"
> prompt for any new program it has not seen before. Click **More info**, then **Run anyway**.
> See [Code signing](#code-signing) below for the longer explanation.

## Using Miqati

- **Click the widget** to open today's times. Click again to close them.
- **Drag it** to move it anywhere on the screen. The tray menu can snap it back onto the taskbar.
- **Right-click the tray icon** to show or hide the widget, to re-dock it, or to quit.
- **Reminders** are optional and can pop a notification a few minutes before each prayer, at
  the time of the prayer, or both. They only fire from the installed version, and only if you
  turn them on in the settings.
- **Updates**: when a newer version is out, Miqati says so in the detail view and in the
  settings. It only checks, it never installs anything on its own: you download the new
  installer from the release page when you want it.

## Settings

Open the settings from the detail view. The everyday options are up front:

- your **city**: start typing a name and pick it from the list, which covers more than 12 000
  cities and towns worldwide, Morocco's included, or use the magnifier button to detect where
  you are;
- the **language**: French, English or Arabic;
- the **time format**: 12 or 24 hours;
- your **reminders** and how long before a prayer they should appear.

Behind "Advanced" you will find the coordinates, the calculation method, the Asr school, the
high latitude rule and the timezone, for anyone who wants to match their local mosque's
timetable. Everything is saved as you change it.

## Your privacy

Miqati computes prayer times locally, and never sends anything about you anywhere. Two
requests can leave your machine, and **Offline mode** in the settings turns both off, after
which the application makes no network request at all:

- the optional city detection, which asks `ip-api.com` to turn your IP address into a city;
- a check for new releases, which only reads public release information from GitHub.

[PRIVACY.md](PRIVACY.md) has the details, including which endpoint is encrypted.

## Something not working?

| What you see | What to do |
|---|---|
| Windows warned me about the app | It is SmartScreen being cautious; see [Install](#install). |
| No reminder notifications | They only fire from the installed version, and only if they are enabled in the settings. |
| The times look wrong for my city | Check the calculation method and the timezone under "Advanced". The magnifier button re-detects your city and picks the method used there. |
| Anything else | Please [open an issue](https://github.com/ismail-bahloul/Miqati/issues). Your city, your Windows version and what you expected are enough to make it actionable, and a screenshot helps. |

## Verifying the download

Each release ships a `SHA256SUMS.txt`. The installer is published twice under the same
digest: `Miqati_x64-setup.exe`, the stable name the button above downloads, and
`Miqati_<version>_x64-setup.exe`. Every copy carries a build provenance attestation that
ties it to this repository's CI:

```bash
gh attestation verify <installer.exe> -R ismail-bahloul/Miqati
```

## Code signing

These installers are not signed yet, which is why SmartScreen may warn the first time you
run one (see [Install](#install)). Signing them through the
[SignPath Foundation](https://signpath.org/), which offers it free to open-source projects,
is planned.

## Building from source

You will need [Rust](https://rustup.rs), a Windows toolchain, and the
[WebView2 runtime](https://developer.microsoft.com/microsoft-edge/webview2/).

```bash
cargo build --release
cargo tauri build
```

## License

Released under the [GPL-3.0](LICENSE) license. Free and open source. © Ismail Bahloul

The prayer-time engine (`salaat-core`) is a port of the GPL-3.0 [`salaatprayertime`](https://github.com/MazenMohamed203/salaatprayertime) KDE widget.

The city list behind the settings picker is derived from [GeoNames](https://www.geonames.org/),
licensed [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).
