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
