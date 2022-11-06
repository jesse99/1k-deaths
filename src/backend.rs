//! Contains the game logic, i.e. everything but rendering, user input, and program initialization.
mod primitives;
mod relation;
mod store;
mod store_from_str;

use relation::*;
use store::*;
use store_from_str::*;

pub use primitives::Point;

/// External API for the game state. Largely acts as a wrapper around the Store.
pub struct Game {
    store: Store,
}

impl Game {
    pub fn new() -> Game {
        let mut store = Store::new();
        let loc = Point::new(10, 10);
        store.create(ObjectId::Player, Relation::Location(loc));

        let oid = ObjectId::Cell(loc);
        store.create(oid, Relation::Objects(vec![ObjectId::Player]));
        store.create(oid, Relation::Terrain(Terrain::Dirt));

        let oid = ObjectId::DefaultCell;
        store.create(oid, Relation::Objects(vec![]));
        store.create(oid, Relation::Terrain(Terrain::Wall));

        Game { store }
    }

    pub fn player_loc(&self) -> Point {
        self.store.expect_location(ObjectId::Player)
    }

    pub fn move_player(&mut self, dx: i32, dy: i32) {
        let loc = self.player_loc();
        self.store
            .update(ObjectId::Player, Relation::Location(Point::new(loc.x + dx, loc.y + dy)));
    }
}
