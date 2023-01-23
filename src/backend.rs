//! Contains the game logic, i.e. everything but rendering, user input, and program initialization.
mod facts;
mod player_actions;
mod primitives;
mod relation;
mod store2;
mod store_from_str;

use facts::*;
// use player_actions::*;
use store2::*;

pub use facts::{Character, Portable, Terrain};
pub use primitives::Point;
pub use primitives::Size;

// use self::relation::Character3;

#[derive(Debug, Eq, PartialEq)]
pub struct Content {
    pub terrain: Terrain,
    pub character: Option<Character>,
    pub portables: Vec<Portable>,
    // TODO: non-portable objects vector, e.g. traps or fountains
}

#[derive(Debug, Eq, PartialEq)]
pub enum Tile {
    /// player can see this
    Visible(Content),

    /// player can't see this but has in the past, note that this may not reflect the current state
    Stale(Content),

    /// player has never seen this location (and it may not exist)
    NotVisible,
}

/// External API for the game state. Largely acts as a wrapper around the Store.
pub struct Game {
    level: Level,
}

impl Game {
    pub fn new() -> Game {
        let level = Level::from(include_str!("backend/maps/start.txt"));
        Game { level }
    }

    pub fn player_loc(&self) -> Point {
        self.level.expect_location(PLAYER_ID)
    }

    /// 1) If the loc is in the level and within the player's FoV then return the current
    /// state of the cell.
    /// 2) If the loc is a cell the player has seen in the past then return what he had
    /// seen (which may not be accurate now).
    /// 3) Otherwise return NotVisible.
    pub fn tile(&self, loc: Point) -> Tile {
        let terrain = self.level.get_terrain(loc);
        let character = self.level.find_char(loc);
        let portables = self.level.get_portables(loc);
        Tile::Visible(Content {
            terrain,
            character,
            portables,
        })
    }

    pub fn move_player(&mut self, dx: i32, dy: i32) {
        self.level.bump_player(dx, dy);
    }
}
