//! These are the functions that UIs use to respond to player input.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

/// Most often this will move the player to a new location, but it's also used to attack
/// NPCs, and to interact with objects like doors.
pub fn bump(state: &mut State, dir: Direction) {
    use Direction::*;
    let delta = match dir {
        North => Point::new(0, -1),
        NorthEast => Point::new(1, -1),
        East => Point::new(1, 0),
        SouthEast => Point::new(1, 1),
        South => Point::new(0, 1),
        SouthWest => Point::new(-1, 1),
        West => Point::new(-1, 0),
        NorthWest => Point::new(-1, -1),
    };
    let loc = loc(state);
    if loc.x + delta.x >= 0 && loc.y + delta.x >= 0 {
        state.process(Event::MoveChar(Oid(0), loc + delta));
    } else {
        state.messages.push(Message::new(Topic::Failed, "Can't move there"))
    }
}

pub fn loc(state: &State) -> Point {
    state.char_to_loc.lookup(Oid(0))
}

fn special_terrain(state: &mut State, loc: &Point) -> bool {
    false
}
