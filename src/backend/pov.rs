use super::event::Event;
use super::fov::FoV;
use super::level::{Level, Terrain};
use super::point::Point;
use super::EventPosted;
use fnv::FnvHashSet;

/// Field of View for a character. These are invalidated for certain events
/// (e.g. terrain changes).
pub struct PoV {
    visible: FnvHashSet<Point>,
    dirty: bool, // true if visible is invalid
}

impl PoV {
    pub fn new() -> PoV {
        PoV {
            visible: FnvHashSet::default(),
            dirty: true,
        }
    }

    pub fn visible(&mut self, origin: &Point, level: &Level, loc: &Point) -> bool {
        if self.dirty {
            self.refresh(origin, level);
            self.dirty = false;
        }

        self.visible.contains(loc)
    }

    fn refresh(&mut self, origin: &Point, level: &Level) {
        self.visible.clear();

        let mut view = FoV {
            start: *origin,
            size: level.size,
            radius: 20, // TODO: do better with this
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

impl EventPosted for PoV {
    fn posted(&mut self, event: Event) {
        match event {
            Event::NewGame => self.dirty = true,
            Event::NewLevel {
                width: _,
                height: _,
            } => self.dirty = true,
            Event::SetTerrain(_loc, _terrain) => {
                // TODO: try changing the signature so that it takes a Game reference
                // TODO: would have to add a warning that game fields may not be updated
                // TODO: dirty only if terrain changes visibility
                // TODO: can we do something similar with moved? maybe add an id field and check level to see if id's match?
                self.dirty = true;
            }
            Event::PlayerMoved(_loc) => self.dirty = true,
        };
    }
}
