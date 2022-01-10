use super::level::Terrain;
use super::Message;
use super::Point;
use super::Size;

/// These are the "facts" associated with a particular game. All game state
/// should be able to be re-constructed from the event stream.
#[derive(Clone, Eq, PartialEq)]
pub enum Event {
    AddMessage(Message),
    NewGame,
    NewLevel(Size),
    SetTerrain(Point, Terrain),
    PlayerMoved(Point),
}
