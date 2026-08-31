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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Coordinates {
    pub lat: f64,
    pub lon: f64,
}

/// A saved window position, in logical pixels (DPI-independent).
/// `y` is the BOTTOM edge of the window (height-independent anchor), so the
/// widget keeps its vertical spot whether compact or expanded.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WindowPosition {
    pub x: f64,
    pub y: f64,
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
    /// Last user-dragged position (logical px; `y` = bottom edge); `None` = docked near the bar.
    pub window_position: Option<WindowPosition>,
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
            window_position: None,
        }
    }
}

/// Load configuration from disk, or fall back to defaults if absent/invalid.
pub fn load() -> PrayerConfig {
    let path = config_file();
    if let Ok(text) = std::fs::read_to_string(&path) {
        // Tolerate a UTF-8 BOM (Notepad / PowerShell -Encoding UTF8 add one),
        // which would otherwise make serde_json fail and silently reset the
        // whole configuration to defaults.
        let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
        if let Ok(cfg) = serde_json::from_str::<PrayerConfig>(text) {
            return cfg;
        }
    }
    PrayerConfig::default()
}

pub fn save(cfg: &PrayerConfig) -> std::io::Result<()> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)?;
    let text = serde_json::to_string_pretty(cfg)?;
    std::fs::write(config_file(), text)
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
        assert!(cfg.window_position.is_none());
    }

    #[test]
    fn defaults_roundtrip() {
        let cfg = PrayerConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let back: PrayerConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.autostart, false);
        assert_eq!(back.start_hidden, false);
    }
}
