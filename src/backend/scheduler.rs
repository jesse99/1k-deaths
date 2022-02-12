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
use super::time;
use super::{Game, Oid, Time};
use rand::rngs::SmallRng;
use rand::Rng;
use std::cell::RefCell;
use std::cmp::Ordering;

pub struct Scheduler {
    entries: Vec<Entry>,
    now: Time,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            entries: Vec::new(),
            now: Time::zero(),
        }
    }

    /// Player starts with a small amount of time units. NPCs start out with zero time
    /// units. That way the player will always have the first move. Other objects may
    /// start out with a negative time so that they execute some time in the future.
    pub fn add(&mut self, oid: Oid, initial: Time) {
        debug_assert!(
            !self.entries.iter().any(|entry| entry.oid == oid),
            "{oid} is already scheduled!"
        );
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

    /// Find the next object with enough time units to perform the action it wants to do.
    /// Note that the player has an advantage because he is allowed to take an action
    /// whenever he has the minimum amount of time available. However he will go into the
    /// negative so other NPCs will have a lot of time to take their own actions).
    pub fn players_turn(game: &mut Game) -> bool {
        let offset = {
            let rng = &mut *game.rng.borrow_mut();
            rng.gen_range(0..game.scheduler.entries.len())
        };
        for _ in 0..100 {
            for i in 0..game.scheduler.entries.len() {
                let index = (i + offset) % game.scheduler.entries.len();
                let entry = game.scheduler.entries[index];
                if entry.units >= time::MIN_TIME {
                    if entry.oid.0 == 0 {
                        return true;
                    } else if let Some((duration, extra)) = game.obj_acted(entry.oid, entry.units) {
                        assert!(duration >= time::MIN_TIME);
                        assert!(duration <= entry.units);
                        assert!(extra >= Time::zero());
                        game.scheduler.obj_acted(entry.oid, duration, extra, &game.rng);
                        return false;
                    }
                }
            }
            game.scheduler.not_acted();
        }
        panic!("At least the player should have moved!");
    }

    pub fn player_acted(&mut self, taken: Time, rng: &RefCell<SmallRng>) {
        assert!(taken >= time::MIN_TIME);

        let taken = taken.fuzz(rng);
        let extra = Time::zero();
        self.adjust_units(Oid(0), taken, extra);
    }

    /// This is used when an object causes another object to use up some of it's time.
    /// Examples of this include stunning a character or a stronger character shoving a
    /// weaker one out of the way. Note that this just subtracts the time from oid: it does
    /// not give time to other objects.
    pub fn force_acted(&mut self, oid: Oid, taken: Time, rng: &RefCell<SmallRng>) {
        assert!(taken >= time::MIN_TIME);

        let taken = taken.fuzz(rng);
        for entry in self.entries.iter_mut() {
            if entry.oid == oid {
                entry.units = entry.units - taken;
                break;
            }
        }
    }
}

// ---- Private methods ------------------------------------------------------------------
impl Scheduler {
    fn not_acted(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.units = entry.units + time::DIAGNOL_MOVE;
        }
    }

    fn obj_acted(&mut self, oid: Oid, taken: Time, extra: Time, rng: &RefCell<SmallRng>) {
        assert!(taken >= time::MIN_TIME);
        assert!(extra >= Time::zero());

        let units = taken.fuzz(rng);
        self.adjust_units(oid, units, extra);
    }

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
                if entry.oid.0 != 0 && entry.units > time::mins(100 * 60) {
                    let mut mesg = String::new();
                    for entry in &self.entries {
                        mesg += &format!("{} has {}s\n", entry.oid, entry.units);
                    }
                    panic!("{mesg}");
                }
            }
        }
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
