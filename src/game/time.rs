use rand::prelude::*;
use rand_distr::StandardNormal;
use std::cell::RefCell;
use std::fmt::{self, Formatter};
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

pub const CARDINAL_MOVE: Time = Time { t: 8 * SECS_TO_TIME };
pub const DIAGNOL_MOVE: Time = Time {
    t: 11 * SECS_TO_TIME + 314 * MS_TO_TIME,
};
pub const DESTROY_EMP_SWORD: Time = Time { t: 24 * SECS_TO_TIME };
pub const FLOOD: Time = Time { t: 32 * SECS_TO_TIME };
pub const MOVE_THRU_SHALLOW_WATER: Time = Time { t: 2 * SECS_TO_TIME };
pub const OPEN_DOOR: Time = Time { t: 10 * SECS_TO_TIME };
pub const PICK_UP: Time = Time { t: 4 * SECS_TO_TIME };
pub const SHOVE_DOORMAN: Time = Time { t: 16 * SECS_TO_TIME };
pub const SPEAK_TO_SPECTATOR: Time = Time { t: 2 * SECS_TO_TIME };

pub const MIN_TIME: Time = Time { t: 1 * SECS_TO_TIME };

#[derive(Copy, Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Time {
    t: i64,
}

/// Minimum time for actions is 1s although the times at which objects are scheduled is
/// fuzzed so that an object scheduled for 1s from now will actually execute in 1s +/- a
/// small delta.
impl Time {
    pub fn zero() -> Time {
        Time { t: 0 }
    }

    pub fn max() -> Time {
        Time { t: i64::MAX }
    }

    /// Used by the scheduler.
    pub fn fuzz(&self, rng: &RefCell<dyn RngCore>) -> Time {
        let taken = rand_normal64(self.t, 20, rng);
        let taken = i64::max(taken, 1); // time has to advance
        Time { t: taken }
    }

    pub fn as_ms(&self) -> i64 {
        self.t
    }
}

/// In general this only should be used for "extra" time. For the most part use the constants
/// above (e.g. CARDINAL_MOVE).
pub fn secs(s: i64) -> Time {
    Time { t: s * SECS_TO_TIME }
}

// /// In general this only should be used for "extra" time. For the most part use the constants
// /// above (e.g. CARDINAL_MOVE).
// pub fn ms(s: i64) -> Time {
//     Time { t: s * MS_TO_TIME }
// }

// ---- Time traits ----------------------------------------------------------------------
const MS_TO_TIME: i64 = 1;
const SECS_TO_TIME: i64 = 1000 * MS_TO_TIME;

impl fmt::Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if i64::abs(self.t) < SECS_TO_TIME {
            write!(f, "{} ms", self.t)
        } else if i64::abs(self.t) < 60 * SECS_TO_TIME {
            let x = (self.t as f64) / (SECS_TO_TIME as f64);
            write!(f, "{x:.1} secs")
        } else if i64::abs(self.t) < 60 * 60 * SECS_TO_TIME {
            let x = (self.t as f64) / (SECS_TO_TIME as f64);
            let x = x / 60.0;
            write!(f, "{x:.1} mins")
        } else {
            let x = (self.t as f64) / (SECS_TO_TIME as f64);
            let x = x / (60.0 * 60.0);
            write!(f, "{x:.1} hours")
        }
    }
}

impl Add for Time {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Time { t: self.t + rhs.t }
    }
}

impl Sub for Time {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Time { t: self.t - rhs.t }
    }
}

impl AddAssign for Time {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self { t: self.t + rhs.t };
    }
}

impl SubAssign for Time {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Self { t: self.t - rhs.t };
    }
}

impl Div<i64> for Time {
    type Output = Self;

    fn div(self, rhs: i64) -> Self::Output {
        Time { t: self.t / rhs }
    }
}

impl Mul<i64> for Time {
    type Output = Self;

    fn mul(self, rhs: i64) -> Self::Output {
        Time { t: self.t * rhs }
    }
}

/// Returns a number with the standard normal distribution centered on x where the
/// values are all within +/- the given percentage.
fn rand_normal64(x: i64, percent: i32, rng: &RefCell<dyn RngCore>) -> i64 {
    assert!(percent > 0);
    assert!(percent <= 100);

    // Could use a generic for this but the type bounds get pretty gnarly.
    let rng = &mut *rng.borrow_mut();
    let scaling: f64 = rng.sample(StandardNormal); // ~95% are in -2..2
    let scaling = if scaling >= -2.0 && scaling <= 2.0 {
        scaling / 2.0 // all are in -1..1
    } else {
        0.0 // the few outliers are mapped to the mode
    };
    let scaling = scaling * (percent as f64) / 100.0; // all are in +/- percent
    let delta = (x as f64) * scaling; // all are in +/- percent of x
    x + (delta as i64)
}
