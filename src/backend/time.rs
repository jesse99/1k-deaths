// We have a few goals for the time system:
// 1) When the player does something like go down stairs we don't want a group of NPCs to
// all get off a slow heavy damage attack all at once: the player should always have some
// time to take some sort of action before those slow attacks get off.
// 2) When something like a big slow ice spell goes off the resist check should happen
// when the spell lands.
// 3) For, at least some actions, there should be a visible annoucement that the action is
// about to happen. For example, a message saying that a group of mages has started chanting
// in unison.
// 4) When a character follows an equal speed fleeing character it should be able to, once
// in a while, take an action besides just chasing the other character.
//
// For a while I was handling this by using two part actions: there was a ScheduledEvent
// that was added to a priority queue and when it was scheduled the associated events would
// fire. This was nice because push was constant time and pop was O(log(N)), but cancelling
// an action was O(N) and more importantly conflicting actions could be scheduled. For example,
// the player could move into cell C and deep water could flood into cell C. Both of these
// can be legit at the time they were scheduled but the two should not both be performed.
//
// To work around those icky sorts of issues I've moved towards a more traditional energy
// based system: objects accumulate time units and when they have enough time units they
// perform an action. When an object does an action the time it takes is given to all the
// other objects. So a wizard who casts a long spell may have to wait a while to cast it
// and once it goes off everything else will be able to do quite a lot.
use super::{Event, Oid};
use rand::rngs::SmallRng;
use rand::Rng;
use rand_distr::StandardNormal;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{self, Formatter};
use std::ops::{Add, Sub};

#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Time {
    t: i64,
}

impl Time {
    pub fn zero() -> Time {
        Time { t: 0 }
    }
}

/// Minimum time for actions is 1s although the times at which objects are scheduled is
/// fuzzed so that an object scheduled for 1s from now will actually execute in 1s +/- a
/// small delta.
pub fn secs(s: i64) -> Time {
    Time { t: s * SECS_TO_TIME }
}

pub enum Turn {
    Player,
    Npc(Oid, Event, Time),
    NoOne,
}

pub struct Scheduler {
    entries: Vec<Entry>,
    now: Time,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            entries: Vec::new(),
            now: Time { t: 0 },
        }
    }

    /// Player starts with a small amount of time units. NPCs start out with zero time
    /// units. That way the player will always have the first move. Other objects may
    /// start out with a negative time so that they execute some time in the future.
    pub fn add(&mut self, oid: Oid, initial: Time) {
        debug_assert!(!self.entries.iter().any(|entry| entry.oid == oid), "{oid} is already scheduled!");
        if oid.0 == 0 {
            assert!(initial.t >= SECS_TO_TIME, "player should start out with positive time units");
        } else {
            assert!(initial.t <= 0, "NPCs should start out with zero or negative time units");
        }
        // info!("added {oid} to the scheduler");
        self.entries.push(Entry { oid, units: initial });
    }

    pub fn remove(&mut self, oid: Oid) {
        // We can't just unwrap this because objects can elect to remove themselves from
        // scheduling if they have done everything they want to do.
        if let Some(index) = self.entries.iter().position(|entry| entry.oid == oid) {
            // info!("removing {oid} from the scheduler");
            self.entries.remove(index);
        }
    }

    /// Find an object that wants to do an action. The callback is given an oid with the
    /// amount of time the associated object has available. if the object elects to peform
    /// an action then it should return an event along with the duration of the event.
    /// 
    /// If this function returns Turn::Player then the UI will block for user input (note 
    /// that the player has an advantage because he is allowed to take a big action whenever 
    /// he has some time available. However he will go into the negative so other NPCs will 
    /// have a lot of time to take their own actions).
    /// 
    /// If this function returns Turn::Npc then an NPC (or more rarely some other sort of
    /// object) will perform an Action. If this function returns NoOne then either no one
    /// had enough time for an action or everyone elected to wait for more time units.
    pub fn find_actor<F>(&self, rng: &RefCell<SmallRng>, callback: F) -> Turn
    where
        F: Fn(Oid, Time) -> Option<(Event, Time)>,
    {
        let offset = {
            let rng = &mut *rng.borrow_mut();
            rng.gen_range(0..self.entries.len())
        };
        for i in 0..self.entries.len() {
            let index = (i + offset) % self.entries.len();
            let entry = self.entries[index];
            if entry.units.t >= SECS_TO_TIME {
                if entry.oid.0 == 0 {
                    // info!("{index}: scheduling player");
                    return Turn::Player;
                } else if let Some((event, duration)) = callback(entry.oid, entry.units) {
                    assert!(duration.t >= SECS_TO_TIME);
                    assert!(duration <= entry.units);
                    // info!("{index}: scheduling {}", entry.oid);
                    return Turn::Npc(entry.oid, event, duration);
                } else {
                    // info!("{index}: {} only had {} time units", entry.oid, entry.units);
                }
            }
        }
        Turn::NoOne
    }

    /// Called if find_actor returned NoOne. All objects are given a small amount of time.
    pub fn not_acted(&mut self) {
        // info!("no one acted");
        for entry in self.entries.iter_mut() {
            entry.units = entry.units + secs(6);
        }
    }

    /// Called whenever an object does an Action. taken is the duration of the action and
    /// added to everyone elses time units. extra is used to schedule oid further into the
    /// future than would normally be the case.
    pub fn acted(&mut self, oid: Oid, taken: Time, extra: Time, rng: &RefCell<SmallRng>) {
        assert!(taken.t >= SECS_TO_TIME);
        assert!(extra.t >= 0);

        let units = self.fuzz_time(taken, rng);
        // info!("{oid} acted and took {units}s");
        self.adjust_units(oid, units, extra);
    }
}

// ---- Private methods ------------------------------------------------------------------
impl Scheduler {
    fn adjust_units(&mut self, oid: Oid, taken: Time, extra: Time) {
        self.now = self.now + taken;

        for entry in self.entries.iter_mut() {
            if entry.oid == oid {
                entry.units = entry.units - taken - extra;
            } else {
                entry.units = entry.units + taken;

                // In theory the player can rest for an arbitrarily long time. NPCs can
                // also elect to do nothing but if they don't do anything for a long time
                // we'll assert because that's most likely a bug. TODO: we should be able
                // to use a much tighter bound when we stop flooding.
                if entry.oid.0 != 0 && entry.units.t > 100 * HOURS_TO_TIME {
                    let mut mesg = String::new();
                    for entry in &self.entries {
                        mesg += &format!("{} has {}s\n", entry.oid, entry.units);
                    }
                    panic!("{mesg}");
                }
            }
        }
    }

    fn fuzz_time(&self, units: Time, rng: &RefCell<SmallRng>) -> Time {
        let rng = &mut *rng.borrow_mut();
        let delta: f64 = rng.sample(StandardNormal); // most are in -2..2
        let delta = delta / 2.0; // most are in -1..1
        let delta = delta * 0.15 * ((units.t / SECS_TO_TIME) as f64); // most are in +/- 15% of units
        let max_delta = 0.3 * (units.t as f64);
        let delta = f64::clamp(delta, -max_delta, max_delta); // no more than +/- 30% of units

        let taken = units.t + (SECS_TO_TIME * delta as i64);
        let taken = i64::max(taken, 1); // time has to advance
        Time { t: taken }
    }
}

// ---- Time traits ----------------------------------------------------------------------
const SECS_TO_TIME: i64 = 100;
const MINS_TO_TIME: i64 = 60 * SECS_TO_TIME;
const HOURS_TO_TIME: i64 = 60 * MINS_TO_TIME;

impl fmt::Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let secs = (self.t as f64) / (SECS_TO_TIME as f64);
        write!(f, "{secs:.1}")
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

// ---- Entry struct ---------------------------------------------------------------------
#[derive(Clone, Copy, Eq, PartialEq)]
struct Entry {
    oid: Oid,
    /// Amount of time this object currently has to perform an action. To allow the player
    /// the first turn new objects start out with a negative time.
    units: Time,
}

impl Ord for Entry {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.units.cmp(&rhs.units)
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}
