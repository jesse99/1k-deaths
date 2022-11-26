use super::*;
use arraystring::{typenum::U16, ArrayString};
use serde::{Deserialize, Serialize};
// use std::borrow::Cow;
use std::fmt;
use std::hash::{Hash, Hasher};

pub type TagStr = ArrayString<U16>;

/// Used to uniquely identify most objects in the [`Store`] via [`ObjectId`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Counter {
    // Used by Display so that we get more informative logging.
    #[cfg(debug_assertions)]
    pub tag: TagStr,

    // Unique identifier for an ObjectId::Obj.
    pub value: u32,
}

// TODO: instead of type could have relations like Armor, Weapon, Unique, etc
// TODO: UI could use these to figure out how to render

/// Used with the [`Store`] to identify a set of associated [`Relation`s].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ObjectId {
    /// These will have Objects and Terrain relations.
    Cell(Point),

    /// Used internally when a Cell is not found. Acts like a normal cell except that it's
    /// an error to try and mutate it.
    DefaultCell,

    /// Relations associated with the game as a whole, e.g. Messages.
    Game,

    /// NPC, portable item, trap, etc. Note that the behavior of objects is defined by
    /// their relations so we don't attempt to distinguish them via their ObjectId.
    Obj(Counter),

    /// The [`Store`] will always have one and only one of these. It will have an Objects
    /// relation for the player's inventory.
    Player,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Character {
    Guard,
    Player,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
    Wall,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum MessageKind {
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
pub struct Message {
    pub kind: MessageKind,
    pub text: String, // TODO: intern these? probably quite a few duplicates
}

/// Used to identify a particular Relation for operations like Store::find.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RelationTag {
    // TODO: need to generate RelationTag and Relation
    Character,
    Location,
    Messages,
    Objects,
    Terrain,
}

/// Used to associate a value with an [`ObjectId`] in the [`Store`].
#[derive(Debug, Serialize, Deserialize)]
pub enum Relation {
    /// Cells may have one of these.
    Character(Character),

    /// Used for characters and items.
    Location(Point),

    /// Messages to be shown to the player as the game is played.
    Messages(Vec<Message>),

    /// Characters and items for a Cell. Also items in a character's inventory.
    Objects(Vec<ObjectId>),

    /// A Cell will always have this.
    Terrain(Terrain),
}
// TODO: also will need Material and Durability

// TODO: generate this
impl Relation {
    // Would be nicer to use the From trait but that consumes the input.
    pub fn tag(&self) -> RelationTag {
        match self {
            Relation::Character(_) => RelationTag::Character,
            Relation::Location(_) => RelationTag::Location,
            Relation::Messages(_) => RelationTag::Messages,
            Relation::Objects(_) => RelationTag::Objects,
            Relation::Terrain(_) => RelationTag::Terrain,
        }
    }
}

#[cfg(debug_assertions)]
impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ObjectId::Obj(count) => write!(f, "{}:{}", count.tag, count.value),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[cfg(not(debug_assertions))]
impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ObjectId::Obj(count) => write!(f, "Obj:{}", count.value),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl fmt::Display for Terrain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for Character {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for RelationTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// We're using custom hashers so that old entries in the store will hash the same even
// as new case variants are added. This minimizes changes to insta snapshots as they
// are added (but serialization will break unless the new variants are added to the end).
impl Hash for ObjectId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ObjectId::Cell(loc) => {
                1000.hash(state);
                loc.hash(state);
            }
            ObjectId::DefaultCell => 1.hash(state),
            ObjectId::Game => 2.hash(state),
            ObjectId::Obj(counter) => {
                5000.hash(state);
                counter.value.hash(state)
            }
            ObjectId::Player => 3.hash(state),
        }
    }
}

impl Hash for RelationTag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            RelationTag::Character => 5.hash(state),
            RelationTag::Location => 1.hash(state),
            RelationTag::Messages => 2.hash(state),
            RelationTag::Objects => 3.hash(state),
            RelationTag::Terrain => 4.hash(state),
        }
    }
}
