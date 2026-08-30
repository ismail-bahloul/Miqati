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
            .join("SalaatWidget")
    }
    #[cfg(not(target_os = "windows"))]
    {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("SalaatWidget")
    }
}

fn config_file() -> PathBuf {
    config_dir().join("config.json")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Coordinates {
    pub lat: f64,
    pub lon: f64,
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
        }
    }
}

/// Load configuration from disk, or fall back to defaults if absent/invalid.
pub fn load() -> PrayerConfig {
    let path = config_file();
    if let Ok(text) = std::fs::read_to_string(&path) {
        if let Ok(cfg) = serde_json::from_str::<PrayerConfig>(&text) {
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
