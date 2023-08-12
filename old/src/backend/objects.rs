/// These are the non-primitive types that go into the [`Store`].
use arraystring::{typenum::U16, ArrayString};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;

pub type TagStr = ArrayString<U16>;

/// Used to uniquely identify objects in the [`Store`]. Oids are typically created with
/// the various Level create methods.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Oid {
    // Used by Display so that we get more informative logging.
    #[cfg(debug_assertions)]
    pub tag: Option<TagStr>, // Option to allow us to use stuff like PLAYER_ID, annoying but it is debug only and just for Display

    pub value: u32,
}

/// The player and NPCs will each have a unique Oid. That oid will store the following:
/// * A Character value.
/// * A location Point.
/// * A list of inventory Oid's (may be empty).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum Character {
    #[default]
    Guard,
    Player,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Durability {
    pub current: i32,
    pub max: i32,
}

/// Objects that may be picked up and dropped off. These store:
/// * A Portable value.
/// * An InvItem value (if they can be equipped).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum Portable {
    #[default]
    MightySword,
    WeakSword,
}

/// See [`Portable`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InvItem {
    // TODO: replace this with Slot?
    // pub slot: Option<Slot>, // None if not equipped
    pub oid: Oid,
}

/// Each cell on the level will have a unique Oid which looks like "(1, 2)". Levels may be
/// irregular (and may grow as the result of actions like digging). Cell Oids store the
/// following:
/// * A Terrain value.
/// * A location Point. Note that levels can be irregular.
/// * A list of object Oids. These are typically Portable objects with an optional
/// Character at the end.
/// Note that cells that are not explicitly part of the level are managed internally by
/// Level using DEFAULT_CELL_ID.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum Terrain {
    /// Will have Durability (and usually Material) if the door can be broken down.
    /// If it has a Binding tag then it can only be opened by characters that
    /// have a matching Binding object in their inventory (i.e. a key).
    ClosedDoor,

    DeepWater,
    Dirt,
    OpenDoor,

    /// Will have a Material tag.
    Rubble,

    ShallowWater,

    /// TODO: may want Material and Durability but burnt trees should probably remain impassible
    Tree,

    Vitr,

    /// Will normally have Durability and Material tags. At zero durability changes to Rubble.
    #[default]
    Wall,
}

/// The GAME_ID Oid stores a list of these.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Message {
    pub kind: MessageKind,
    pub text: String, // TODO: intern these? probably quite a few duplicates
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum MessageKind {
    /// Operation failed.
    Error,

    /// Player is near death, special message when entering a new level, etc.
    Critical,

    // Player took a critical hit, buff is wearing off, etc.
    Important,

    // Relatively spammy messages, e.g. player was hit.
    #[default]
    Normal,

    // Messages that are not normally shown.
    Debug,
}

impl Oid {
    #[cfg(debug_assertions)]
    pub fn new(tag: &str, value: u32) -> Oid {
        Oid {
            tag: Some(TagStr::from_str_truncate(tag)),
            value: value,
        }
    }

    #[cfg(not(debug_assertions))]
    pub fn new(_tag: &str, value: u32) -> Oid {
        Oid { value: value }
    }

    #[cfg(debug_assertions)]
    pub const fn without_tag(value: u32) -> Oid {
        Oid {
            tag: None,
            value: value,
        }
    }

    #[cfg(not(debug_assertions))]
    pub const fn without_tag(value: u32) -> Oid {
        Oid { value: value }
    }
}

mod display_impl {
    use super::*;

    impl Display for Message {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{:?}: {}", self.kind, self.text)
        }
    }

    impl Display for Character {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    impl Display for Durability {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    impl Display for Portable {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    impl Display for Terrain {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    impl Display for InvItem {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    impl fmt::Display for Oid {
        #[cfg(debug_assertions)]
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            if let Some(tag) = self.tag {
                write!(f, "{}#{}", tag, self.value)
            } else {
                match self.value {
                    0 => write!(f, "player#{}", self.value),
                    1 => write!(f, "default cell#{}", self.value),
                    2 => write!(f, "game#{}", self.value),
                    _ => panic!("excpected a tag"),
                }
            }
        }

        #[cfg(not(debug_assertions))]
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "#{}", self.value)
        }
    }
}