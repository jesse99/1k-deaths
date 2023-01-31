use super::primitives::FoV;
use super::{Level, Point, PLAYER_ID};
use fnv::FnvHashSet;
use serde::{Deserialize, Serialize};

pub const RADIUS: i32 = 10; // TODO: should this depend on race or perception? or gear?

/// Field of View for a character. These are invalidated for certain events
/// (e.g. terrain changes).
#[derive(Serialize, Deserialize)]
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
    pub fn visible(&self, level: &Level, loc: Point) -> bool {
        assert!(!self.dirty);
        let player_loc = level.expect_location(PLAYER_ID);
        if loc.distance2(player_loc) <= RADIUS * RADIUS {
            self.visible.contains(&loc)
        } else {
            false
        }
    }

    // This can't be an ordinary method or we run into all sorts of borrowing grief.
    pub fn refresh(level: &mut Level) {
        if level.pov.dirty {
            let loc = level.expect_location(PLAYER_ID);
            PoV::do_refresh(level, &loc);
            level.pov.edition = level.pov.edition.wrapping_add(1);
            level.pov.dirty = false;
        }
    }

    // Game is mutable so that we can create a Cell if one isn't already there.
    fn do_refresh(level: &mut Level, origin: &Point) {
        level.pov.visible.clear();

        let mut new_locs = Vec::new();
        let mut view = FoV {
            start: *origin,
            radius: RADIUS,
            visible_tile: |loc| {
                new_locs.push(loc);
            },
            blocks_los: { |loc| level.blocks_los(loc) },
        };
        view.visit();

        for loc in new_locs {
            level.pov.visible.insert(loc);
        }
    }
}
