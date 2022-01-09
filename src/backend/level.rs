use super::event::Event;
use super::point::Point;
use super::size::Size;
use fnv::FnvHashMap;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Terrain {
    // TODO: this will change in the future when we move towards more of a component system
    ClosedDoor,
    DeepWater,
    Ground,
    ShallowWater,
    Wall,
}

pub struct Level {
    pub size: Size,
    pub player: Point,
    pub terrain: FnvHashMap<Point, Terrain>, // TODO: does a hash map really help? could use Vec2d
}

impl Level {
    pub fn new() -> Level {
        Level {
            size: Size::zero(),
            player: Point::origin(),
            terrain: FnvHashMap::default(),
        }
    }

    pub fn posted(&mut self, event: Event) {
        match event {
            Event::NewLevel(new_size) => {
                self.size = new_size;
                self.player = Point::new(20, 10); // TODO: need to do better here
                self.terrain = FnvHashMap::default();
            }
            Event::SetTerrain(loc, terrain) => {
                self.terrain.insert(loc, terrain);
            }
            Event::PlayerMoved(loc) => self.player = loc,
            _ => (),
        };
    }

    pub fn can_move(&self, dx: i32, dy: i32) -> bool {
        let new_loc = Point::new(self.player.x + dx, self.player.y + dy);
        match self.terrain.get(&new_loc).unwrap() {
            Terrain::ClosedDoor => false,
            Terrain::DeepWater => false,
            Terrain::ShallowWater => true,
            Terrain::Wall => false,
            Terrain::Ground => true,
        }
    }
}
