//! Prayer notifications.
//!
//! A countdown you have to watch yourself is not a reminder, so the widget
//! raises two desktop notifications per prayer: one a few minutes ahead (so you
//! can wrap up what you are doing) and one when the time comes in.
//!
//! This runs in the backend rather than the frontend on purpose: the widget can
//! be hidden in the tray — or never shown at all with `start_hidden` — and the
//! reminders must still fire.

use std::time::Duration;

use chrono::Local;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::AppState;

/// How often the scheduler re-evaluates the next prayer.
const POLL: Duration = Duration::from_secs(15);

/// A prayer must have been observed this close to its time for the "it is now"
/// notification to fire on the transition. It keeps a stale reminder from
/// firing hours late after the machine wakes from sleep, and keeps a mere
/// settings change (a new city shifts the next prayer) from looking like one.
const TRANSITION_WINDOW_SECS: u64 = 120;

/// Sunrise ends Fajr, it is not a prayer — never announce it.
const SKIPPED: &str = "Sunrise";

/// Localized prayer name for a backend identifier.
fn prayer_label(name: &str, language: &str) -> String {
    let label = match (language, name) {
        ("ar", "Fajr") => "الفجر",
        ("ar", "Dhuhr") => "الظهر",
        ("ar", "Asr") => "العصر",
        ("ar", "Maghrib") => "المغرب",
        ("ar", "Isha") => "العشاء",
        _ => name,
    };
    label.to_string()
}

/// Body text for the "the time has come in" notification.
fn body_at_time(label: &str, language: &str) -> String {
    match language {
        "ar" => format!("حان وقت {label}"),
        "en" => format!("It is time for {label}"),
        _ => format!("C'est l'heure de {label}"),
    }
}

/// Body text for the early reminder.
fn body_before(label: &str, minutes: u8, language: &str) -> String {
    // The reminder trips only within the lead time, but a launch (or a waking
    // machine) that lands in the last half-minute would round to zero: nobody
    // says "in 0 minutes", the prayer is due now.
    if minutes == 0 {
        return match language {
            "ar" => format!("{label} الآن"),
            "en" => format!("{label} now"),
            _ => format!("{label} maintenant"),
        };
    }
    match (language, minutes) {
        ("ar", _) => format!("{label} بعد {minutes} دقيقة"),
        ("en", 1) => format!("{label} in 1 minute"),
        ("en", _) => format!("{label} in {minutes} minutes"),
        (_, 1) => format!("{label} dans 1 minute"),
        (_, _) => format!("{label} dans {minutes} minutes"),
    }
}

fn notify(app: &AppHandle, title: &str, body: &str) {
    let _ = app.notification().builder().title(title).body(body).show();
}

/// What one scheduler tick decided to raise. Holds the prayer identifiers; the
/// caller turns them into localized text.
#[derive(Debug, Default, PartialEq, Eq)]
struct Decision {
    /// A prayer whose time has just come in.
    at_time: Option<String>,
    /// A prayer the early reminder is now due for, with how many minutes it
    /// announces.
    before: Option<(String, u8)>,
}

/// The notification state machine, kept separate from the thread so every
/// branch is unit-testable — the transition heuristic is where the false
/// positives hide (a machine waking from sleep, a change of city).
#[derive(Default)]
struct Scheduler {
    /// Last observed (prayer, seconds remaining). A change of prayer name means
    /// the previous one just came in.
    last: Option<(String, u64)>,
    /// Prayer the early reminder already fired for; a different next prayer
    /// re-arms it.
    warned_for: Option<String>,
}

impl Scheduler {
    /// Forget what we were watching. Used when notifications are switched off,
    /// so re-enabling them cannot announce a transition nobody observed.
    fn reset(&mut self) {
        self.last = None;
    }

    /// Advance the state machine with the currently-next prayer and how long is
    /// left before it.
    fn tick(&mut self, cfg: &crate::config::PrayerConfig, name: &str, remaining: u64) -> Decision {
        let mut decision = Decision::default();

        // "It is now": the next prayer changed, and we were watching the
        // previous one right up to its time. Without that second condition a
        // machine resuming from sleep — or simply a new city shifting the next
        // prayer — would look exactly like a prayer coming in.
        if cfg.notify_at_time {
            if let Some((previous, previous_remaining)) = &self.last {
                if previous != name
                    && *previous_remaining <= TRANSITION_WINDOW_SECS
                    && previous != SKIPPED
                {
                    decision.at_time = Some(previous.clone());
                }
            }
        }

        // Early reminder, once per prayer occurrence.
        if cfg.notify_before && name != SKIPPED {
            let lead = u64::from(cfg.notify_before_minutes) * 60;
            if remaining <= lead && self.warned_for.as_deref() != Some(name) {
                // Announce the time actually left, not the configured lead.
                // They match within a poll in steady state, but launching the
                // app — or waking it — three minutes before a prayer must not
                // claim ten.
                let minutes = ((remaining + 30) / 60) as u8;
                decision.before = Some((name.to_string(), minutes));
                self.warned_for = Some(name.to_string());
            }
        }

        self.last = Some((name.to_string(), remaining));
        decision
    }
}

/// Spawn the notification scheduler.
pub fn spawn(app: AppHandle) {
    std::thread::spawn(move || {
        let mut scheduler = Scheduler::default();

        loop {
            std::thread::sleep(POLL);

            let cfg = {
                let state = app.state::<AppState>();
                let Ok(cfg) = state.cfg.lock() else { continue };
                cfg.clone()
            };
            if !cfg.notify_at_time && !cfg.notify_before {
                scheduler.reset();
                continue;
            }

            let Ok(status) = crate::commands::compute_status_payload(&cfg, Local::now()) else {
                continue; // no location configured yet
            };

            let decision = scheduler.tick(&cfg, &status.next_name, status.remaining_seconds);

            if let Some(prayer) = decision.at_time {
                let label = prayer_label(&prayer, &cfg.language);
                notify(&app, &label, &body_at_time(&label, &cfg.language));
            }
            if let Some((prayer, minutes)) = decision.before {
                let label = prayer_label(&prayer, &cfg.language);
                let body = body_before(&label, minutes, &cfg.language);
                notify(&app, &label, &body);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_fall_back_to_the_backend_identifier() {
        assert_eq!(prayer_label("Dhuhr", "fr"), "Dhuhr");
        assert_eq!(prayer_label("Dhuhr", "en"), "Dhuhr");
        assert_eq!(prayer_label("Dhuhr", "ar"), "الظهر");
        // Sunrise has no Arabic entry here: it is never announced.
        assert_eq!(prayer_label("Sunrise", "ar"), "Sunrise");
    }

    #[test]
    fn bodies_are_localized() {
        assert_eq!(body_at_time("Asr", "fr"), "C'est l'heure de Asr");
        assert_eq!(body_at_time("Asr", "en"), "It is time for Asr");
        assert_eq!(body_before("Asr", 10, "en"), "Asr in 10 minutes");
        assert_eq!(body_before("Asr", 5, "fr"), "Asr dans 5 minutes");
        // Under half a minute the reminder rounds to zero, which must read as
        // "now" rather than "in 0 minutes".
        assert_eq!(body_before("Asr", 0, "en"), "Asr now");
        assert_eq!(body_before("Asr", 0, "fr"), "Asr maintenant");
        assert_eq!(body_before("العصر", 0, "ar"), "العصر الآن");
        // Singular: "1 minute", not "1 minutes".
        assert_eq!(body_before("Isha", 1, "en"), "Isha in 1 minute");
        assert_eq!(body_before("Isha", 1, "fr"), "Isha dans 1 minute");
    }

    /// Both reminders on, 10-minute lead.
    fn cfg() -> crate::config::PrayerConfig {
        crate::config::PrayerConfig::default()
    }

    fn at_time(d: &Decision) -> Option<&str> {
        d.at_time.as_deref()
    }

    fn before(d: &Decision) -> Option<&str> {
        d.before.as_ref().map(|(name, _)| name.as_str())
    }

    #[test]
    fn nothing_fires_on_the_very_first_tick() {
        let mut s = Scheduler::default();
        // Startup mid-afternoon: there is no previous observation to compare to,
        // so no prayer can have "just" come in.
        let d = s.tick(&cfg(), "Asr", 3600);
        assert_eq!(at_time(&d), None);
        assert_eq!(before(&d), None);
    }

    #[test]
    fn prayer_coming_in_is_announced() {
        let mut s = Scheduler::default();
        s.tick(&cfg(), "Dhuhr", 30); // watched it right up to its time
        let d = s.tick(&cfg(), "Asr", 9000);
        assert_eq!(at_time(&d), Some("Dhuhr"));
    }

    #[test]
    fn waking_from_sleep_does_not_announce_a_stale_prayer() {
        let mut s = Scheduler::default();
        // Machine suspended an hour before Dhuhr and woke up after Asr.
        s.tick(&cfg(), "Dhuhr", 3600);
        let d = s.tick(&cfg(), "Maghrib", 5400);
        assert_eq!(at_time(&d), None, "Dhuhr passed unobserved, hours ago");
    }

    #[test]
    fn changing_city_does_not_announce_a_prayer() {
        let mut s = Scheduler::default();
        // Asr was 40 min away; a new city makes Maghrib the next prayer.
        s.tick(&cfg(), "Asr", 2400);
        let d = s.tick(&cfg(), "Maghrib", 1800);
        assert_eq!(at_time(&d), None);
    }

    #[test]
    fn sunrise_is_never_announced() {
        let mut s = Scheduler::default();
        // Sunrise ends Fajr; it must not produce either notification.
        let d = s.tick(&cfg(), "Sunrise", 60);
        assert_eq!(before(&d), None, "no early reminder for sunrise");
        let d = s.tick(&cfg(), "Dhuhr", 9000);
        assert_eq!(at_time(&d), None, "no 'it is now' when sunrise passes");
    }

    #[test]
    fn early_reminder_fires_once_per_prayer() {
        let mut s = Scheduler::default();
        let d = s.tick(&cfg(), "Asr", 600); // exactly on the 10-minute lead
        assert_eq!(before(&d), Some("Asr"));
        for remaining in [585, 540, 120, 15] {
            let d = s.tick(&cfg(), "Asr", remaining);
            assert_eq!(before(&d), None, "already warned at {remaining}s");
        }
    }

    #[test]
    fn the_reminder_announces_the_time_actually_left() {
        let mut s = Scheduler::default();
        // Launched three minutes before Isha, with the default 10-minute lead:
        // saying "in 10 minutes" would be plainly wrong.
        let d = s.tick(&cfg(), "Isha", 190);
        assert_eq!(d.before, Some(("Isha".to_string(), 3)));

        // Steady state: the reminder trips just under the lead and rounds back
        // to it.
        let mut s = Scheduler::default();
        let d = s.tick(&cfg(), "Asr", 598);
        assert_eq!(d.before, Some(("Asr".to_string(), 10)));
    }

    #[test]
    fn early_reminder_re_arms_for_the_next_prayer() {
        let mut s = Scheduler::default();
        assert_eq!(before(&s.tick(&cfg(), "Asr", 600)), Some("Asr"));
        assert_eq!(before(&s.tick(&cfg(), "Maghrib", 600)), Some("Maghrib"));
    }

    #[test]
    fn early_reminder_waits_for_the_lead_time() {
        let mut s = Scheduler::default();
        assert_eq!(before(&s.tick(&cfg(), "Isha", 601)), None);
        assert_eq!(before(&s.tick(&cfg(), "Isha", 600)), Some("Isha"));
    }

    #[test]
    fn lead_time_is_configurable() {
        let mut cfg = cfg();
        cfg.notify_before_minutes = 30;
        let mut s = Scheduler::default();
        assert_eq!(before(&s.tick(&cfg, "Fajr", 1800)), Some("Fajr"));
    }

    #[test]
    fn each_reminder_can_be_switched_off_independently() {
        let mut cfg = cfg();
        cfg.notify_before = false;
        let mut s = Scheduler::default();
        s.tick(&cfg, "Dhuhr", 30);
        let d = s.tick(&cfg, "Asr", 600);
        assert_eq!(before(&d), None, "early reminder disabled");
        assert_eq!(at_time(&d), Some("Dhuhr"), "but the other one still fires");

        cfg.notify_before = true;
        cfg.notify_at_time = false;
        let mut s = Scheduler::default();
        s.tick(&cfg, "Dhuhr", 30);
        let d = s.tick(&cfg, "Asr", 600);
        assert_eq!(at_time(&d), None, "'it is now' disabled");
        assert_eq!(before(&d), Some("Asr"));
    }

    #[test]
    fn reset_forgets_the_pending_transition() {
        let mut s = Scheduler::default();
        s.tick(&cfg(), "Dhuhr", 30);
        // Notifications switched off, then back on: nobody watched Dhuhr come
        // in, so re-enabling them must not announce it retroactively.
        s.reset();
        let d = s.tick(&cfg(), "Asr", 9000);
        assert_eq!(at_time(&d), None);
    }

    #[test]
    fn a_full_cycle_fires_each_prayer_exactly_once() {
        let mut s = Scheduler::default();
        let mut at = Vec::new();
        let mut early = Vec::new();
        // Walk Fajr -> Isha, each prayer approached from 15 min out down to 0.
        for prayer in ["Fajr", "Dhuhr", "Asr", "Maghrib", "Isha"] {
            for remaining in [900, 600, 300, 60, 0] {
                let d = s.tick(&cfg(), prayer, remaining);
                if let Some(p) = d.at_time {
                    at.push(p);
                }
                if let Some((p, _)) = d.before {
                    early.push(p);
                }
            }
        }
        assert_eq!(early, ["Fajr", "Dhuhr", "Asr", "Maghrib", "Isha"]);
        // The last prayer of the walk has not been superseded yet.
        assert_eq!(at, ["Fajr", "Dhuhr", "Asr", "Maghrib"]);
    }
}
