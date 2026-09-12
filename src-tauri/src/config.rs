//! Persistent configuration for the widget (location, calculation method…).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Where the app stores its data/config, following the OS convention
/// (e.g. `%APPDATA%` on Windows, `~/.config`/platform data dir elsewhere).
pub fn config_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Miqati")
    }
    #[cfg(not(target_os = "windows"))]
    {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Miqati")
    }
}

fn config_file() -> PathBuf {
    config_dir().join("config.json")
}

/// Backup copy of the last good config, used to recover from a corrupted
/// `config.json` instead of silently resetting every setting.
fn backup_file() -> PathBuf {
    config_dir().join("config.bak.json")
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Coordinates {
    pub lat: f64,
    pub lon: f64,
}

/// A saved window position, in logical pixels (DPI-independent).
/// `y` is the BOTTOM edge of the window (height-independent anchor), so the
/// widget keeps its vertical spot whether compact or expanded.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindowPosition {
    pub x: f64,
    pub y: f64,
}

/// Per-prayer manual adjustments, in minutes (can be negative). Applied after
/// the astronomical computation, so users can match their local mosque's
/// timetable (the most requested tweak for this kind of widget).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PrayerOffsets {
    pub fajr: i16,
    pub sunrise: i16,
    pub dhuhr: i16,
    pub asr: i16,
    pub maghrib: i16,
    pub isha: i16,
}

impl Default for PrayerOffsets {
    fn default() -> Self {
        Self {
            fajr: 0,
            sunrise: 0,
            dhuhr: 0,
            asr: 0,
            maghrib: 0,
            isha: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)] // tolerate config files written by older versions (missing fields)
pub struct PrayerConfig {
    /// Method id matching [`salaat_core::CalculationMethod`]'s AlAdhan index.
    pub method: u8,
    pub school: u8,
    pub high_lat_rule: u8,
    pub language: String, // "fr" | "ar" | "en"
    pub hour12: bool,
    pub coordinates: Option<Coordinates>,
    /// Display name of the chosen location (city label).
    pub city: String,
    /// Launch the app at system startup (autostart entry).
    pub autostart: bool,
    /// Start hidden in the tray instead of visible.
    pub start_hidden: bool,
    /// IANA timezone of the configured location (e.g. "Africa/Casablanca");
    /// None falls back to the machine's local timezone offset.
    pub timezone: Option<String>,
    /// Keep the widget above other windows.
    #[serde(default = "default_true")]
    pub always_on_top: bool,
    /// Last user-dragged position (logical px; `y` = bottom edge); `None` = docked near the bar.
    pub window_position: Option<WindowPosition>,
    /// Notify when a prayer time comes in.
    #[serde(default = "default_true")]
    pub notify_at_time: bool,
    /// Notify shortly before a prayer time (see [`Self::notify_before_minutes`]).
    #[serde(default = "default_true")]
    pub notify_before: bool,
    /// How long before the prayer the early reminder fires, in minutes.
    #[serde(default = "default_before_minutes")]
    pub notify_before_minutes: u8,
    /// Manual per-prayer adjustments, in minutes.
    pub offsets: PrayerOffsets,
    /// Day adjustment applied to the Hijri date (−2…+2). The tabular civil
    /// calendar can differ from the locally observed one by a day or two, so
    /// users match it to their country's calendar. Defaults to `1`, which is
    /// what the app applied unconditionally before this became a setting.
    #[serde(default = "default_hijri_adjust")]
    pub hijri_adjust: i32,
    /// Offline mode: when on, the app makes no network request on its own
    /// (no automatic city detection). Prayer times are always computed locally,
    /// so the widget stays fully functional either way. Explicit user actions
    /// (the “use my location” button) still work.
    pub offline_mode: bool,
}

fn default_true() -> bool {
    true
}

fn default_before_minutes() -> u8 {
    10
}

fn default_hijri_adjust() -> i32 {
    1
}

impl Default for PrayerConfig {
    fn default() -> Self {
        Self {
            method: 12,       // UOIF (France) — sensible default for the primary audience
            school: 0,        // standard (Shafi'i)
            high_lat_rule: 2, // angle-based
            language: "fr".into(),
            hour12: false,
            coordinates: None,
            city: String::new(),
            autostart: false,
            start_hidden: false,
            timezone: None,
            always_on_top: true,
            window_position: None,
            notify_at_time: true,
            notify_before: true,
            notify_before_minutes: default_before_minutes(),
            offsets: PrayerOffsets::default(),
            hijri_adjust: default_hijri_adjust(),
            offline_mode: false,
        }
    }
}

/// Parse a `PrayerConfig` from a JSON value **field by field**, falling back to
/// the default for any field that is missing or has an unexpected type/range.
///
/// This is deliberately lenient: a single bad value (say a `notify_before_minutes`
/// of 300, or a stray string where a number is expected) used to make the whole
/// `serde_json::from_str` fail, so `load()` returned `PrayerConfig::default()` —
/// silently dropping the user's city, coordinates and timezone. Recovering the
/// valid fields keeps the important settings alive.
fn from_value_lenient(v: &serde_json::Value) -> PrayerConfig {
    let d = PrayerConfig::default();
    let get_u8 = |key: &str, fallback: u8| {
        v.get(key).and_then(|x| x.as_u64()).map(|n| n.min(255) as u8).unwrap_or(fallback)
    };
    let get_bool = |key: &str, fallback: bool| {
        v.get(key).and_then(|x| x.as_bool()).unwrap_or(fallback)
    };
    let get_string = |key: &str, fallback: String| {
        v.get(key)
            .and_then(|x| x.as_str())
            .map(str::to_owned)
            .unwrap_or(fallback)
    };

    PrayerConfig {
        method: get_u8("method", d.method),
        school: get_u8("school", d.school),
        high_lat_rule: get_u8("high_lat_rule", d.high_lat_rule),
        language: get_string("language", d.language),
        hour12: get_bool("hour12", d.hour12),
        coordinates: v
            .get("coordinates")
            .and_then(|c| {
                Some(Coordinates {
                    lat: c.get("lat")?.as_f64()?,
                    lon: c.get("lon")?.as_f64()?,
                })
            }),
        city: get_string("city", d.city),
        autostart: get_bool("autostart", d.autostart),
        start_hidden: get_bool("start_hidden", d.start_hidden),
        timezone: v
            .get("timezone")
            .and_then(|x| x.as_str())
            .map(str::to_owned),
        always_on_top: get_bool("always_on_top", d.always_on_top),
        window_position: v.get("window_position").and_then(|p| {
            Some(WindowPosition {
                x: p.get("x")?.as_f64()?,
                y: p.get("y")?.as_f64()?,
            })
        }),
        notify_at_time: get_bool("notify_at_time", d.notify_at_time),
        notify_before: get_bool("notify_before", d.notify_before),
        notify_before_minutes: get_u8("notify_before_minutes", d.notify_before_minutes),
        offsets: v
            .get("offsets")
            .map(|o| {
                let get = |key: &str| o.get(key).and_then(|x| x.as_i64());
                PrayerOffsets {
                    fajr: get("fajr").unwrap_or(0) as i16,
                    sunrise: get("sunrise").unwrap_or(0) as i16,
                    dhuhr: get("dhuhr").unwrap_or(0) as i16,
                    asr: get("asr").unwrap_or(0) as i16,
                    maghrib: get("maghrib").unwrap_or(0) as i16,
                    isha: get("isha").unwrap_or(0) as i16,
                }
            })
            .unwrap_or(d.offsets),
        hijri_adjust: v
            .get("hijri_adjust")
            .and_then(|x| x.as_i64())
            .map(|n| n.clamp(-2, 2) as i32)
            .unwrap_or(d.hijri_adjust),
        offline_mode: get_bool("offline_mode", d.offline_mode),
    }
}

/// Read and parse a config file (tolerating a UTF-8 BOM, added by Notepad /
/// PowerShell `-Encoding UTF8`).
fn read_config(path: &std::path::Path) -> Option<PrayerConfig> {
    let text = std::fs::read_to_string(path).ok()?;
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    Some(from_value_lenient(&value))
}

/// Load configuration from disk.
///
/// Order of preference: the main file, then the backup written by [`save`].
/// Only when neither is readable do we fall back to the built-in defaults.
pub fn load() -> PrayerConfig {
    read_config(&config_file())
        .or_else(|| read_config(&backup_file()))
        .unwrap_or_default()
}

pub fn save(cfg: &PrayerConfig) -> std::io::Result<()> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)?;

    // Keep the previous good file as a backup *before* replacing it, so a bad
    // write (or a later hand-edit gone wrong) never loses the settings.
    let main = config_file();
    if main.exists() {
        let _ = std::fs::copy(&main, backup_file());
    }

    let text = serde_json::to_string_pretty(cfg)?;
    std::fs::write(main, text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_config_without_new_fields_loads_with_defaults() {
        // Config file written by a previous version: new fields must default.
        let json = r#"{"method":3,"school":1,"high_lat_rule":0,"language":"en","hour12":true,"coordinates":{"lat":10.0,"lon":20.0},"city":"Test"}"#;
        let cfg: PrayerConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.method, 3);
        assert_eq!(cfg.school, 1);
        assert_eq!(cfg.language, "en");
        assert!(!cfg.autostart);
        assert!(!cfg.start_hidden);
        assert!(cfg.always_on_top);
        assert!(cfg.window_position.is_none());
        // Notifications are opt-out: an existing config must gain them enabled.
        assert!(cfg.notify_at_time);
        assert!(cfg.notify_before);
        assert_eq!(cfg.notify_before_minutes, 10);
    }

    #[test]
    fn defaults_roundtrip() {
        let cfg = PrayerConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let back: PrayerConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.autostart, false);
        assert_eq!(back.start_hidden, false);
    }

    #[test]
    fn out_of_range_field_keeps_the_rest_of_the_config() {
        // Regression: a `notify_before_minutes` that does not fit a u8 used to
        // fail the whole parse, silently resetting city/coordinates.
        let v: serde_json::Value = serde_json::from_str(
            r#"{"method":21,"city":"Tétouan","notify_before_minutes":300,
                "coordinates":{"lat":35.57,"lon":-5.37},"timezone":"Africa/Casablanca"}"#,
        )
        .unwrap();
        let cfg = from_value_lenient(&v);
        assert_eq!(cfg.method, 21);
        assert_eq!(cfg.city, "Tétouan");
        assert_eq!(cfg.timezone.as_deref(), Some("Africa/Casablanca"));
        assert!(cfg.coordinates.is_some());
        // The bad field is clamped rather than dropping everything.
        assert_eq!(cfg.notify_before_minutes, 255);
    }

    #[test]
    fn wrong_type_field_keeps_the_rest_of_the_config() {
        let v: serde_json::Value = serde_json::from_str(
            r#"{"city":"Paris","hour12":"yes","coordinates":{"lat":48.85,"lon":2.35}}"#,
        )
        .unwrap();
        let cfg = from_value_lenient(&v);
        assert_eq!(cfg.city, "Paris");
        assert!(cfg.coordinates.is_some());
        // A non-boolean falls back to the default instead of failing.
        assert!(!cfg.hour12);
    }

    #[test]
    fn lenient_parse_matches_serde_on_a_valid_config() {
        let original = PrayerConfig {
            method: 21,
            city: "Tétouan".into(),
            coordinates: Some(Coordinates {
                lat: 35.57,
                lon: -5.37,
            }),
            timezone: Some("Africa/Casablanca".into()),
            notify_before_minutes: 15,
            ..PrayerConfig::default()
        };
        let v = serde_json::to_value(&original).unwrap();
        let lenient = from_value_lenient(&v);
        let strict: PrayerConfig = serde_json::from_value(v).unwrap();
        assert_eq!(lenient.method, strict.method);
        assert_eq!(lenient.city, strict.city);
        assert_eq!(lenient.notify_before_minutes, strict.notify_before_minutes);
        assert_eq!(lenient.timezone, strict.timezone);
        assert_eq!(lenient.always_on_top, strict.always_on_top);
    }

    #[test]
    fn offsets_default_to_zero_and_survive_a_roundtrip() {
        // Absent in an older config file: neutral offsets, not an error.
        let v: serde_json::Value = serde_json::from_str(r#"{"city":"Paris"}"#).unwrap();
        let cfg = from_value_lenient(&v);
        assert_eq!(cfg.offsets, PrayerOffsets::default());

        // Present and negative values must be kept as-is.
        let v: serde_json::Value = serde_json::from_str(
            r#"{"city":"Paris","offsets":{"fajr":-5,"maghrib":3,"isha":-2}}"#,
        )
        .unwrap();
        let cfg = from_value_lenient(&v);
        assert_eq!(cfg.offsets.fajr, -5);
        assert_eq!(cfg.offsets.maghrib, 3);
        assert_eq!(cfg.offsets.isha, -2);
        assert_eq!(cfg.offsets.dhuhr, 0);

        // A malformed entry falls back to zero without dropping the rest.
        let v: serde_json::Value =
            serde_json::from_str(r#"{"city":"Paris","offsets":{"fajr":"oops","asr":4}}"#)
                .unwrap();
        let cfg = from_value_lenient(&v);
        assert_eq!(cfg.offsets.fajr, 0);
        assert_eq!(cfg.offsets.asr, 4);
    }

    #[test]
    fn hijri_adjust_defaults_to_one_and_is_clamped() {
        // Legacy default: the app used to apply +1 unconditionally, so an
        // existing config must not start showing a different date.
        let v: serde_json::Value = serde_json::from_str(r#"{"city":"Paris"}"#).unwrap();
        assert_eq!(from_value_lenient(&v).hijri_adjust, 1);

        // Explicit values in range are kept.
        for n in -2..=2 {
            let v: serde_json::Value =
                serde_json::from_str(&format!(r#"{{"city":"Paris","hijri_adjust":{n}}}"#))
                    .unwrap();
            assert_eq!(from_value_lenient(&v).hijri_adjust, n);
        }

        // Out-of-range values are clamped rather than dropping the config.
        let v: serde_json::Value = serde_json::from_str(
            r#"{"city":"Paris","hijri_adjust":40}"#,
        )
        .unwrap();
        let cfg = from_value_lenient(&v);
        assert_eq!(cfg.hijri_adjust, 2);
        assert_eq!(cfg.city, "Paris");
    }

    #[test]
    fn offline_mode_defaults_to_off_and_survives_a_roundtrip() {
        // Online by default: an existing config must keep detecting the city.
        let v: serde_json::Value = serde_json::from_str(r#"{"city":"Paris"}"#).unwrap();
        assert!(!from_value_lenient(&v).offline_mode);

        let v: serde_json::Value =
            serde_json::from_str(r#"{"city":"Paris","offline_mode":true}"#).unwrap();
        assert!(from_value_lenient(&v).offline_mode);
    }
}
