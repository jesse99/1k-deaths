mod primitives;

pub use primitives::Color;
pub use primitives::Point;
pub use primitives::Size;

/// Ecapsulates all the backend game state. All the fields and methods are private so
/// UIs must use the render and input sub-modules.
pub struct State {
    player_loc: Point, // TODO: replace this with indexing into chars position
}

impl State {
    pub fn new() -> State {
        State {
            player_loc: Point::new(20, 20),
        }
    }
}

// TODO: move into a file
pub mod render {
    //! These are the functions that UIs use when rendering.
    use super::*;

    pub fn player_loc(state: &State) -> Point {
        state.player_loc
    }
}

// TODO: move into a file
pub mod player {
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
        state.player_loc.x += delta.x;
        state.player_loc.y += delta.y;
    }
}
