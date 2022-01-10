use super::details::Game1;
use super::event::Event;
use super::fov::FoV;
use super::level::{Level, Terrain};
use super::point::Point;
use fnv::FnvHashSet;

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

    pub fn posted(&mut self, game: &Game1, event: &Event) {
        match event {
            Event::NewGame => self.dirty = true,
            Event::NewLevel(_size) => self.dirty = true,
            Event::SetTerrain(loc, new_terrain) => {
                // Only dirty if the terrain change was something that would
                // change visibility.
                let old_terrain = game.level.terrain.get(loc).unwrap_or(&Terrain::Wall);
                let old_blocks = blocks_los(*old_terrain);
                let new_blocks = blocks_los(*new_terrain);
                if old_blocks != new_blocks {
                    self.dirty = true;
                }
            }
            // TODO: this should dirty only if the origin changes. Maybe we can add an id to PoV
            // and check to see if loc matches that id's location.
            Event::PlayerMoved(_loc) => self.dirty = true,
            _ => (),
        };
    }

    pub fn edition(&self) -> u32 {
        self.edition
    }

    /// Returns an iterator covering all the visible locations.
    pub fn locations(&self) -> std::collections::hash_set::Iter<'_, Point> {
        self.visible.iter()
    }

    /// Returns true if loc is visible from origin.
    pub fn visible(&mut self, origin: &Point, level: &Level, loc: &Point) -> bool {
        if self.dirty {
            self.refresh(origin, level);
            self.edition = self.edition.wrapping_add(1);
            self.dirty = false;
        }

        self.visible.contains(loc)
    }

    fn refresh(&mut self, origin: &Point, level: &Level) {
        self.visible.clear();

        let mut view = FoV {
            start: *origin,
            size: level.size,
            radius: 15, // TODO: do better with this
            visible_tile: |loc| {
                self.visible.insert(loc);
            },
            blocks_los: |loc| match level.terrain.get(&loc) {
                Some(terrain) => blocks_los(*terrain),
                None => true,
            },
        };
        view.visit();
    }
}

fn blocks_los(terrain: Terrain) -> bool {
    match terrain {
        Terrain::ClosedDoor => true,
        Terrain::DeepWater => false,
        Terrain::ShallowWater => false,
        Terrain::Wall => true,
        Terrain::Ground => false,
    }
}
