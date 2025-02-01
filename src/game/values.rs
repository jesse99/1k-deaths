/// These are the non-primitive types that go into the [`Store`].
use serde::{Deserialize, Serialize};
use std::fmt;

// /// The player and NPCs will each have a unique Oid. That oid will store the following:
// /// * A Character value.
// /// * A location Point.
// /// * A list of inventory Oid's (may be empty).
// #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
// pub enum Character {
//     #[default]
//     Guard,
//     Player,
// }

// #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
// pub struct Durability {
//     pub current: i32,
//     pub max: i32,
// }

// /// Objects that may be picked up and dropped off. These store:
// /// * A Portable value.
// /// * An InvItem value (if they can be equipped).
// #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
// pub enum Portable {
//     #[default]
//     MightySword,
//     WeakSword,
// }

// /// See [`Portable`].
// #[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
// pub struct InvItem {
//     // TODO: replace this with Slot?
//     // pub slot: Option<Slot>, // None if not equipped
//     pub oid: Oid,
// }

// /// Each cell on the level will have a unique Oid which displays like "(1, 2)". Levels may be
// /// irregular (and may grow as the result of actions like digging). Cell Oids store the
// /// following:
// /// * A Terrain value.
// /// * A location Point. Note that levels can be irregular.
// /// * A list of object Oids. These are typically Portable objects with an optional
// /// Character at the end.
// /// Note that cells that are not explicitly part of the level are managed internally by
// /// Level using DEFAULT_CELL_ID.
// #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
// pub enum Terrain {
//     /// Will have Durability (and usually Material) if the door can be broken down.
//     /// If it has a Binding tag then it can only be opened by characters that
//     /// have a matching Binding object in their inventory (i.e. a key).
//     ClosedDoor,

//     DeepWater,
//     Dirt,
//     OpenDoor,

//     /// Will have a Material tag.
//     Rubble,

//     ShallowWater,

//     /// TODO: may want Material and Durability but burnt trees should probably remain impassible
//     Tree,

//     Vitr,

//     /// Will normally have Durability and Material tags. At zero durability changes to Rubble.
//     #[default]
//     Wall,
// }

mod display_impl {
    use super::*;

    // impl Display for Character {
    //     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    //         write!(f, "{:?}", self)
    //     }
    // }

    // impl Display for Durability {
    //     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    //         write!(f, "{:?}", self)
    //     }
    // }

    // impl Display for Portable {
    //     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    //         write!(f, "{:?}", self)
    //     }
    // }

    // impl Display for InvItem {
    //     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    //         write!(f, "{:?}", self)
    //     }
    // }
}
