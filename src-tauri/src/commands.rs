//! Tauri command surface for the widget.

use serde::Serialize;

use chrono::{Local, Timelike};
use salaat_core::{
    hijri::{self, MONTHS_FR},
    prayer_times::{
        AsrSchool, CalculationMethod, HighLatitudeRule, PrayerName, PrayerTimesBuilder,
    },
};

use crate::AppState;

/// The JSON payload sent to the frontend each refresh.
#[derive(Serialize)]
pub struct StatusPayload {
    times: [f64; 6],
    next_name: String,
    remaining_seconds: u64,
    hijri: String,
    city: String,
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
        _ => CalculationMethod::UnionOrganization,
    }
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
    let offset = now.offset().local_minus_utc() as f64 / 3600.0;

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
        let t_tomorrow = builder
            .build(tomorrow, coords.lat, coords.lon, offset)
            .times;
        let fajr_tomorrow = t_tomorrow.fajr; // in minutes
        let secs = ((fajr_tomorrow + 24.0 * 60.0 - now_min) * 60.0).ceil() as u64;
        next = Some((PrayerName::Fajr, secs));
    }

    let (next_name, remaining_seconds) = next.unwrap_or((PrayerName::Fajr, 0));

    // Hijri date.
    let hij = hijri::gregorian_to_hijri(now.date_naive(), 1);
    let hijri_str = format!("{} {}", hij.day, MONTHS_FR[(hij.month - 1) as usize]);

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
    })
}

/// Tauri entry point: compute status from the configured location and "now".
#[tauri::command]
pub fn get_status(state: tauri::State<AppState>) -> Result<StatusPayload, String> {
    let cfg = state.cfg.lock().unwrap().clone();
    compute_status_payload(&cfg, Local::now())
}

#[tauri::command]
pub fn open_settings(_app: tauri::AppHandle) -> Result<(), String> {
    // TODO(step 3): open a settings window.
    Ok(())
}

#[tauri::command]
pub fn quit_app(app: tauri::AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
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
}
