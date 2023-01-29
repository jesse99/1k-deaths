use std::fmt;

/// Encapsulates a width and height. Similar to Point but with different semantics.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

impl Size {
    pub fn new(width: i32, height: i32) -> Size {
        Size { width, height }
    }

    // pub fn zero() -> Size {
    //     Size {
    //         width: 0,
    //         height: 0,
    //     }
    // }

    #[cfg(test)] // for now this is only used within unit tests
    pub fn area(self) -> i32 {
        self.width * self.height
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.width, self.height)
    }
}
