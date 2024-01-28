use super::FoV;
use super::{Game, Point};
use fnv::FnvHashSet;
use onek_shared::{Oid, DEFAULT_CELL_OID};

pub const RADIUS: i32 = 10; // TODO: should this depend on race or perception? or gear?

/// Field of View for a character. These are invalidated for certain events
/// (e.g. terrain changes).
pub struct PoV {
    edition: u32, // incremented each time visible is updated
    pub visible: FnvHashSet<Point>,
    dirty: bool, // true if visible is invalid
}

impl PoV {
    pub fn new() -> PoV {
        PoV {
            edition: 0,
            visible: FnvHashSet::default(),
            dirty: false,
        }
    }

    pub fn reset(&mut self) {
        self.edition = 0;
        self.visible.clear();
        self.dirty = true;
    }

    pub fn dirty(&mut self) {
        self.dirty = true;
    }

    pub fn edition(&self) -> u32 {
        self.edition
    }

    /// Returns an iterator covering all the visible locations.
    pub fn locations(&self) -> std::collections::hash_set::Iter<'_, Point> {
        self.visible.iter()
    }

    /// Returns true if loc is visible from origin.
    pub fn visible(&self, game: &Game, loc: &Point) -> bool {
        assert!(!self.dirty);
        if loc.distance2(game.player_loc) <= RADIUS * RADIUS {
            self.visible.contains(loc)
        } else {
            false
        }
    }

    // This can't be an ordinary method or we run into all sorts of borrowing grief.
    pub fn refresh(game: &mut Game) {
        if game.pov.dirty {
            let loc = game.player_loc;
            PoV::do_refresh(game, &loc);
            game.pov.edition = game.pov.edition.wrapping_add(1);
            game.pov.dirty = false;
        }
    }

    // Game is mutable so that we can create a Cell if one isn't already there.
    fn do_refresh(game: &mut Game, origin: &Point) {
        game.pov.visible.clear();

        let mut new_locs = Vec::new();
        let mut view = FoV {
            start: *origin,
            radius: RADIUS,
            visible_tile: |loc| {
                new_locs.push(loc);
            },
            blocks_los: { |loc| blocks_los(game, &loc) },
        };
        view.visit();

        for loc in new_locs {
            game.pov.visible.insert(loc);
        }
    }
}

fn oid_blocks_los(game: &Game, oid: &Oid) -> bool {
    let object = game.objects.get(oid).unwrap();
    if let Some(blocks) = object.get("blocks_los") {
        blocks.to_bool()
    } else {
        false
    }
}

fn blocks_los<'a>(game: &Game, loc: &Point) -> bool {
    if let Some(oids) = game.level.get(&loc) {
        for oid in oids.iter() {
            if oid_blocks_los(game, oid) {
                return true;
            }
        }
        false
    } else {
        oid_blocks_los(game, &DEFAULT_CELL_OID)
    }
}
