use super::{Game, Point, Terrain};
use fnv::FnvHashMap;

/// Locations that were visible to a character. Note that PoV overrides
/// this so, as an optimization, this may include locations that are actually
/// visible. Currently this is only used for the Player to render locations
/// that he has seen before.
pub struct OldPoV {
    old: FnvHashMap<Point, Terrain>, // may not match the current Level state
    edition: u32,                    // current PoV edition
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
                let terrain = game.terrain.get(&loc).unwrap_or(&Terrain::Unknown);
                game.old_pov.old.insert(*loc, *terrain);
            }
            game.old_pov.edition = game.pov.edition();
        }
    }

    pub fn get(&self, loc: &Point) -> Option<&Terrain> {
        self.old.get(loc)
    }
}
