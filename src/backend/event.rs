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
use super::{Message, Object, Oid, Point, State};
use std::fmt::{self, Formatter};

/// These are used to initiate an Action which requires some time before it actually
/// happens.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ScheduledAction {
    DamageWall(Point, Oid),          // (obj_loc, obj_oid))
    FightRhulad(Point, Oid),         // (char_loc, char)
    FloodDeep(Point),                // (water_loc)
    FloodShallow(Point),             // (water_loc)
    Move(Point, Point),              // (old_loc, new_loc)
    OpenDoor(Point, Point, Oid),     // (ch_loc, obj_loc, obj_oid)
    PickUp(Point, Oid),              // (obj_loc, obj_oid)
    ShoveDoorman(Point, Oid, Point), // (old_loc, char, new_loc)
}

/// These normally happen after a ScheduledAction and mutate the game somehow.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Action {
    AddObject(Point, Object),          // (obj_loc, obj)
    DestroyObject(Point, Oid),         // (obj_loc, obj_oid))
    Move(Oid, Point, Point),           // (char, old_loc, new_loc)
    PickUp(Oid, Point, Oid),           // typically (char_oid, obj_loc, obj_oid)
    ReplaceObject(Point, Oid, Object), // (obj_loc, old_obj_oid, new_obj)
}

/// These are the "facts" associated with a particular game. All game state
/// should be able to be re-constructed from the event stream.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Event {
    Action(Action),
    AddMessage(Message),
    BeginConstructLevel,
    EndConstructLevel,
    NewGame,
    ScheduledAction(Oid, ScheduledAction), // oid is usually a Character
    ForceAction(Oid, ScheduledAction),     // like ScheduledAction except that it cancels pending actions
    StateChanged(State),
    // Note that new variants MUST be added at the end (or saved games will break).
}

// ---- Display impls --------------------------------------------------------------------
impl fmt::Display for ScheduledAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use ScheduledAction::*;
        match self {
            DamageWall(loc, oid) => write!(f, "DamageWall({loc}, {oid})"),
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

impl fmt::Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Action::*;
        match self {
            AddObject(loc, obj) => write!(f, "AddObject({loc}, {obj})"),
            DestroyObject(loc, oid) => write!(f, "DestroyObject({loc}, {oid})"),
            Move(oid, old_loc, new_loc) => write!(f, "Move({oid}, {old_loc}, {new_loc})"),
            PickUp(oid, obj_loc, obj_oid) => write!(f, "PickUp({oid}, {obj_loc}, {obj_oid})"),
            ReplaceObject(obj_loc, obj_oid, new_obj) => write!(f, "ReplaceObject({obj_loc}, {obj_oid}, {new_obj})"),
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Event::*;
        match self {
            Action(action) => write!(f, "Action({action})"),
            AddMessage(mesg) => write!(f, "AddMessage({mesg})"),
            BeginConstructLevel => write!(f, "BeginConstructLevel"),
            EndConstructLevel => write!(f, "EndConstructLevel"),
            NewGame => write!(f, "NewGame"),
            ScheduledAction(oid, action) => write!(f, "ScheduledAction({oid}, {action})"),
            ForceAction(oid, action) => write!(f, "ForceAction({oid}, {action})"),
            StateChanged(state) => write!(f, "StateChanged({state})"),
        }
    }
}
