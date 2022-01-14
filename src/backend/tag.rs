use super::Color;
use derive_more::Display;
use std::fmt::{self, Formatter};

/// Affects behavior of items like burning oil or a pick axe. Also affects
/// spell behavior and whether characters can move through terrain.
#[allow(dead_code)]
#[derive(Clone, Copy, Display, Eq, PartialEq)]
pub enum Material {
    Wood,
    Stone,
    Metal,
}

/// Used with Tag::Liquid.
#[derive(Clone, Copy, Display, Eq, PartialEq)]
pub enum Liquid {
    Water,
    Vitr,
}

/// Object state and properties consist of a list of these tags. Objects can
/// be classified as Terrain, Weapon, etc but note that this is a fuzzy
/// concept because those classes can be combined.
#[derive(Clone, Eq, PartialEq)]
pub enum Tag {
    /// Player, monsters, special entities. Triggers an interaction when
    /// players try to move into them. Will also have a Name tag. XXX mention optional tags
    Character,
    /// Will also have a Character tag.
    Player,

    /// Description will have the sign's message.
    Sign,

    /// Normally also has a terrain tag.
    /// Will have Durability (and usually Material) if the door can be broken down.
    /// If it has a Binding tag then it can only be opened by characters that
    /// have a matching Binding object in their inventory (i.e. a key).
    ClosedDoor,
    /// Each level will have one of these. Will also have the Character tag.
    /// Grass, dirt, etc. Will have a Terrain tag,
    Ground,
    /// Water, lava, vitr etc. Will have a Terrain tag,
    Liquid { liquid: Liquid, deep: bool },
    /// Normally also has a terrain tag. This will also share tags with
    /// ClosedDoor so that they can be preserved as doors transition from
    /// open to closed.
    OpenDoor,
    /// Used for objects that are the lowest layer in a Cell, e.g. grassy ground.
    /// Note that this can be used for unusual objects such as a ballista. Will
    /// have a Background tag.
    Terrain,
    /// Will have a terrain tag. TODO: may want Material and Durability but
    /// burnt trees should probably remain impassible
    Tree,
    /// Will have a terrain tag and normally Durability and Material tags.
    /// At zero durability the wall is broken through.
    Wall,

    /// Normally only used with Terrain.
    Background(Color),
    /// Typically at zero durability an object will change somehow, e.g. a
    /// door will become open or a character will die.
    Durability { current: i32, max: i32 },
    /// Used for some terrain objects, e.g. walls and doors.
    Material(Material),
    /// Characters and portable objects all have names.
    Name(String),
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Tag::Character => write!(f, "Character"),
            Tag::Player => write!(f, "Player"),
            Tag::Sign => write!(f, "Sign"),
            Tag::ClosedDoor => write!(f, "ClosedDoor"),
            Tag::Ground => write!(f, "Ground"),
            Tag::Liquid { liquid, deep } => write!(f, "Liquid({liquid}, {deep}"),
            Tag::OpenDoor => write!(f, "OpenDoor"),
            Tag::Terrain => write!(f, "Terrain"),
            Tag::Tree => write!(f, "Tree"),
            Tag::Wall => write!(f, "Wall"),
            Tag::Background(color) => write!(f, "Background({color})"),
            Tag::Durability { current, max } => write!(f, "Durability({current}, {max})"),
            Tag::Material(material) => write!(f, "Material({material})"),
            Tag::Name(text) => write!(f, "Name({text})"),
        }
    }
}
