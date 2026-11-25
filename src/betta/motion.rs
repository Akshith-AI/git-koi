use crate::leetcode::{Availability, LeetCodeSnapshot};
use chrono::Days;
use std::f64::consts::TAU;

/// Four seconds at the common 15 FPS export rate.
pub const BETTA_FRAMES: u32 = 60;
/// Five geometrically distinct fin/gill states for interactive transitions.
pub const FLARE_LEVELS: u32 = 5;

/// Activity influences energy without interpreting missing observations as inactivity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BettaActivity {
    /// Recent accepted submissions or calendar activity were observed.
    Active,
    /// Available calendar observations confirm no activity in a complete seven-day window.
    Quiet,
    /// A complete recent window is unavailable; use neutral swimming.
    Unknown,
}

/// Choose a movement signal as of the snapshot date, never from the current clock.
pub fn betta_activity(snapshot: &LeetCodeSnapshot) -> BettaActivity {
    let Some(start) = snapshot.observed_on.checked_sub_days(Days::new(6)) else {
        return BettaActivity::Unknown;
    };
    if let Availability::Available { data } = &snapshot.recent_accepted {
        if data.iter().any(|entry| {
            chrono::DateTime::from_timestamp(entry.timestamp, 0)
                .is_some_and(|stamp| (start..=snapshot.observed_on).contains(&stamp.date_naive()))
        }) {
            return BettaActivity::Active;
        }
    }
    if let Availability::Available { data } = &snapshot.calendar {
        use chrono::Datelike;
        if data.year == snapshot.observed_on.year() && start.year() == data.year {
            return if data
                .submissions
                .range(start..=snapshot.observed_on)
                .any(|(_, &count)| count > 0)
            {
                BettaActivity::Active
            } else {
                BettaActivity::Quiet
            };
        }
    }
    BettaActivity::Unknown
}

/// A periodic pose. Frame indices wrap and the spread is clamped to zero through one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BettaPose {
    /// Fin wave phase in radians.
    pub phase: f64,
    /// Relative activity amplitude.
    pub energy: f64,
    /// Fin and operculum opening, zero relaxed and one fully flared.
    pub flare: f64,
    /// Gentle horizontal drift in native pixels.
    pub dx: f64,
    /// Gentle vertical drift in native pixels.
    pub dy: f64,
}

/// Derive a loop-closing pose without random state or wall-clock reads.
pub fn betta_pose(frame: u32, activity: BettaActivity, flare: f64) -> BettaPose {
    let phase = f64::from(frame % BETTA_FRAMES) / f64::from(BETTA_FRAMES) * TAU;
    let energy = match activity {
        BettaActivity::Active => 1.,
        BettaActivity::Quiet => 0.4,
        BettaActivity::Unknown => 0.7,
    };
    BettaPose {
        phase,
        energy,
        flare: if flare.is_finite() {
            flare.clamp(0., 1.)
        } else {
            0.
        },
        dx: 3. * phase.sin() * energy,
        dy: 2. * (phase * 2.).sin() * energy,
    }
}
