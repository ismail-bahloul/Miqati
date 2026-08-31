//! Tauri command surface for the widget.

use serde::Serialize;

use chrono::{Local, Timelike};
use salaat_core::{
    hijri,
    prayer_times::{
        AsrSchool, CalculationMethod, HighLatitudeRule, PrayerName, PrayerTimesBuilder,
    },
};

use crate::{config, AppState};
use tauri::{Emitter, Manager};

/// The JSON payload sent to the frontend each refresh.
#[derive(Serialize)]
pub struct StatusPayload {
    times: [f64; 6],
    next_name: String,
    remaining_seconds: u64,
    hijri: String,
    city: String,
    language: String,
    hour12: bool,
}

fn method_from(u8_idx: u8) -> CalculationMethod {
    match u8_idx {
        0 => CalculationMethod::Shia,
        1 => CalculationMethod::Karachi,
        2 => CalculationMethod::Isna,
        3 => CalculationMethod::MuslimWorldLeague,
        4 => CalculationMethod::UmmAlQura,
        5 => CalculationMethod::Egyptian,
        7 => CalculationMethod::Tehran,
        8 => CalculationMethod::Gulf,
        9 => CalculationMethod::Kuwait,
        10 => CalculationMethod::Qatar,
        11 => CalculationMethod::Singapore,
        12 => CalculationMethod::UnionOrganization,
        13 => CalculationMethod::Diyanet,
        14 => CalculationMethod::Russia,
        16 => CalculationMethod::Dubai,
        17 => CalculationMethod::Jakim,
        18 => CalculationMethod::Tunisia,
        19 => CalculationMethod::Algeria,
        20 => CalculationMethod::Kemenag,
        21 => CalculationMethod::Morocco,
        22 => CalculationMethod::Portugal,
        23 => CalculationMethod::Jordan,
        _ => CalculationMethod::UnionOrganization,
    }
}

/// UTC offset (hours) for the configured location's timezone at the given
/// instant, handling DST. Falls back to the machine's local offset when no
/// timezone is configured (or it cannot be parsed).
fn resolve_offset<Tz>(now: chrono::DateTime<Tz>, timezone: Option<&str>) -> f64
where
    Tz: chrono::TimeZone,
{
    if let Some(name) = timezone {
        if let Ok(tz) = name.parse::<chrono_tz::Tz>() {
            return offset_hours(now.with_timezone(&tz));
        }
    }
    offset_hours(now)
}

/// UTC offset in hours (local wall time minus UTC) for a given instant.
fn offset_hours<Tz: chrono::TimeZone>(dt: chrono::DateTime<Tz>) -> f64 {
    (dt.naive_local() - dt.naive_utc()).num_seconds() as f64 / 3600.0
}

fn high_lat_from(u8_idx: u8) -> HighLatitudeRule {
    match u8_idx {
        0 => HighLatitudeRule::MiddleOfNight,
        1 => HighLatitudeRule::Seventh,
        _ => HighLatitudeRule::AngleBased,
    }
}

fn school_from(u8_idx: u8) -> AsrSchool {
    match u8_idx {
        1 => AsrSchool::Hanafi,
        _ => AsrSchool::General,
    }
}

/// Compute the current status (times + next prayer + hijri) for the configured
/// location, using **the given wall-clock instant** and its current UTC offset
/// (so DST transitions are handled automatically). Pure & testable.
fn compute_status_payload(
    cfg: &crate::config::PrayerConfig,
    now: chrono::DateTime<Local>,
) -> Result<StatusPayload, String> {
    let coords = cfg.coordinates.ok_or("location not configured yet")?;
    let offset = resolve_offset(now, cfg.timezone.as_deref());

    let method = method_from(cfg.method);
    let high_lat = high_lat_from(cfg.high_lat_rule);
    let school = school_from(cfg.school);

    let builder = PrayerTimesBuilder {
        method,
        asr_school: school,
        high_lat_rule: high_lat,
    };

    // Compute today's times in the device's local zone.
    let times = builder
        .build(now.date_naive(), coords.lat, coords.lon, offset)
        .times;

    // Minutes since local midnight, right now.
    let now_min = now.hour() as f64 * 60.0 + now.minute() as f64 + now.second() as f64 / 60.0;

    // Find the next prayer. Handle same-day ordering using the raw minutes;
    // if all prayers for today have passed, roll to tomorrow's Fajr.
    let list = [
        (PrayerName::Fajr, times.fajr),
        (PrayerName::Sunrise, times.sunrise),
        (PrayerName::Dhuhr, times.dhuhr),
        (PrayerName::Asr, times.asr),
        (PrayerName::Maghrib, times.maghrib),
        (PrayerName::Isha, times.isha),
    ];
    let mut next: Option<(PrayerName, u64)> = None;
    for (name, t) in list {
        if t > now_min {
            next = Some((name, ((t - now_min) * 60.0).ceil() as u64));
            break;
        }
    }
    // Rollover: after Isha, next is tomorrow's Fajr. Compute tomorrow's Fajr.
    if next.is_none() {
        let tomorrow = now.date_naive() + chrono::Duration::days(1);
        let offset_tomorrow =
            resolve_offset(now + chrono::Duration::days(1), cfg.timezone.as_deref());
        let t_tomorrow = builder
            .build(tomorrow, coords.lat, coords.lon, offset_tomorrow)
            .times;
        let fajr_tomorrow = t_tomorrow.fajr; // in minutes
        let secs = ((fajr_tomorrow + 24.0 * 60.0 - now_min) * 60.0).ceil() as u64;
        next = Some((PrayerName::Fajr, secs));
    }

    let (next_name, remaining_seconds) = next.unwrap_or((PrayerName::Fajr, 0));

    // Hijri date, localized.
    let hij = hijri::gregorian_to_hijri(now.date_naive(), 1);
    let hijri_str = match cfg.language.as_str() {
        "ar" => hij.format(&hijri::MONTHS_AR),
        "en" => hij.format(&hijri::MONTHS_EN),
        _ => hij.format(&hijri::MONTHS_FR),
    };

    let times_arr = [
        times.fajr,
        times.sunrise,
        times.dhuhr,
        times.asr,
        times.maghrib,
        times.isha,
    ];

    Ok(StatusPayload {
        times: times_arr,
        next_name: next_name.as_str().to_string(),
        remaining_seconds,
        hijri: hijri_str,
        city: cfg.city.clone(),
        language: cfg.language.clone(),
        hour12: cfg.hour12,
    })
}

/// Tauri entry point: compute status from the configured location and "now".
#[tauri::command]
pub fn get_status(state: tauri::State<AppState>) -> Result<StatusPayload, String> {
    let cfg = state.cfg.lock().unwrap().clone();
    compute_status_payload(&cfg, Local::now())
}

/// Toggle the settings window: show it centered, or hide it if already open.
#[tauri::command]
pub fn open_settings(app: tauri::AppHandle) -> Result<(), String> {
    const SETTINGS_WINDOW: &str = "settings";

    // The window is pre-created in `setup` (see lib.rs): creating it lazily
    // here left the WebView2 child with a 0×0 size (blank window).
    if let Some(window) = app.get_webview_window(SETTINGS_WINDOW) {
        // Toggle: hide if open, otherwise center/show/focus.
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.center();
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
    Ok(())
}

/// Return the current configuration (used by the settings window).
#[tauri::command]
pub fn get_config(state: tauri::State<AppState>) -> Result<config::PrayerConfig, String> {
    Ok(state.cfg.lock().unwrap().clone())
}

/// Validate and persist a configuration submitted by the settings window,
/// keeping the OS autostart entry in sync.
#[tauri::command]
pub fn set_config(
    app: tauri::AppHandle,
    state: tauri::State<AppState>,
    mut cfg: config::PrayerConfig,
) -> Result<(), String> {
    validate_config(&cfg)?;

    let previous = state.cfg.lock().unwrap().clone();
    // The window position is only ever changed by dragging, never by the
    // settings form — preserve it across saves.
    cfg.window_position = previous.window_position;

    // Keep the OS autostart entry in sync with the setting.
    if previous.autostart != cfg.autostart {
        use tauri_plugin_autostart::ManagerExt;
        let manager = app.autolaunch();
        if cfg.autostart {
            manager.enable().map_err(|e| e.to_string())?;
        } else {
            manager.disable().map_err(|e| e.to_string())?;
        }
    }

    config::save(&cfg).map_err(|e| e.to_string())?;
    *state.cfg.lock().unwrap() = cfg;

    // Tell the widget to refresh right away (language, 12/24 h, times).
    let _ = app.emit_to(crate::MAIN_WINDOW, "config-changed", ());
    Ok(())
}

/// Save a new user-dragged window position (logical pixels; `y` = bottom edge),
/// called by the frontend after a drag gesture ends.
#[tauri::command]
pub fn save_window_position(state: tauri::State<AppState>, x: f64, y: f64) -> Result<(), String> {
    let mut cfg = state.cfg.lock().unwrap();
    cfg.window_position = Some(config::WindowPosition { x, y });
    config::save(&cfg).map_err(|e| e.to_string())
}

fn validate_config(cfg: &config::PrayerConfig) -> Result<(), String> {
    match cfg.coordinates {
        Some(c) => {
            if !(-90.0..=90.0).contains(&c.lat) || !(-180.0..=180.0).contains(&c.lon) {
                return Err("Coordonnées invalides (lat ∈ [-90, 90], lon ∈ [-180, 180])".into());
            }
        }
        None => return Err("Indiquez une ville (latitude/longitude requises)".into()),
    }
    if !matches!(cfg.language.as_str(), "fr" | "en" | "ar") {
        return Err("Langue invalide".into());
    }
    if cfg.school > 1 || cfg.high_lat_rule > 2 {
        return Err("Réglage école / hautes latitudes invalide".into());
    }
    Ok(())
}

#[tauri::command]
pub fn quit_app(app: tauri::AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}

/// Result of IP-based geolocation.
#[derive(Serialize)]
pub struct DetectedLocation {
    pub city: String,
    pub lat: f64,
    pub lon: f64,
    /// IANA timezone of the detected location (e.g. "Africa/Casablanca").
    pub timezone: Option<String>,
    /// Recommended AlAdhan method index for the detected country.
    pub method: u8,
}

/// Recommended method (AlAdhan index) per country, used on geolocation.
fn country_method(cc: &str) -> u8 {
    match cc {
        "FR" => 12,
        "MA" => 21,
        "DZ" => 19,
        "TN" => 18,
        "SA" => 4,
        "QA" => 10,
        "KW" => 9,
        "AE" => 16,
        "MY" => 17,
        "ID" => 20,
        "RU" => 14,
        "TR" => 13,
        "PT" => 22,
        "JO" => 23,
        "EG" => 5,
        "PK" => 1,
        "IR" => 0,
        "SG" => 11,
        "US" => 2,
        "CA" => 2,
        "GB" => 3,
        _ => 3, // Muslim World League
    }
}

/// Detect the user's approximate location by IP (ip-api.com). Done in Rust so
/// it is not subject to the webview's "mixed content" restrictions and works
/// reliably on first launch. Returns city/coordinates/timezone + recommended
/// method.
#[tauri::command]
pub fn detect_location() -> Result<DetectedLocation, String> {
    let url = "http://ip-api.com/json/?fields=status,city,lat,lon,countryCode,timezone&lang=fr";
    let text = ureq::get(url)
        .timeout(std::time::Duration::from_secs(8))
        .call()
        .map_err(|e| e.to_string())?
        .into_string()
        .map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    if v["status"].as_str() != Some("success") {
        let msg = v["message"].as_str().unwrap_or("geolocation failed");
        return Err(msg.to_string());
    }
    Ok(DetectedLocation {
        city: v["city"].as_str().unwrap_or("").to_string(),
        lat: v["lat"].as_f64().unwrap_or(0.0),
        lon: v["lon"].as_f64().unwrap_or(0.0),
        timezone: v["timezone"].as_str().map(|s| s.to_string()),
        method: country_method(v["countryCode"].as_str().unwrap_or("")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Coordinates;

    fn cfg_paris() -> crate::config::PrayerConfig {
        crate::config::PrayerConfig {
            method: 12, // UOIF
            school: 0,
            high_lat_rule: 2,
            language: "fr".into(),
            hour12: false,
            coordinates: Some(Coordinates {
                lat: 48.8534,
                lon: 2.3488,
            }),
            city: "Paris".into(),
            autostart: false,
            start_hidden: false,
            timezone: None,
            window_position: None,
        }
    }

    #[test]
    fn status_for_paris_produces_sane_values() {
        // Fixed "now": 2026-01-10 12:30 local (UTC+1 winter).
        use chrono::{FixedOffset, TimeZone, Utc};
        let tz = FixedOffset::east_opt(3600).unwrap();
        let now = tz
            .with_ymd_and_hms(2026, 1, 10, 12, 30, 0)
            .unwrap()
            .with_timezone(&Utc)
            .with_timezone(&Local);

        let payload = compute_status_payload(&cfg_paris(), now).expect("computes");
        // Dhuhr should be just about now (12:58 AlAdhan), so next prayer is
        // Dhuhr or soon after; times array must be ordered.
        let t = payload.times;
        assert!(t[0] < t[1] && t[1] < t[2] && t[2] < t[3] && t[3] < t[4] && t[4] < t[5]);
        assert!(!payload.hijri.is_empty());
        // nextName must be a known prayer
        assert!(matches!(
            payload.next_name.as_str(),
            "Fajr" | "Sunrise" | "Dhuhr" | "Asr" | "Maghrib" | "Isha"
        ));
        assert_eq!(payload.city, "Paris");
    }

    #[test]
    fn status_without_location_errors() {
        use chrono::{FixedOffset, TimeZone, Utc};
        let tz = FixedOffset::east_opt(0).unwrap();
        let now = tz
            .with_ymd_and_hms(2026, 1, 1, 12, 0, 0)
            .unwrap()
            .with_timezone(&Utc)
            .with_timezone(&Local);
        let mut cfg = cfg_paris();
        cfg.coordinates = None;
        assert!(compute_status_payload(&cfg, now).is_err());
    }

    #[test]
    fn resolve_offset_uses_city_timezone() {
        use chrono::{FixedOffset, TimeZone};
        let tz = FixedOffset::east_opt(3600).unwrap();
        let now = tz.with_ymd_and_hms(2026, 8, 31, 12, 0, 0).unwrap();
        // Paris is UTC+2 in summer (DST), independent of the machine's offset.
        assert_eq!(resolve_offset(now, Some("Europe/Paris")), 2.0);
        // No timezone configured -> machine offset (+1 here).
        assert_eq!(resolve_offset(now, None), 1.0);
    }

    #[test]
    fn country_method_maps_common() {
        assert_eq!(country_method("MA"), 21);
        assert_eq!(country_method("FR"), 12);
        assert_eq!(country_method("DZ"), 19);
        assert_eq!(country_method("TN"), 18);
        assert_eq!(country_method("SA"), 4);
        assert_eq!(country_method("US"), 2);
        assert_eq!(country_method("XX"), 3); // unknown -> MWL
        assert_eq!(country_method(""), 3);
    }

    #[test]
    fn validate_accepts_paris() {
        assert!(validate_config(&cfg_paris()).is_ok());
    }

    #[test]
    fn validate_rejects_bad_coordinates() {
        let mut cfg = cfg_paris();
        cfg.coordinates = Some(Coordinates {
            lat: 95.0,
            lon: 2.0,
        });
        assert!(validate_config(&cfg).is_err());
        cfg.coordinates = Some(Coordinates {
            lat: 48.0,
            lon: 200.0,
        });
        assert!(validate_config(&cfg).is_err());
        cfg.coordinates = None;
        assert!(validate_config(&cfg).is_err());
    }

    #[test]
    fn validate_rejects_bad_language_or_school() {
        let mut cfg = cfg_paris();
        cfg.language = "de".into();
        assert!(validate_config(&cfg).is_err());
        cfg.language = "fr".into();
        cfg.school = 9;
        assert!(validate_config(&cfg).is_err());
        cfg.school = 0;
        cfg.high_lat_rule = 7;
        assert!(validate_config(&cfg).is_err());
    }
}
