use super::{Message, Object, Point, ProbeMode, Tag};

/// These are the "facts" associated with a particular game. All game state
/// should be able to be re-constructed from the event stream.
#[derive(Clone)]
pub enum Event {
    AddMessage(Message),
    NewGame,
    NewLevel,
    AddObject(Point, Object),
    ChangeObject(Point, Tag, Object),
    ChangeProbe(ProbeMode),
    PlayerMoved(Point),
}
