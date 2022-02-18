use super::primitives::FoV;
use super::{Game, Object, Oid, Point};
use fnv::FnvHashSet;

pub const RADIUS: i32 = 10; // TODO: should this depend on race or perception? or gear?

/// Field of View for a character. These are invalidated for certain events
/// (e.g. terrain changes).
pub struct PoV {
    edition: u32, // incremented each time visible is updated
    visible: FnvHashSet<Point>,
    dirty: bool, // true if visible is invalid
}

impl PoV {
    pub fn new() -> PoV {
        PoV {
            edition: 0,
            visible: FnvHashSet::default(),
            dirty: true,
        }
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
    pub fn visible(&self, loc: &Point) -> bool {
        assert!(!self.dirty);
        self.visible.contains(loc)
    }

    // This can't be an ordinary method or we run into all sorts of borrowing grief.
    pub fn refresh(game: &mut Game) {
        if game.pov.dirty {
            let loc = game.player_loc();
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
            blocks_los: { |loc| blocks_los(game.lookup.cell_iter(&loc)) },
        };
        view.visit();

        for loc in new_locs {
            game.pov.visible.insert(loc);
        }
    }
}

fn blocks_los<'a>(objs: impl Iterator<Item = (Oid, &'a Object)>) -> bool {
    let mut count = 0;
    for obj in objs {
        if obj.1.blocks_los() {
            return true;
        }
        count += 1;
    }
    count == 0 // non-existent cell
}
