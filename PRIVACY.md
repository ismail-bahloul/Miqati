# Privacy Policy

Miqati is a local, offline-first desktop application. There is no account, no
telemetry, no analytics, no advertising and no tracking. The developer collects
no data about you.

## What is stored on your device

Your settings (city, coordinates, calculation method, language, time format,
reminders, window position) are saved **locally only**, in a plain JSON file:

- Windows: `%APPDATA%\Miqati\config.json`
- Linux: `~/.config/Miqati/config.json`

A backup copy (`config.bak.json`) is kept next to it. Nothing in this file
leaves your machine, except as described below.

## Network access

Prayer times are computed locally, and the application never sends any data
about you. Two requests can leave your machine, and both are disabled by
**Offline mode** in the settings, after which the application makes no network
request at all:

- **Optional city lookup.** It queries [`ip-api.com`](https://ip-api.com) to
  turn your IP address into a city, coordinates, timezone and country, so that
  the app can guess your location on first run. As with any web request, this
  shares your **IP address** (and nothing else) with that third party, whose own
  privacy policy applies. Note that the free `ip-api.com` endpoint is served
  over **plain HTTP**, so this lookup is not encrypted in transit. The request
  is skipped entirely once a location is set.
- **Update check.** Once per launch, the app asks GitHub's public releases API
  (`api.github.com`) whether a newer version has been published. It only *reads*
  public release information — no data about you is sent. This happens over
  **HTTPS**, and the result is only used to show a discreet notice: nothing is
  ever downloaded or installed without you clicking it.

No data is sent to the developer or to any server operated by this project.

## Notifications

Reminders are generated and delivered locally by Windows. No content leaves your
device.

## Children

This application is not directed at children and collects no personal data.

## Changes

Any change to this policy is committed to this repository, and therefore
reflected in its history.

## Contact

Questions or concerns: open an issue at
<https://github.com/ismail-bahloul/Miqati/issues>.
