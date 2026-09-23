//! # salaat-core
//!
//! Pure, offline, platform-agnostic calculation of Islamic prayer times and the
//! Hijri date. No networking, no UI, no OS dependencies — only the Gregorian
//! date and geographic coordinates are needed.
//!
//! The astronomical algorithms are ported from the reference implementation
//! used by the KDE Plasma widget (`salaatprayertime`), itself based on the
//! AlAdhan / PrayTimes astronomical method set.

pub mod hijri;
pub mod prayer_times;

pub use prayer_times::{CalculationMethod, PrayerTimes, PrayerTimesBuilder};
