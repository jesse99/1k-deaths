//! Contains the game logic, i.e. everything but rendering, user input, and program initialization.
mod level;
mod point;

pub use level::{Level, Terrain};
pub use point::Point;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum GameState {
    Running,
    Exiting,
}
