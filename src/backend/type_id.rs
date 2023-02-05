use super::{Character, InvItem, Message, Oid, Point, Portable, Terrain};

/// Every type used as a VALUE in the [`Store`] must implement this to return a unique
/// numeric ID for that type. (This is checked at runtime for debug builds).
pub trait TypeId {
    fn id(&self) -> u16;
}

impl TypeId for Oid {
    fn id(&self) -> u16 {
        0
    }
}

impl TypeId for Character {
    fn id(&self) -> u16 {
        1
    }
}

impl TypeId for Point {
    fn id(&self) -> u16 {
        2
    }
}

impl TypeId for InvItem {
    fn id(&self) -> u16 {
        3
    }
}

impl TypeId for Terrain {
    fn id(&self) -> u16 {
        4
    }
}

impl TypeId for Portable {
    fn id(&self) -> u16 {
        5
    }
}

impl TypeId for Message {
    fn id(&self) -> u16 {
        6
    }
}
