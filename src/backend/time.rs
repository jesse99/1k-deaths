// We have a few goals for the time system:
// 1) When the player does something like go down stairs we don't want a group of NPCs to
// all get off a slow heavy damage attack all at once: the player should always have some
// time to take some sort of action before those slow attachs get off.
// 2) When something like a big slow ice spell goes off the resist check should happen
// when the spell lands.
// 3) For, at least some actions, there should be a visible annoucement that the action is
// about to happen. For example, a message saying that a group of mages has started chanting
// in unison.
// 4) When a character follows an equal speed fleeing character it should be able to, once
// in a while, take an action besides just chasing the other character.
//
// We handle #1 through #3 by using ScheduleAction to queue up an Action to be performed later.
// #3 is addressed by using a normal distribution to randomize the time at which Scheduled
// Actions fire.
//
// Roguelikes often handle this using an energy system (see https://www.reddit.com/r/roguelikedev/comments/4pk2k6/faq_friday_41_time_systems/).
// Each object accumulates time units and actions occur when it accumulates enough. This
// seems like it would handle #1 and #2 OK, doesn't provide a natural way to handle #3,
// and could be augmented to handle #4. The biggest disadvantage with it is that, while it
// is a simpler design, it requires that all objects accumulate time units whenever another
// object takes an action. Maybe that's not so bad in practice but it would introduce an
// O(N) algorithm in the hot code path which doesn't seem great.
use super::{Oid, ScheduledAction};
use fnv::FnvHashSet;
use rand::rngs::SmallRng;
use rand::Rng;
use rand_distr::StandardNormal;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fmt::{self, Formatter};
use std::ops::{Add, Sub};

#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Time {
    t: i64,
}

/// Minimum time for actions is 1s although the times at which objects are scheduled is
/// fuzzed so that an object scheduled for 1s from now will actually execute in 1s +/- a
/// small delta.
pub fn secs(s: i64) -> Time {
    assert!(s > 0, "times shouldn't be zero");
    Time { t: s * SECS_TO_TIME }
}

pub struct Scheduler {
    heap: BinaryHeap<Entry>,
    now: Time,
    player: bool,
    scheduled: FnvHashSet<Oid>,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            heap: BinaryHeap::new(),
            now: Time { t: 0 },
            scheduled: FnvHashSet::default(),
            player: false,
        }
    }

    /// Used at the beginning of the world to check if we should block for user input.
    pub fn has_player(&self) -> bool {
        self.player
    }

    pub fn push(&mut self, oid: Oid, saction: ScheduledAction, delay: Time, rng: &RefCell<SmallRng>) {
        // Can't schedule an object more than once (we'd have to find the existing entry
        // and update the time for whichever expires later).
        debug_assert!(
            self.scheduled.insert(oid),
            "{oid} is already scheduled (scheduling {saction})"
        );

        let rng = &mut *rng.borrow_mut();
        let delta: f64 = rng.sample(StandardNormal); // most are in -2..2
        let delta = delta / 2.0; // most are in -1..1
        let delta = delta * 0.15 * ((delay.t / SECS_TO_TIME) as f64); // most are in +/- 15% of delay
        let max_delta = 0.3 * (delay.t as f64);
        let delta = f64::clamp(delta, -max_delta, max_delta); // no more than +/- 30% of delay

        let delay = delay.t + (SECS_TO_TIME * delta as i64);
        let delay = i64::max(delay, 1); // time has to advance
        let entry = Entry {
            oid,
            saction,
            at: Time { t: self.now.t + delay },
        };
        // trace!("scheduled {oid} and {saction} for {}", entry.at);
        self.heap.push(entry);
        if oid.0 == 0 {
            self.player = true;
        }
    }

    /// This will cancel any pending actions for oid.
    pub fn force_push(&mut self, oid: Oid, saction: ScheduledAction, delay: Time, rng: &RefCell<SmallRng>) {
        // After this old_heap will be the original heap and self.heap will be empty.
        let mut old_heap = BinaryHeap::with_capacity(self.heap.len());
        std::mem::swap(&mut self.heap, &mut old_heap);

        // Push all the entries into self.heap other than those that match oid (there
        // should be one of those).
        let entries = old_heap.iter().filter(|e| e.oid != oid);
        for entry in entries {
            self.heap.push(*entry);
        }

        // This is all rather slow but it should be a rare operation...
        self.scheduled.remove(&oid);
        self.push(oid, saction, delay, rng);
    }

    pub fn pop(&mut self) -> (Oid, ScheduledAction) {
        let entry = self.heap.pop().unwrap();
        {
            #[cfg(debug_assertions)]
            self.pop_scheduled(entry.oid);
        }
        self.now = entry.at;
        if entry.oid.0 == 0 {
            self.player = false;
        }
        // trace!("popping {} and {}", entry.oid, entry.saction);
        (entry.oid, entry.saction)
    }

    #[cfg(debug_assertions)]
    pub fn pop_scheduled(&mut self, oid: Oid) {
        self.scheduled.remove(&oid);
    }
}

// ---- Time traits ----------------------------------------------------------------------
const SECS_TO_TIME: i64 = 100;

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
    saction: ScheduledAction,
    at: Time,
}

impl Ord for Entry {
    fn cmp(&self, rhs: &Self) -> Ordering {
        rhs.at.cmp(&self.at) // reversed so that the heap has the smallest time at the top
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}
