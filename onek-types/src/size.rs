use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

/// Encapsulates a width and height. Similar to [`Point`] but with different semantics.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

impl Size {
    pub fn new(width: i32, height: i32) -> Size {
        Size { width, height }
    }

    pub fn zero() -> Size {
        Size { width: 0, height: 0 }
    }

    pub fn area(self) -> i32 {
        self.width * self.height
    }
}

impl Ord for Size {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.area().cmp(&rhs.area())
    }
}

impl PartialOrd for Size {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.width, self.height)
    }
}
