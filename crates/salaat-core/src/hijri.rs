//! Conversion from a Gregorian date to the Islamic (Hijri) calendar date.
//!
//! Ported from the arithmetic tabular-civil algorithm used by the KDE widget
//! (`getHijriDate` in `OfflinePrayerCalc.js`). This is the *civil* (tabular,
//! arithmetic) approximation — results may differ by a day from actual lunar
//! sighting depending on the region and convention.

use chrono::{Datelike, NaiveDate};

/// The conventional names of the twelve Hijri months (English), index 0-based.
pub const MONTHS_EN: [&str; 12] = [
    "Muharram",
    "Safar",
    "Rabi al-Awwal",
    "Rabi al-Thani",
    "Jumada al-Awwal",
    "Jumada al-Thani",
    "Rajab",
    "Sha'ban",
    "Ramadan",
    "Shawwal",
    "Dhu al-Qi'dah",
    "Dhu al-Hijjah",
];

/// The conventional names of the twelve Hijri months (French).
pub const MONTHS_FR: [&str; 12] = [
    "Muharram",
    "Safar",
    "Rabi al-awwal",
    "Rabi al-thani",
    "Joumada al-oula",
    "Joumada al-thania",
    "Rajab",
    "Chaabane",
    "Ramadan",
    "Chawwal",
    "Dhou al-qi'da",
    "Dhou al-hijja",
];

/// The conventional names of the twelve Hijri months (Arabic).
pub const MONTHS_AR: [&str; 12] = [
    "محرم",
    "صفر",
    "ربيع الأول",
    "ربيع الآخر",
    "جمادى الأولى",
    "جمادى الآخرة",
    "رجب",
    "شعبان",
    "رمضان",
    "شوال",
    "ذو القعدة",
    "ذو الحجة",
];

/// A date in the Hijri (Islamic) calendar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HijriDate {
    pub day: u32,
    pub month: u32, // 1..=12
    pub year: i32,
}

impl HijriDate {
    /// Format as `"DD MonthName YYYY"` in the given month-name set.
    pub fn format(&self, months: &[&str; 12]) -> String {
        format!(
            "{} {} {}",
            self.day,
            months.get(self.month as usize - 1).copied().unwrap_or(""),
            self.year
        )
    }
}

/// Convert a Gregorian date to a tabular-civil Hijri date, with an optional
/// day adjustment (typically `+1` to match observed calendars).
pub fn gregorian_to_hijri(date: NaiveDate, adjustment: i32) -> HijriDate {
    let day_f = date.day() as f64;
    let mut month_f = date.month() as f64;
    let mut year_f = date.year() as f64;

    if month_f < 3.0 {
        year_f -= 1.0;
        month_f += 12.0;
    }

    let a = (year_f / 100.0).floor();
    let mut b = 2.0 - a + (a / 4.0).floor();
    if year_f < 1583.0 {
        b = 0.0;
    }
    if year_f.floor() == 1582.0 {
        if month_f > 10.0 {
            b = -10.0;
        }
        if month_f == 10.0 {
            b = 0.0;
            if day_f > 4.0 {
                b = -10.0;
            }
        }
    }

    let jd = (365.25 * (year_f + 4716.0)).floor() + (30.6001 * (month_f + 1.0)).floor() + day_f + b
        - 1524.0;
    let jd = jd + adjustment as f64 - 1.0;

    // Tabular (civil) conversion formula.
    let iyear: f64 = 10631.0 / 30.0;
    let epochastro: f64 = 1948084.0;
    let shift1 = 8.01 / 60.0;
    let mut z = jd - epochastro;
    let cyc = (z / 10631.0).floor();
    z = z - 10631.0 * cyc;
    let j = ((z - shift1) / iyear).floor();
    let iy = 30.0 * cyc + j;
    z = z - (j * iyear + shift1).floor();
    let mut im = ((z + 28.5001) / 29.5).floor();
    if im == 13.0 {
        im = 12.0;
    }
    let id = z - (29.5001 * im - 29.0).floor();

    HijriDate {
        day: id as u32,
        month: im as u32,
        year: iy as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_conversion() {
        // 11 March 2024 is commonly observed as 1 Ramadan 1445.
        let d = NaiveDate::from_ymd_opt(2024, 3, 11).unwrap();
        let h = gregorian_to_hijri(d, 1);
        assert_eq!(h.year, 1445);
        assert_eq!(h.month, 9);
    }

    #[test]
    fn format_works() {
        let h = HijriDate {
            day: 1,
            month: 9,
            year: 1445,
        };
        assert_eq!(h.format(&MONTHS_EN), "1 Ramadan 1445");
        assert_eq!(h.format(&MONTHS_FR), "1 Ramadan 1445");
    }
}
