use std::collections::HashMap;
use std::fmt;

enum Terrain {
    DeepWater,
    Ground,
    ShallowWater,
    Wall,
}

/// Location within a level.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    /// top-left
    pub fn origin() -> Point {
        Point { x: 0, y: 0 }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

pub struct Level {
    pub width: i32,
    pub height: i32,
    pub player: Point,
    terrain: HashMap<Point, Terrain>, // TODO: use FnvHashMap?
}

impl Level {
    fn new() -> Level {
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
}

fn render(level: &Level) {
    for y in 0..level.height {
        let mut line = String::new();
        for x in 0..level.width {
            let pt = Point::new(x, y);
            line += if pt == level.player {
                "@"
            } else {
                match level.terrain.get(&pt).unwrap() {
                    Terrain::DeepWater => "W",
                    Terrain::ShallowWater => "w",
                    Terrain::Wall => "#",
                    Terrain::Ground => ".",
                }
            }
        }
        println!("{}", line);
    }
}

// TODO:
// use some sort of curses library to render the map
// start handling user input
//    q should exit
//    arrow keys should move the player around
//    maybe disallow moving into walls and deep water
fn main() {
    let level = Level::new();
    render(&level);
}
