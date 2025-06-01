use super::{Character, PassiveTime};
use crate::shared::*;

/// Every type used as a VALUE in the [`Store`] must implement this to return a unique
/// numeric ID for that type. (This is checked at runtime for debug builds).
pub trait TypeId<T> {
    const ID: u16;
}

impl<T> TypeId<T> for Character {
    const ID: u16 = 1;
}

impl<T> TypeId<T> for PassiveTime {
    const ID: u16 = 2;
}

impl<T> TypeId<T> for Point {
    const ID: u16 = 3;
}

impl<T> TypeId<T> for Species {
    const ID: u16 = 4;
}

impl<T> TypeId<T> for Terrain {
    const ID: u16 = 5;
}
