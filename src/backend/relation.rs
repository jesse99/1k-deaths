use super::*;
use std::fmt;

/// Used to uniquely identify most objects in the [`Store`] via [`ObjectId`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct Counter {
    // Used by Display so that we get more informative logging.
    #[cfg(debug_assertions)]
    pub tag: &'static str,

    // Unique identifier for an ObjectId::Obj.
    pub value: u32,
}

// TODO: instead of type could have relations like Armor, Weapon, Unique, etc
// TODO: UI could use these to figure out how to render

/// Used with the [`Store`] to identify a set of associated [`Relation`s].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum ObjectId {
    /// These will have Background, Objects, and Terrain relations.
    Cell(Point),

    /// Used internally when a Cell is not found. Acts like a normal cell except that it's
    /// an error to try and mutate it.
    DefaultCell,

    /// NPC, portable item, trap, etc. Note that the behavior of objects is defined by
    /// their relations so we don't attempt to distinguish them via their ObjectId.
    Obj(Counter),

    /// The [`Store`] will always have one and only one of these. It will have an Objects
    /// relation for the player's inventory.
    Player,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Terrain {
    /// Will have Durability (and usually Material) if the door can be broken down.
    /// If it has a Binding tag then it can only be opened by characters that
    /// have a matching Binding object in their inventory (i.e. a key).
    ClosedDoor,

    DeepWater,

    /// Grass, dirt, etc.
    Ground,

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

/// Used to identify a particular Relation for operations like Store::find.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum RelationTag {
    // TODO: need to generate RelationTag and Relation
    Background,
    Location,
    Objects,
    Terrain,
}

/// Used to associate a value with an [`ObjectId`] in the [`Store`].
pub(super) enum Relation {
    /// Typically just used for Cell.
    Background(Color),

    /// Used for characters and items.
    Location(Point),

    /// Characters and items for a Cell. Also items in a character's inventory.
    Objects(Vec<ObjectId>),

    /// A Cell will always have this.
    Terrain(Terrain),
}
// TODO: also will need Material and Durability

// TODO: generate this
impl Relation {
    // Would be nicer to use the From trait but that consumes the input.
    pub(super) fn tag(&self) -> RelationTag {
        match self {
            Relation::Background(_) => RelationTag::Background,
            Relation::Location(_) => RelationTag::Location,
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

impl fmt::Display for RelationTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
