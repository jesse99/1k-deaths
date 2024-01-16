use super::FoV;
use super::{Game, Point, Terrain};
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
            blocks_los: { |loc| blocks_los(game.terrain.get(&loc)) },
        };
        view.visit();

        for loc in new_locs {
            game.pov.visible.insert(loc);
        }
    }
}

// TODO: This is a bit tricky: we'd like objects (like a closed door or some statues) to
// block LOS but we want to minimize logic in the state service. Probably should handle
// this by having a blocks_los field in objects.
fn blocks_los<'a>(t: Option<&Terrain>) -> bool {
    match t {
        Some(terrain) => match terrain {
            Terrain::DeepWater => false,
            Terrain::Dirt => false,
            Terrain::ShallowWater => false,
            Terrain::Wall => true,
            Terrain::Unknown => true,
        },
        None => true, // non-existent cell
    }
}
