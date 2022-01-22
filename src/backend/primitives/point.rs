use std::fmt::{self, Formatter};

/// Represents a point in cartesian space, typically a location within a level.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
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

    /// distance squared between two points
    pub fn distance2(&self, rhs: &Point) -> i32 {
        let dx = self.x - rhs.x;
        let dy = self.y - rhs.y;
        dx * dx + dy * dy
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
