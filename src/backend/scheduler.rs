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
// perform an action. When an object does an action it decrements its time units accordingly.
// When all objects have had a chance to move time is advanced an all objects are given
// that bit of extra time. So a wizard who casts a long spell may have to wait a while to
// cast it and once it goes off everything else will be able to do quite a lot while the
// wizard is recovering.
use super::ai::{self, Acted};
use super::time;
use super::{Action, Game, Oid, Time};
use fnv::FnvHashMap;
use rand::prelude::SliceRandom;
use rand::rngs::SmallRng;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::io::{Error, Write};

pub struct Scheduler {
    entries: FnvHashMap<Oid, Time>,
    now: Time,
    round: Vec<Entry>, // objects who are given a chance to move before time advances
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            entries: FnvHashMap::default(),
            now: Time::zero(),
            round: Vec::new(),
        }
    }

    pub fn now(&self) -> Time {
        self.now
    }

    /// Player starts with a small amount of time units. NPCs start out with zero time
    /// units. That way the player will always have the first move. Other objects may
    /// start out with a negative time so that they execute some time in the future.
    pub fn add(&mut self, oid: Oid, initial: Time) {
        if oid.0 == 0 {
            assert!(
                initial >= time::MIN_TIME,
                "player should start out with positive time units"
            );
        } else {
            assert!(
                initial <= Time::zero(),
                "NPCs should start out with zero or negative time units"
            );
        }
        // info!("added {oid} to the scheduler");
        let old = self.entries.insert(oid, initial);
        debug_assert!(old.is_none(), "{oid} is already scheduled!");
    }

    pub fn remove(&mut self, oid: Oid) {
        // Note that objects can remove themselves from scheduling if they have nothing
        // left to do so this may be a no-op.
        self.entries.remove(&oid);
    }

    /// Iterates through all objects in the current round until one performs an action.
    pub fn player_is_ready(game: &mut Game) -> bool {
        // To ensure fairness all objects with the minimum action time are collected
        // together into a "round". Once they have all had a chance to move time advances
        // and a new round starts.
        if game.scheduler.round.is_empty() {
            let mut items: Vec<Entry> = game
                .scheduler
                .entries
                .iter()
                .filter_map(|(&oid, &units)| {
                    if units >= time::MIN_TIME {
                        Some(Entry { oid, units })
                    } else {
                        None
                    }
                })
                .collect();
            items.shuffle(&mut *game.rng());
            game.scheduler.round = items;
        }
        while !game.scheduler.round.is_empty() {
            let entry = game.scheduler.round.pop().unwrap();
            if entry.oid.0 == 0 {
                // The player can move whenever he has a bit of time. This may once in a
                // while matter but he will go into negative time units which will allow
                // NPCs to do more.
                return true;
            } else {
                match ai::acted(game, entry.oid, entry.units) {
                    Acted::Acted(duration) => {
                        assert!(duration >= time::MIN_TIME);
                        assert!(duration <= entry.units);
                        game.scheduler.obj_acted(entry.oid, duration, &game.rng);
                        return false;
                    }
                    Acted::DidntAct => (),
                    Acted::Removed => {
                        game.stream.push(Action::Object);
                        return false; // there's been some sort of state change so the UI may need to update
                    }
                }
            }
        }
        game.scheduler.advance_time();
        false
    }

    pub fn player_acted(&mut self, taken: Time, rng: &RefCell<SmallRng>) {
        assert!(taken >= time::MIN_TIME);
        let taken = taken.fuzz(rng);
        let units = self.entries.get_mut(&Oid(0)).unwrap();
        *units -= taken;
        trace!("   player acted for {taken} and has {units}");
    }

    /// This is used when an object causes another object to use up some of its time.
    /// Examples of this include stunning a character or a stronger character shoving a
    /// weaker one out of the way.
    pub fn force_acted(&mut self, oid: Oid, taken: Time, rng: &RefCell<SmallRng>) {
        assert!(taken >= time::MIN_TIME);
        let taken = taken.fuzz(rng);
        if let Some(units) = self.entries.get_mut(&oid) {
            *units -= taken;
            trace!("   {oid} forced acted for {taken} and has {units}");
        }
    }

    pub fn dump<W: Write>(&self, writer: &mut W, game: &Game) -> Result<(), Error> {
        write!(writer, "scheduler is at {}\n", self.now)?;

        let mut items: Vec<Entry> = self.entries.iter().map(|(&oid, &units)| Entry { oid, units }).collect();
        items.sort_by(|a, b| a.units.partial_cmp(&b.units).unwrap());

        write!(writer, "   oid  units dname\n")?;
        for entry in items.iter().rev() {
            let obj = game.level.obj(entry.oid).0;
            write!(writer, "   {} {} {}\n", entry.oid, entry.units, obj.dname())?;
        }
        Ok(())
    }
}

// ---- Private methods ------------------------------------------------------------------
impl Scheduler {
    fn advance_time(&mut self) {
        self.now += time::DIAGNOL_MOVE;
        for units in self.entries.values_mut() {
            *units += time::DIAGNOL_MOVE;
        }
    }

    fn obj_acted(&mut self, oid: Oid, taken: Time, rng: &RefCell<SmallRng>) {
        assert!(taken >= time::MIN_TIME);
        assert!(oid.0 != 0);

        let taken = taken.fuzz(rng);
        let units = self.entries.get_mut(&oid).unwrap();
        *units -= taken;
        trace!("   {oid} acted for {taken} and has {units}");
    }
}

// ---- Entry struct ---------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Entry {
    oid: Oid,
    /// Amount of time this object currently has to perform an action.
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
