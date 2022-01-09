use super::level::Terrain;
use super::point::Point;
use super::size::Size;

/// These are the "facts" associated with a particular game. All game state
/// should be able to be re-constructed from the event stream.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Event {
    NewGame,
    NewLevel(Size),
    SetTerrain(Point, Terrain),
    PlayerMoved(Point),
}
