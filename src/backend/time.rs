use rand::rngs::SmallRng;
use std::cell::RefCell;
use std::fmt::{self, Formatter};
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

pub const BASE_ATTACK: Time = Time { t: 1 * SECS_TO_TIME };
pub const CARDINAL_MOVE: Time = Time { t: 8 * SECS_TO_TIME };
pub const DIAGNOL_MOVE: Time = Time {
    t: 11 * SECS_TO_TIME + 3 * SEC_TENTHS_TO_TIME,
};
pub const DESTROY_EMP_SWORD: Time = Time { t: 24 * SECS_TO_TIME };
pub const DIG_STONE: Time = Time { t: 32 * SECS_TO_TIME };
pub const FLOOD: Time = Time { t: 32 * SECS_TO_TIME };
pub const MOVE_THRU_SHALLOW_WATER: Time = Time { t: 2 * SECS_TO_TIME };
pub const OPEN_DOOR: Time = Time { t: 10 * SECS_TO_TIME };
pub const PICK_UP: Time = Time { t: 4 * SECS_TO_TIME };
pub const SCRATCH_METAL: Time = Time { t: 3 * SECS_TO_TIME };
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
    pub fn fuzz(&self, rng: &RefCell<SmallRng>) -> Time {
        let taken = super::rand_normal64(self.t, 20, rng);
        let taken = i64::max(taken, 1); // time has to advance
        Time { t: taken }
    }
}

/// In general this only should be used for "extra" time. For the most part use the constants
/// above (e.g. CARDINAL_MOVE).
pub fn secs(s: i64) -> Time {
    Time { t: s * SECS_TO_TIME }
}

// ---- Time traits ----------------------------------------------------------------------
const SEC_TENTHS_TO_TIME: i64 = 10;
const SECS_TO_TIME: i64 = 10 * SEC_TENTHS_TO_TIME;

impl fmt::Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let secs = (self.t as f64) / (SECS_TO_TIME as f64);
        if f64::abs(secs) < 60.0 {
            write!(f, "{secs:.1}s")
        } else if f64::abs(secs) < 60.0 * 60.0 {
            write!(f, "{:.1}m", secs / 60.0)
        } else {
            write!(f, "{:.1}h", secs / (60.0 * 60.0))
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

impl Mul<i64> for Time {
    type Output = Self;

    fn mul(self, rhs: i64) -> Self::Output {
        Time { t: self.t * rhs }
    }
}
