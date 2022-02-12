//! There are three main classes of events:
//! 1) ScheduledAction events which happen in two parts: first the ScheduledAction is
//! scheduled to execute at a later time. When it does execute Action events are posted
//! to make the actual game changes.
//!
//! 2) Action events are the typical way that the game is modified. They normally happen
//! via ScheduledAction's.
//!
//! 3) Everything else. This includes events like AddMessage which don't take time to
//! execute as well as events used for construction like BeginConstructLevel.
// use super::time::{self, Time};
use super::{Message, Object, Oid, Point, State};
use std::fmt::{self, Formatter};

/// These are the events which take time to execute.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Action {
    Dig(Point, Oid, i32),            // (obj_loc, obj_oid, damage)
    FightRhulad(Point, Oid),         // (char_loc, char)
    FloodDeep(Point),                // (water_loc)
    FloodShallow(Point),             // (water_loc)
    Move(Point, Point),              // (old_loc, new_loc)
    OpenDoor(Point, Point, Oid),     // (ch_loc, obj_loc, obj_oid)
    PickUp(Point, Oid),              // (obj_loc, obj_oid)
    ShoveDoorman(Point, Oid, Point), // (old_loc, char, new_loc)
}

/// These are the "facts" associated with a particular game. All game state
/// should be able to be re-constructed from the event stream.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Event {
    Action(Oid, Action),
    AddObject(Point, Object), // used when building a level
    AddMessage(Message),
    EndConstructLevel,
    StateChanged(State),
    // Note that new variants MUST be added at the end (or saved games will break).
}

// ---- Display impls --------------------------------------------------------------------
impl fmt::Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Action::*;
        match self {
            Dig(loc, oid, damage) => write!(f, "DamageWall({loc}, {oid}, {damage})"),
            FightRhulad(ch_loc, ch) => write!(f, "FightRhulad({ch_loc}, {ch})"),
            FloodDeep(loc) => write!(f, "FloodDeep({loc})"),
            FloodShallow(loc) => write!(f, "FloodShallow({loc})"),
            Move(old_loc, new_loc) => write!(f, "Move({old_loc}, {new_loc})"),
            OpenDoor(ch_loc, obj_loc, obj_oid) => write!(f, "OpenDoor({ch_loc}, {obj_loc}, {obj_oid})"),
            PickUp(obj_loc, obj_oid) => write!(f, "PickUp({obj_loc}, {obj_oid})"),
            ShoveDoorman(old_loc, ch, new_loc) => write!(f, "ShoveDoorman({old_loc}, {ch}, {new_loc})"),
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Event::*;
        match self {
            Action(oid, action) => write!(f, "Action({oid}, {action})"),
            AddObject(loc, obj) => write!(f, "AddObject({loc}, {obj})"),
            AddMessage(mesg) => write!(f, "AddMessage({mesg})"),
            EndConstructLevel => write!(f, "EndConstructLevel"),
            StateChanged(state) => write!(f, "StateChanged({state})"),
        }
    }
}
