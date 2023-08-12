use serde::{Deserialize, Serialize};
use std::fmt::{self, Formatter};
use std::hash::{Hash, Hasher};

/// Represents a point in cartesian space, typically a location within a level.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
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

    // pub fn adjacent(&self, rhs: Point) -> bool {
    //     let dx = (self.x - rhs.x).abs();
    //     let dy = (self.y - rhs.y).abs();
    //     dx <= 1 && dy <= 1 && !(dx == 0 && dy == 0)
    // }

    /// distance squared between two points
    pub fn distance2(&self, rhs: Point) -> i32 {
        let dx = self.x - rhs.x;
        let dy = self.y - rhs.y;
        dx * dx + dy * dy
    }

    #[cfg(test)]
    pub fn diagnol(&self, new_loc: Point) -> bool {
        assert!(*self != new_loc);

        let dx = self.x - new_loc.x;
        let dy = self.y - new_loc.y;
        assert!(dx >= -1 && dx <= 1);
        assert!(dy >= -1 && dy <= 1);

        dx != 0 && dy != 0
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Hash for Point {
    // This should be quite a bit better than simply folding x onto y.
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut s = self.x as i64;
        s <<= 32;
        s |= self.y as i64;
        s.hash(state);
    }
}
