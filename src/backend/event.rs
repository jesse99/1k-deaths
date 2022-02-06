use super::{Message, ObjId, Object, Point, State};
use std::fmt::{self, Formatter};

/// These are used to initiate an Action which requires some time before it actually
/// happens.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ScheduledAction {
    DamageWall(Point, ObjId),          // (obj_loc, obj_oid))
    FightRhulad(Point, ObjId),         // (char_loc, char)
    Move(Point, Point),                // (old_loc, new_loc)
    OpenDoor(Point, Point, ObjId),     // (ch_loc, obj_loc, obj_oid)
    PickUp(Point, ObjId),              // (obj_loc, obj_oid)
    ShoveDoorman(Point, ObjId, Point), // (old_loc, char, new_loc)
}

/// These normally happen after a ScheduledAction and mutate the game somehow.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Action {
    AddObject(Point, Object),            // (obj_loc, obj)
    DestroyObject(Point, ObjId),         // (obj_loc, obj_oid))
    Move(ObjId, Point, Point),           // (char, old_loc, new_loc)
    PickUp(ObjId, Point, ObjId),         // typically (char_oid, obj_loc, obj_oid)
    ReplaceObject(Point, ObjId, Object), // (obj_loc, old_obj_oid, new_obj)
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
    ScheduledAction(ObjId, ScheduledAction), // oid is usually a Character
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
            StateChanged(state) => write!(f, "StateChanged({state})"),
        }
    }
}
