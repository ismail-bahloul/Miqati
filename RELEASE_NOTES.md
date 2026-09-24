## Miqati v0.2.2

Fixes on top of v0.2.1. **This is the first build cut from the current
source**, so it supersedes v0.2.1 — download this one.

### Fixed
- **"in 1 minute", not "in 1 minutes"**: the reminder text is now singular where
  it should be, in both English and French.
- **The tray's position reset actually works**: "Dock to taskbar" is now
  **"Reset position"**. It always brings the widget back — including when it was
  hidden — and places it clear of both the screen edge and the taskbar, instead of
  silently doing nothing when the widget was hidden.
- **No more vanishing widget right after install**: the fullscreen-hide watcher
  now waits 5 seconds after launch before it can hide anything, so a closing
  installer or a UAC prompt can no longer make the widget disappear before you
  have seen it.

### Installation
Download and run `Miqati_0.2.2_x64-setup.exe` (no administrator rights required). Windows SmartScreen may ask for confirmation (the app is not code-signed): click "More info" then "Run anyway".

### Verifying the download
Both published installers share the same SHA-256:
`04d6c68eedf557c2a2a611bf29925ee40a99b8ddfcc68fcc478ecd59a004f554`

VirusTotal report: <https://www.virustotal.com/gui/file/04d6c68eedf557c2a2a611bf29925ee40a99b8ddfcc68fcc478ecd59a004f554> — **3 of 70 engines**, all three generic machine-learning heuristics (Sophos *Generic ML PUA*, SecureAge, Arctic Wolf). Microsoft, ESET, Kaspersky, BitDefender and every other major engine report it **clean**; unsigned installers routinely get picked up that way.

The build also carries a GitHub provenance attestation tying it to this repository:

```bash
gh attestation verify Miqati_x64-setup.exe -R ismail-bahloul/Miqati
```

## Miqati v0.2.1

### Added
- **City picker**: the city field in the settings now suggests towns as you type
  (12 000+ cities, including every Moroccan town). Picking one fills in the
  coordinates, the timezone and the country's usual calculation method, and a
  line under the field spells out the position the app will actually use.
- **Update notice**: the widget tells you when a newer version has been published
  (a discreet line in the detail view, and in the settings). It only checks:
  nothing is downloaded or installed without your click, and it stays silent in
  offline mode.

### Fixed
- **Correct times after a clock-rule change**: prayer times were an hour off for
  a country that changes its timezone rules after the app was built (Morocco
  moved back to UTC+0 on 20 September 2026, while the bundled timezone data
  still said UTC+1). When the configured city is the machine's own time zone,
  the app now follows the operating system's timezone rules, which the OS keeps
  up to date.
- **Timezones outside the short list are no longer dropped** when the settings
  are saved.

### Changed
- Smaller, tidier build: removed an unused dependency and a few unused window
  permissions, and dropped duplicated image files.

### Installation
Download and run `Miqati_0.2.1_x64-setup.exe` (no administrator rights required). Windows SmartScreen may ask for confirmation (the app is not code-signed): click "More info" then "Run anyway".

## Miqati v0.2.0

Second release of **Miqati**, a prayer-times widget docked above the Windows taskbar.

### What's new
- **Native notifications**: an optional reminder before the prayer time, at the
  prayer time, or both. They keep firing while the widget is hidden, and survive
  sleep and a change of city.
- **Offline mode**: turn off automatic location detection and the app makes no
  network request at all.
- **Simpler settings**: the everyday options (city, language, time format,
  reminders) are up front; the technical ones (coordinates, calculation method,
  Asr school, high-latitude rule, timezone) live behind "Advanced".
- **More resilient configuration**: a single invalid field no longer wipes the
  whole file, and a backup copy is kept.
- **Reliability fixes**: the countdown no longer freezes after sleep, the widget
  hides correctly for fullscreen apps (and stays hidden if you hid it yourself),
  and it stays above the taskbar.

### Installation
Download and run `Miqati_0.2.0_x64-setup.exe` (no administrator rights required). Windows SmartScreen may ask for confirmation (the app is not code-signed): click "More info" then "Run anyway".

## Miqati v0.1.0

First public release of **Miqati**, a prayer-times widget docked above the Windows taskbar.

### Features
- Compact widget with the next prayer + live countdown, and a detail view with all prayers and the Hijri date
- Docked above the taskbar, movable, bottom-anchored (compact / detail)
- Automatic IP-based location (city, coordinates, timezone, country calculation method)
- Times computed from the city's timezone (independent of the machine's timezone)
- Languages: Français / English / العربية (with RTL layout)
- Auto-saved, localized settings
- Notification-area icon (show/hide, re-dock, quit)
- "Always on top" option
- Start with Windows / start hidden (optional)

### Installation
Download and run `Miqati_0.1.0_x64-setup.exe` (no administrator rights required). Windows SmartScreen may ask for confirmation (the app is not code-signed): click "More info" then "Run anyway".
