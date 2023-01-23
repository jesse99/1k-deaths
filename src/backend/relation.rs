use super::*;
use arraystring::{typenum::U16, ArrayString};
use serde::{Deserialize, Serialize};
// use std::borrow::Cow;
use std::fmt;
use std::hash::{Hash, Hasher};

pub type TagStr3 = ArrayString<U16>;

/// Used to uniquely identify most objects in the [`Store`] via [`ObjectId3`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Counter3 {
    // Used by Display so that we get more informative logging.
    #[cfg(debug_assertions)]
    pub tag: TagStr3,

    // Unique identifier for an ObjectId3::Obj.
    pub value: u32,
}

// TODO: instead of type could have relations like Armor, Weapon, Unique, etc
// TODO: UI could use these to figure out how to render

/// Used with the [`Store`] to identify a set of associated [`Relation3`s].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ObjectId3 {
    /// These will have Objects and Terrain3 relations.
    Cell(Point),

    /// Used internally when a Cell is not found. Acts like a normal cell except that it's
    /// an error to try and mutate it.
    DefaultCell,

    /// Relations associated with the game as a whole, e.g. Messages.
    Game,

    /// NPC, portable item, trap, etc. Note that the behavior of objects is defined by
    /// their relations so we don't attempt to distinguish them via their ObjectId3.
    Obj(Counter3),

    /// The [`Store`] will always have one and only one of these. It will have an Objects
    /// relation for the player's inventory.
    Player,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Character3 {
    Guard,
    Player,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Terrain3 {
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
    Wall,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Portable {
    MightySword,
    WeakSword,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum MessageKind3 {
    /// Player is near death, special message when entering a new level, etc.
    Critical,

    // Player took a critical hit, buff is wearing off, etc.
    Important,

    // Relatively spammy messages, e.g. player was hit.
    Normal,

    // Messages that are not normally shown.
    Debug,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message3 {
    pub kind: MessageKind3,
    pub text: String, // TODO: intern these? probably quite a few duplicates
}

/// Used to identify a particular Relation3 for operations like Store::find.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RelationTag3 {
    // TODO: need to generate RelationTag3 and Relation3
    Character3,
    Location,
    Messages,
    Objects,
    Portable,
    Terrain3,
}

/// Used to associate a value with an [`ObjectId3`] in the [`Store`].
#[derive(Debug, Serialize, Deserialize)]
pub enum Relation3 {
    /// Characters will have this.
    Character3(Character3),

    /// Used for characters and items.
    Location(Point),

    /// Messages to be shown to the player as the game is played.
    Messages(Vec<Message3>),

    /// Characters and items for a Cell. Also items in a character's inventory.
    Objects(Vec<ObjectId3>),

    /// Objects that can be picked up and placed in a characters inventory.
    Portable(Portable),

    /// A Cell will always have this.
    Terrain3(Terrain3),
}
// TODO: also will need Material and Durability

// TODO: generate this
// impl Relation3 {
//     // Would be nicer to use the From trait but that consumes the input.
//     pub fn tag(&self) -> RelationTag3 {
//         match self {
//             Relation3::Character3(_) => RelationTag3::Character3,
//             Relation3::Location(_) => RelationTag3::Location,
//             Relation3::Messages(_) => RelationTag3::Messages,
//             Relation3::Objects(_) => RelationTag3::Objects,
//             Relation3::Portable(_) => RelationTag3::Portable,
//             Relation3::Terrain3(_) => RelationTag3::Terrain3,
//         }
//     }
// }

#[cfg(debug_assertions)]
impl fmt::Display for ObjectId3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ObjectId3::Obj(count) => write!(f, "{}:{}", count.tag, count.value),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[cfg(not(debug_assertions))]
impl fmt::Display for ObjectId3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ObjectId3::Obj(count) => write!(f, "Obj:{}", count.value),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl fmt::Display for Terrain3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for Character3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for RelationTag3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// We're using custom hashers so that old entries in the store will hash the same even
// as new case variants are added. This minimizes changes to insta snapshots as they
// are added (but serialization will break unless the new variants are added to the end).
impl Hash for ObjectId3 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ObjectId3::Cell(loc) => {
                1000.hash(state);
                loc.hash(state);
            }
            ObjectId3::DefaultCell => 1.hash(state),
            ObjectId3::Game => 2.hash(state),
            ObjectId3::Obj(counter) => {
                5000.hash(state);
                counter.value.hash(state)
            }
            ObjectId3::Player => 3.hash(state),
        }
    }
}

impl Hash for RelationTag3 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            RelationTag3::Character3 => 5.hash(state),
            RelationTag3::Location => 1.hash(state),
            RelationTag3::Messages => 2.hash(state),
            RelationTag3::Objects => 3.hash(state),
            RelationTag3::Portable => 6.hash(state),
            RelationTag3::Terrain3 => 4.hash(state),
        }
    }
}
