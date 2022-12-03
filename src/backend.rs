//! Contains the game logic, i.e. everything but rendering, user input, and program initialization.
mod player_actions;
mod primitives;
mod relation;
mod store;
mod store2;
mod store_from_str;

// use player_actions::*;
use relation::*;
use store::*;
// use store_from_str::*;

pub use primitives::Point;
pub use primitives::Size;
pub use relation::{Character, Message, Portable, Terrain};

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
    store: Store,
}

impl Game {
    pub fn new() -> Game {
        let store = Store::from(include_str!("backend/maps/start.txt"));
        Game { store }
    }

    pub fn player_loc(&self) -> Point {
        self.store.expect_location(ObjectId::Player)
    }

    /// 1) If the loc is in the level and within the player's FoV then return the current
    /// state of the cell.
    /// 2) If the loc is a cell the player has seen in the past then return what he had
    /// seen (which may not be accurate now).
    /// 3) Otherwise return NotVisible.
    pub fn tile(&self, loc: Point) -> Tile {
        let oid = ObjectId::Cell(loc);
        match self.store.find(oid, RelationTag::Terrain) {
            Some(&Relation::Terrain(terrain)) => {
                let objects = self.store.find_objects(oid);
                let character = objects.and_then(|oids| oids.last().and_then(|oid| self.store.find_character(*oid)));
                let portables = objects
                    .unwrap_or(&Vec::new())
                    .iter()
                    .filter_map(|oid| self.store.find_portable(*oid))
                    .collect();
                let content = Content {
                    terrain,
                    character,
                    portables,
                };
                Tile::Visible(content)
            }
            _ => Tile::NotVisible,
        }
    }

    pub fn move_player(&mut self, dx: i32, dy: i32) {
        self.store.bump_player(dx, dy);
    }
}
