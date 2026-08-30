//! Astronomical calculation of prayer times.
//!
//! Ported from [`OfflinePrayerCalc.js`] in the KDE `salaatprayertime` widget,
//! itself following the AlAdhan (PrayTimes) astronomical method set.
//!
//! [`OfflinePrayerCalc.js`]: https://github.com/iswad-lab/salaatprayertime

use chrono::{Datelike, NaiveDate};

/// High-latitude resolution rule, following the PrayTimes convention
/// (`highLatRules`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighLatitudeRule {
    /// Middle of the night method.
    MiddleOfNight,
    /// One seventh of the night method (7):
    /// `isha = sunset + (fajr - sunset) / 7`, `fajr` unchanged.
    Seventh,
    /// Angle-based method (2):
    /// `isha = maghrib + isha_angle`, `fajr = dhuhr - fajr_angle` with normalisation.
    AngleBased,
}

/// The school used for the Asr calculation shadow factor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsrSchool {
    /// Standard (Shafi'i, Maliki, Hanbali): shadow = object length + shadow.
    General,
    /// This component: shadow = 2 × object length (Hanafi).
    Hanafi,
}

/// Widely used calculation authorities.
///
/// The numeric `index` mirrors the AlAdhan method table used by the KDE widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalculationMethod {
    /// Jafari (Shia Ithna-Ashari): Fajr 16°, Isha 14°.
    Shia,
    /// University of Islamic Sciences, Karachi: Fajr 18°, Isha 18°.
    Karachi,
    /// Islamic Society of North America: Fajr 15°, Isha 15°.
    Isna,
    /// Muslim World League: Fajr 18°, Isha 17°.
    MuslimWorldLeague,
    /// Umm Al-Qura, Makkah: Fajr 18.5°, Isha 90 min after Maghrib.
    UmmAlQura,
    /// Egyptian General Authority of Survey: Fajr 19.5°, Isha 17.5°.
    Egyptian,
    /// University of Tehran: Fajr 17.7°, Isha 14°.
    Tehran,
    /// Gulf Region: Fajr 19.5°, Isha 17.5°.
    Gulf,
    /// Kuwait: Fajr 18°, Isha 17.5°.
    Kuwait,
    /// Qatar: Fajr 18°, Isha 18°.
    Qatar,
    /// Majlis Ugama Islam Singapura, Singapore.
    Singapore,
    /// Union Organization islamic de France (UOIF): Fajr 12°, Isha 12°.
    /// (Note: uses Fajr/Isha angle 12° as per the current AlAdhan configuration.)
    UnionOrganization,
    /// Diyanet İşleri Başkanlığı, Turkey: Fajr 18°, Isha 17°.
    Diyanet,
    /// Spiritual Administration of Muslims of Russia: Fajr 18°, Isha 17°.
    Russia,
}

impl CalculationMethod {
    /// The Fajr and Isha angles (degrees) for this method.
    ///
    /// Returns `(fajr_angle, isha_angle)`. An `isha_angle` of `90.0` is a
    /// sentinel meaning "90 minutes after Maghrib" (Umm Al-Qura style).
    pub fn angles(self) -> (f64, f64) {
        use CalculationMethod::*;
        match self {
            Shia => (16.0, 14.0),
            Karachi => (18.0, 18.0),
            Isna => (15.0, 15.0),
            MuslimWorldLeague => (18.0, 17.0),
            UmmAlQura => (18.5, 90.0),
            Egyptian => (19.5, 17.5),
            Tehran => (17.7, 14.0),
            Gulf => (19.5, 17.5),
            Kuwait => (18.0, 17.5),
            Qatar => (18.0, 18.0),
            Singapore => (18.0, 18.0),
            UnionOrganization => (12.0, 12.0),
            Diyanet => (18.0, 17.0),
            Russia => (18.0, 17.0),
        }
    }

    /// The AlAdhan numeric index, kept for compatibility with the KDE widget
    /// configuration and Mawaqit later.
    pub fn index(self) -> u8 {
        use CalculationMethod::*;
        match self {
            Shia => 0,
            Karachi => 1,
            Isna => 2,
            MuslimWorldLeague => 3,
            UmmAlQura => 4,
            Egyptian => 5,
            Tehran => 7,
            Gulf => 8,
            Kuwait => 9,
            Qatar => 10,
            Singapore => 11,
            UnionOrganization => 12,
            Diyanet => 13,
            Russia => 14,
        }
    }
}

impl Default for CalculationMethod {
    fn default() -> Self {
        CalculationMethod::UnionOrganization
    }
}

/// One day's requested prayer times, as minutes since local midnight.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DayTimes {
    /// Fajr start (minutes).
    pub fajr: f64,
    /// Sunrise (minutes).
    pub sunrise: f64,
    /// Dhuhr (Zawal) (minutes).
    pub dhuhr: f64,
    /// Asr (minutes).
    pub asr: f64,
    /// Maghrib (minutes).
    pub maghrib: f64,
    /// Isha (minutes).
    pub isha: f64,
}

/// Container for the computed result plus the requested parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrayerTimes {
    pub times: DayTimes,
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub lat: f64,
    pub lon: f64,
    pub tz_offset: f64,
    pub method: CalculationMethod,
    pub asr_school: AsrSchool,
    pub high_lat_rule: HighLatitudeRule,
}

/// Builder-style configuration for a prayer-time computation.
#[derive(Debug, Clone, Copy)]
pub struct PrayerTimesBuilder {
    pub method: CalculationMethod,
    pub asr_school: AsrSchool,
    pub high_lat_rule: HighLatitudeRule,
}

impl Default for PrayerTimesBuilder {
    fn default() -> Self {
        Self {
            method: CalculationMethod::default(),
            asr_school: AsrSchool::General,
            high_lat_rule: HighLatitudeRule::MiddleOfNight,
        }
    }
}

// ---------------------------------------------------------------------------
// Math helpers
// ---------------------------------------------------------------------------

#[inline]
fn dtr(d: f64) -> f64 {
    d.to_radians()
}

#[inline]
fn rtd(r: f64) -> f64 {
    r.to_degrees()
}

#[inline]
fn sin_d(d: f64) -> f64 {
    dtr(d).sin()
}

#[inline]
fn cos_d(d: f64) -> f64 {
    dtr(d).cos()
}

#[inline]
fn tan_d(d: f64) -> f64 {
    dtr(d).tan()
}

#[inline]
fn arcsin_d(d: f64) -> f64 {
    rtd(d.asin())
}

#[inline]
fn arccos_d(d: f64) -> f64 {
    rtd(d.acos())
}

#[inline]
fn arccot_d(x: f64) -> f64 {
    rtd((1.0 / x).atan())
}

/// Normalise an angle to `[0, 360)`.
#[inline]
fn fix_angle(a: f64) -> f64 {
    let a = a - 360.0 * (a / 360.0).floor();
    if a < 0.0 {
        a + 360.0
    } else {
        a
    }
}

/// Normalise a time-of-day (in hours) to `[0, 24)`.
#[inline]
fn fix_hour(a: f64) -> f64 {
    let a = a - 24.0 * (a / 24.0).floor();
    if a < 0.0 {
        a + 24.0
    } else {
        a
    }
}

/// Julian date number for an astronomical calendar date.
fn julian_date(year: i32, month: i32, day: i32) -> f64 {
    let (mut year, mut month) = (year, month);
    if month <= 2 {
        year -= 1;
        month += 12;
    }
    let a = (year as f64 / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();
    (365.25 * (year as f64 + 4716.0)).floor()
        + (30.6001 * (month as f64 + 1.0)).floor()
        + day as f64
        + b
        - 1524.5
}

/// Solar declination (degrees) and equation of time (hours) for a Julian date.
fn sun_position(jd: f64) -> (f64, f64) {
    let d = jd - 2451545.0;
    let g = fix_angle(357.529 + 0.98560028 * d);
    let q = fix_angle(280.459 + 0.98564736 * d);
    let l = fix_angle(q + 1.915 * sin_d(g) + 0.020 * sin_d(2.0 * g));
    let e = 23.439 - 0.00000036 * d;
    let decl = arcsin_d(sin_d(e) * sin_d(l));
    let mut ra = arccos_d(cos_d(l) / cos_d(decl));
    ra = fix_angle(ra);
    if sin_d(l) < 0.0 {
        ra = 360.0 - ra;
    }
    let eqt = q / 15.0 - ra / 15.0;
    (decl, eqt)
}

/// The hour angle delta (in hours) for a given solar altitude `g`.
fn compute_time(lat: f64, decl: f64, g: f64) -> f64 {
    let z = fix_angle(arccos_d(
        (sin_d(g) - sin_d(lat) * sin_d(decl)) / (cos_d(lat) * cos_d(decl)),
    ));
    z / 15.0
}

fn asr_altitude(lat: f64, decl: f64, school: AsrSchool) -> f64 {
    let factor = match school {
        AsrSchool::General => 1.0,
        AsrSchool::Hanafi => 2.0,
    };
    arccot_d(factor + tan_d((lat - decl).abs()))
}

/// Format raw minutes-since-midnight into a civil `"HH:MM"` clock string using
/// the requested 12/24h style.
///
/// `minutes` is expected to already be in the local (tz-adjusted) time.
pub fn format_clock(minutes: f64, hour12: bool) -> String {
    let m0 = (minutes - (minutes / 60.0).floor() * 60.0).round() as i64;
    let mut h = (minutes / 60.0).floor() as i64 % 24;
    let m = if m0 >= 60 { 0 } else { m0 };
    if m0 >= 60 {
        h = (h + 1) % 24;
    }
    if hour12 {
        if h == 0 {
            format!("12:{:02} AM", m)
        } else if h < 12 {
            format!("{}:{:02} AM", h, m)
        } else if h == 12 {
            format!("12:{:02} PM", m)
        } else {
            format!("{}:{:02} PM", h - 12, m)
        }
    } else {
        format!("{:02}:{:02}", h, m)
    }
}

// ---------------------------------------------------------------------------
// Main calculation
// ---------------------------------------------------------------------------

/// Apply the high-latitude rule to a single prayer (`raw_minutes`), following
/// the PrayTimes convention of using a portion of the night measured from
/// sunset. Only applied when the raw value is invalid (NaN).
///
/// `is_fajr` selects the correct anchor: Fajr is anchored from mid-day, Isha
/// from sunset.
fn adjust_high_lat(
    raw_minutes: f64,
    rule: &HighLatitudeRule,
    angle: f64,
    sunset: f64,
    mid_day: f64,
    night: f64,
    is_fajr: bool,
) -> f64 {
    if raw_minutes.is_finite() {
        return raw_minutes;
    }
    match rule {
        HighLatitudeRule::MiddleOfNight => {
            if is_fajr {
                sunset - night / 2.0
            } else {
                sunset + night / 2.0
            }
        }
        // One seventh of the night from sunset (symmetrical for both prayers).
        HighLatitudeRule::Seventh => {
            if is_fajr {
                sunset - night / 7.0
            } else {
                sunset + night / 7.0
            }
        }
        // Angle-based: nudge the prayer by its own angle in hours from the
        // relevant temporal anchor (PrayTimes `angleBased`).
        HighLatitudeRule::AngleBased => {
            if is_fajr {
                mid_day - angle / 15.0
            } else {
                sunset + angle / 15.0
            }
        }
    }
}

impl PrayerTimesBuilder {
    /// Compute the five prayer times + sunrise for the given local date,
    /// latitude, longitude and UTC offset (in hours).
    pub fn build(self, date: NaiveDate, lat: f64, lon: f64, tz_offset_hours: f64) -> PrayerTimes {
        let (method, asr_school, high_lat_rule) =
            (self.method, self.asr_school, self.high_lat_rule);

        let (fajr_angle, isha_angle) = method.angles();

        let jd =
            julian_date(date.year(), date.month() as i32, date.day() as i32) - lon / (15.0 * 24.0);

        let (_, eqt) = sun_position(jd);
        let mid_day = fix_hour(12.0 - eqt - lon / 15.0);

        let (decl_pre, _) = sun_position(jd + -0.25);
        let (decl_mid, _) = sun_position(jd + 0.0);
        let (decl_post, _) = sun_position(jd + 0.25);

        let sunrise = mid_day - compute_time(lat, decl_pre, -0.833);
        let sunset = mid_day + compute_time(lat, decl_post, -0.833);
        let fajr_raw = mid_day - compute_time(lat, decl_pre, -fajr_angle);
        let isha_raw = mid_day + compute_time(lat, decl_post, -isha_angle);

        let asr_alt = asr_altitude(lat, decl_mid, asr_school);
        let asr = mid_day + compute_time(lat, decl_mid, asr_alt);

        // High-latitude adjustment, applied independently to Fajr and Isha
        // (following PrayTimes). A single "night duration" anchors both; it is
        // robust when the sun never reaches the Fajr/Isha altitude that day.
        let night = if fajr_raw.is_finite() {
            fajr_raw + 24.0 - sunset
        } else if isha_raw.is_finite() {
            isha_raw - sunset
        } else {
            12.0 // fallback: half-day night
        };
        let (fajr, isha) = (
            adjust_high_lat(
                fajr_raw,
                &high_lat_rule,
                fajr_angle,
                sunset,
                mid_day,
                night,
                true,
            ),
            adjust_high_lat(
                isha_raw,
                &high_lat_rule,
                isha_angle,
                sunset,
                mid_day,
                night,
                false,
            ),
        );

        // Fix timezone offset (result was computed in UT-based solar terms).
        // NOTE: we deliberately do NOT wrap into [0,24h) here. Keeping the raw
        // minutes lets `seconds_until_next` handle day rollover correctly when a
        // prayer falls after local midnight (e.g. Isha late in high latitudes).
        // `format_clock` normalises for display.
        let tz = tz_offset_hours;
        let mut times = DayTimes {
            fajr: (fajr + tz) * 60.0,
            sunrise: (sunrise + tz) * 60.0,
            dhuhr: (mid_day + tz) * 60.0,
            asr: (asr + tz) * 60.0,
            maghrib: (sunset + tz) * 60.0,
            isha: (isha + tz) * 60.0,
        };

        // Umm Al-Qura sentinel: Isha = Maghrib + 90 minutes.
        if (isha_angle - 90.0).abs() < f64::EPSILON {
            times.isha = fix_hour(sunset + tz + 90.0 / 60.0) * 60.0;
        }

        PrayerTimes {
            times,
            year: date.year(),
            month: date.month(),
            day: date.day(),
            lat,
            lon,
            tz_offset: tz,
            method,
            asr_school,
            high_lat_rule,
        }
    }
}

/// Compute the countdown, in whole seconds, from `now` (a local clock time)
/// until the next prayer boundary.
///
/// Returns the name of the next prayer and the seconds remaining. This is a
/// convenience helper used by the widget layer; the actual interpretation of
/// "now" and day rollover is the caller's responsibility.
pub fn seconds_until_next(
    times: &DayTimes,
    // current time as minutes since local midnight
    now_minutes: f64,
) -> Option<(PrayerName, u64)> {
    let mut candidates: Vec<(PrayerName, f64)> = vec![
        (PrayerName::Fajr, times.fajr),
        (PrayerName::Sunrise, times.sunrise),
        (PrayerName::Dhuhr, times.dhuhr),
        (PrayerName::Asr, times.asr),
        (PrayerName::Maghrib, times.maghrib),
        (PrayerName::Isha, times.isha),
    ];
    candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    for (name, t) in candidates {
        if t > now_minutes {
            let secs = ((t - now_minutes) * 60.0).round() as u64;
            return Some((name, secs));
        }
    }
    None // all past -> tomorrow's Fajr (caller handles wrapping)
}

/// The name of a prayer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrayerName {
    Fajr,
    Sunrise,
    Dhuhr,
    Asr,
    Maghrib,
    Isha,
}

impl PrayerName {
    pub fn as_str(&self) -> &'static str {
        match self {
            PrayerName::Fajr => "Fajr",
            PrayerName::Sunrise => "Sunrise",
            PrayerName::Dhuhr => "Dhuhr",
            PrayerName::Asr => "Asr",
            PrayerName::Maghrib => "Maghrib",
            PrayerName::Isha => "Isha",
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64, eps: f64) {
        assert!((a - b).abs() < eps, "{a} !≈ {b} (±{eps})");
    }

    #[test]
    fn tokyo_sunrise_known_sanity() {
        // Rough sanity: anywhere, sunrise should be within 0..24h and dhuhr
        // should be > sunrise.
        let d = NaiveDate::from_ymd_opt(2026, 6, 21).unwrap();
        let times = PrayerTimesBuilder::default().build(d, 35.68, 139.69, 9.0);
        let t = times.times;
        assert!(t.sunrise >= 0.0 && t.sunrise < 24.0 * 60.0);
        assert!(t.dhuhr > t.sunrise);
        assert!(t.isha > t.maghrib);
    }

    #[test]
    fn format_24h_rounds_to_nearest() {
        assert_eq!(format_clock(90.0, false), "01:30");
        assert_eq!(format_clock(12.2, false), "00:12");
        assert_eq!(format_clock(0.0, false), "00:00");
    }

    #[test]
    fn format_12h_correct() {
        assert_eq!(format_clock(0.0, true), "12:00 AM");
        assert_eq!(format_clock(6.0 * 60.0, true), "6:00 AM");
        assert_eq!(format_clock(17.0 * 60.0, true), "5:00 PM");
        assert_eq!(format_clock(12.0 * 60.0, true), "12:00 PM");
    }

    #[test]
    fn seconds_until_basic() {
        let times = DayTimes {
            fajr: 4.0 * 60.0,
            sunrise: 5.5 * 60.0,
            dhuhr: 12.0 * 60.0,
            asr: 15.5 * 60.0,
            maghrib: 18.0 * 60.0,
            isha: 19.5 * 60.0,
        };
        let (name, secs) = seconds_until_next(&times, 11.0 * 60.0 + 30.0).unwrap();
        assert_eq!(name, PrayerName::Dhuhr);
        // 12:00 - 11:30 = 0h30 = 1800s
        approx(secs as f64, 1800.0, 60.0);
    }

    #[test]
    fn ordering_valid_standard_cases() {
        // For latitudes/dates where Fajr & Isha exist astronomically, the
        // ordering fajr < sunrise < dhuhr < asr < maghrib < isha must hold.
        // (Extreme summer high-latitude cases apply the high-latitude rule and
        //  deliberately blur this ordering — those are tested separately.)
        let cases = [
            // (date, lat, lon, tz)
            (2026, 1, 10, 48.85, 2.35, 1.0),     // Paris, winter
            (2026, 3, 15, 21.03, 105.85, 7.0),   // Hanoi
            (2026, 12, 21, 60.17, 24.94, 2.0),   // Helsinki, winter
            (2026, 6, 21, -33.87, 151.21, 10.0), // Sydney
            (2026, 6, 21, 32.22, 35.25, 3.0),    // Jerusalem
        ];
        for (y, mo, d, lat, lon, tz) in cases {
            let date = NaiveDate::from_ymd_opt(y, mo, d).unwrap();
            let t = PrayerTimesBuilder::default()
                .build(date, lat, lon, tz)
                .times;
            let v = [t.fajr, t.sunrise, t.dhuhr, t.asr, t.maghrib, t.isha];
            for w in v.windows(2) {
                assert!(w[1] > w[0], "at ({lat},{lon}) on {y}-{mo}-{d}: {:?}", v);
            }
        }
    }

    #[test]
    fn high_latitude_never_nan() {
        // Extreme summer at high latitude: the sun never reaches Fajr/Isha
        // altitude. The high-latitude rule must kick in and produce finite
        // values (not NaN).
        let date = NaiveDate::from_ymd_opt(2026, 6, 21).unwrap();
        let t = PrayerTimesBuilder::default()
            .build(date, 60.17, 24.94, 3.0)
            .times;
        let v = [t.fajr, t.sunrise, t.dhuhr, t.asr, t.maghrib, t.isha];
        for x in v {
            assert!(x.is_finite(), "non-finite value: {v:?}");
        }
        // Isha must come after Maghrib
        assert!(t.isha > t.maghrib, "maghrib={} isha={}", t.maghrib, t.isha);
    }

    #[test]
    fn calibration_paris_against_aladhan() {
        // Reference values from the AlAdhan API for Paris (48.8534, 2.3488)
        // on 2026-01-10, method 12 (UOIF, Fajr=12°, Isha=12°), UTC+1 (winter).
        // Fajr 07:26, Sunrise 08:42, Dhuhr 12:58, Asr 14:55, Maghrib 17:15, Isha 18:31
        let date = NaiveDate::from_ymd_opt(2026, 1, 10).unwrap();
        let b = PrayerTimesBuilder {
            method: CalculationMethod::UnionOrganization,
            high_lat_rule: HighLatitudeRule::AngleBased,
            ..Default::default()
        };
        let t = b.build(date, 48.8534, 2.3488, 1.0).times;

        // Tolerance of a few minutes is expected (rounding & sub-minute solar
        // position approximations differ slightly between implementations).
        let cases = [
            (t.fajr, "07:26", 3.0),
            (t.sunrise, "08:42", 3.0),
            (t.dhuhr, "12:58", 3.0),
            (t.asr, "14:55", 3.0),
            (t.maghrib, "17:15", 3.0),
            (t.isha, "18:31", 3.0),
        ];
        for (got, expected, tol) in cases {
            let expected_min = clock_to_min(expected);
            let got_min = got;
            assert!(
                (got_min - expected_min).abs() <= tol,
                "{expected}: expected {expected}, got {}",
                format_clock(got_min, false)
            );
        }
    }

    #[test]
    fn calibration_sydney_against_aladhan() {
        // Sydney (-33.8688, 151.2093) on 2026-03-15, method 3 (MWL, Fajr=18°,
        // Isha=17°), UTC+11 (AEDT). AlAdhan: Fajr 05:30, Sunrise 06:54,
        // Dhuhr 13:04, Asr 16:35, Maghrib 19:14, Isha 20:33.
        let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let b = PrayerTimesBuilder {
            method: CalculationMethod::MuslimWorldLeague,
            high_lat_rule: HighLatitudeRule::AngleBased,
            ..Default::default()
        };
        let t = b.build(date, -33.8688, 151.2093, 11.0).times;
        let cases = [
            (t.fajr, "05:30", 3.0),
            (t.sunrise, "06:54", 3.0),
            (t.dhuhr, "13:04", 3.0),
            (t.asr, "16:35", 3.0),
            (t.maghrib, "19:14", 3.0),
            (t.isha, "20:33", 3.0),
        ];
        for (got, expected, tol) in cases {
            let expected_min = clock_to_min(expected);
            assert!(
                (got - expected_min).abs() <= tol,
                "{expected}: expected {expected}, got {}",
                format_clock(got, false)
            );
        }
    }

    fn clock_to_min(s: &str) -> f64 {
        let h: f64 = s[0..2].parse().unwrap();
        let m: f64 = s[3..5].parse().unwrap();
        h * 60.0 + m
    }
}
