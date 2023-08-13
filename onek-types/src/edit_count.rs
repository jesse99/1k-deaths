use serde::{Deserialize, Serialize};
use std::fmt::{self, Formatter};

/// Monotonically increasing number that is incremented each time state changes.
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct EditCount {
    count: u64,
}

impl EditCount {
    pub fn intial() -> EditCount {
        EditCount { count: 0 }
    }

    pub fn increment(&self) -> EditCount {
        EditCount { count: self.count + 1 }
    }
}

impl fmt::Display for EditCount {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.count)
    }
}
