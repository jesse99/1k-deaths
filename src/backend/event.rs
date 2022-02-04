use super::{Message, ObjId, Object, Point, State};
use std::fmt::{self, Formatter};

/// These are the "facts" associated with a particular game. All game state
/// should be able to be re-constructed from the event stream.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Event {
    AddMessage(Message),
    NewGame,
    StateChanged(State),
    BeginConstructLevel,
    EndConstructLevel,
    AddObject(Point, Object),
    AddToInventory(Point), // TODO: this will likely need to take a character id, and maybe an item id
    ReplaceObject(Point, ObjId, Object), // TODO: later might want an enum instead of a Point
    DestroyObjectId(Point, ObjId),
    PlayerMoved(Point),
    NPCMoved(Point, Point),
    // Note that new variants MUST be added at the end (or saved games will break).
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Event::AddMessage(mesg) => write!(f, "AddMessage({mesg})"),
            Event::NewGame => write!(f, "NewGame"),
            Event::StateChanged(state) => write!(f, "StateChanged({state})"),
            Event::BeginConstructLevel => write!(f, "BeginConstructLevel"),
            Event::EndConstructLevel => write!(f, "EndConstructLevel"),
            Event::AddObject(loc, obj) => write!(f, "AddObject({loc}, {obj})"),
            Event::AddToInventory(loc) => write!(f, "AddToInventory({loc})"),
            Event::ReplaceObject(loc, id, obj) => write!(f, "ReplaceObject({loc}, {id}, {obj})"),
            Event::DestroyObjectId(loc, id) => write!(f, "DestroyObjectId({loc}, {id})"),
            Event::PlayerMoved(loc) => write!(f, "PlayerMoved({loc})"),
            Event::NPCMoved(old, new) => write!(f, "NPCMoved({old}, {new})"),
        }
    }
}
