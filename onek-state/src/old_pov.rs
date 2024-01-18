use super::{Cell, Game, Point, DEFAULT_CELL_ID};
use fnv::FnvHashMap;

/// Locations that were visible to a character. Note that PoV overrides
/// this so, as an optimization, this may include locations that are actually
/// visible. Currently this is only used for the Player to render locations
/// that he has seen before.
pub struct OldPoV {
    old: FnvHashMap<Point, Cell>, // may not match the current level state
    edition: u32,                 // current PoV edition
}

impl OldPoV {
    pub fn new() -> OldPoV {
        OldPoV {
            old: FnvHashMap::default(),
            edition: 0,
        }
    }

    // This can't be an ordinary method or we run into all sorts of borrowing grief.
    pub fn update(game: &mut Game) {
        if game.pov.edition() != game.old_pov.edition {
            for loc in game.pov.locations() {
                let default = vec![DEFAULT_CELL_ID];
                let oids = game.level.get(&loc).unwrap_or(&default);
                // TODO: should we create a trimmed object? something like just description, symbol, color
                // TODO: if we do we'd have to document this in messages/state.rd
                // TODO: think that makes sense: more efficient and player shouldn't be interacting with them
                let objects = oids.iter().map(|oid| game.objects.get(&oid).unwrap().clone()).collect();
                game.old_pov.old.insert(*loc, objects);
            }
            game.old_pov.edition = game.pov.edition();
        }
    }

    pub fn get(&self, loc: &Point) -> Option<&Cell> {
        self.old.get(loc)
    }
}
