use super::event::Event;
use super::point::Point;
use super::EventPosted;
use std::collections::HashMap;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Terrain {
    DeepWater,
    Ground,
    ShallowWater,
    Wall,
}

pub struct Level {
    pub width: i32,
    pub height: i32,
    pub player: Point,
    pub terrain: HashMap<Point, Terrain>, // TODO: use FnvHashMap? or does a hash map really help? could use Vec2d
}

impl Level {
    pub fn new() -> Level {
        Level {
            width: 0,
            height: 0,
            player: Point::origin(),
            terrain: HashMap::new(),
        }
    }

    pub fn can_move(&self, dx: i32, dy: i32) -> bool {
        let new_loc = Point::new(self.player.x + dx, self.player.y + dy);
        match self.terrain.get(&new_loc).unwrap() {
            Terrain::DeepWater => false,
            Terrain::ShallowWater => true,
            Terrain::Wall => false,
            Terrain::Ground => true,
        }
    }
}

impl EventPosted for Level {
    fn posted(&mut self, event: Event) {
        match event {
            Event::NewLevel { width, height } => {
                self.width = width;
                self.height = height;
                self.player = Point::new(20, 10); // TODO: need to do better here
                self.terrain = HashMap::new();
            }
            Event::SetTerrain(loc, terrain) => {
                self.terrain.insert(loc, terrain);
            }
            Event::PlayerMoved(loc) => self.player = loc,
            _ => (),
        };
    }
}
