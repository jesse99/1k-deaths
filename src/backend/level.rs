use super::point::Point;
use std::collections::HashMap;

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
    pub terrain: HashMap<Point, Terrain>, // TODO: use FnvHashMap?
}

impl Level {
    pub fn new() -> Level {
        let width = 100;
        let height = 30;
        let player = Point::new(20, 10);
        let mut terrain = HashMap::new();

        // Terrain defaults to ground
        for y in 0..height {
            for x in 0..width {
                terrain.insert(Point::new(x, y), Terrain::Ground);
            }
        }

        // Walls along the edges
        for y in 0..height {
            terrain.insert(Point::new(0, y), Terrain::Wall);
            terrain.insert(Point::new(width - 1, y), Terrain::Wall);
        }
        for x in 0..width {
            terrain.insert(Point::new(x, 0), Terrain::Wall);
            terrain.insert(Point::new(x, height - 1), Terrain::Wall);
        }

        // Small lake
        terrain.insert(Point::new(29, 20), Terrain::DeepWater);
        terrain.insert(Point::new(30, 20), Terrain::DeepWater); // lake center
        terrain.insert(Point::new(31, 20), Terrain::DeepWater);
        terrain.insert(Point::new(30, 19), Terrain::DeepWater);
        terrain.insert(Point::new(30, 21), Terrain::DeepWater);

        terrain.insert(Point::new(29, 19), Terrain::ShallowWater);
        terrain.insert(Point::new(31, 19), Terrain::ShallowWater);
        terrain.insert(Point::new(29, 21), Terrain::ShallowWater);
        terrain.insert(Point::new(31, 21), Terrain::ShallowWater);

        terrain.insert(Point::new(28, 20), Terrain::ShallowWater);
        terrain.insert(Point::new(32, 20), Terrain::ShallowWater);
        terrain.insert(Point::new(30, 18), Terrain::ShallowWater);
        terrain.insert(Point::new(30, 22), Terrain::ShallowWater);

        Level {
            width,
            height,
            player,
            terrain,
        }
    }

    pub fn handle_move_player(&mut self, dx: i32, dy: i32) {
        if self.can_move(dx, dy) {
            self.move_player(dx, dy)
        }
    }

    fn move_player(&mut self, dx: i32, dy: i32) {
        self.player = Point::new(self.player.x + dx, self.player.y + dy);
    }
    fn can_move(&self, dx: i32, dy: i32) -> bool {
        let new_loc = Point::new(self.player.x + dx, self.player.y + dy);
        match self.terrain.get(&new_loc).unwrap() {
            Terrain::DeepWater => false,
            Terrain::ShallowWater => true,
            Terrain::Wall => false,
            Terrain::Ground => true,
        }
    }
}
