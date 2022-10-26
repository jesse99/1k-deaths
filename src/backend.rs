//! Contains the game logic, i.e. everything but rendering, user input, and program initialization.
mod primitives;
mod store;

use store::*;

pub use primitives::Point;

// TODO: Oids are not very intelligible. If that becomes an issue we could use a newtype
// string (e.g. "wall 97") or a simple struct with a static string ref and a counter.

/// Used to identify an object within the Store.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Oid(u32); // TODO: seems very large but is there a chance this can overflow?

/// External API for the game state. Largely acts as a wrapper around the Store.
pub struct Game {
    store: Store,
}

impl Game {
    pub fn new() -> Game {
        let mut store = Store::new();
        store.create(Oid(0), Relation::Location(Point::origin()));
        Game { store }
    }

    pub fn player_loc(&self) -> Point {
        *self.store.expect_location(Oid(0))
    }

    pub fn move_player(&mut self, dx: i32, dy: i32) {
        let loc = self.player_loc();
        self.store
            .update(Oid(0), Relation::Location(Point::new(loc.x + dx, loc.y + dy)));
    }
}
