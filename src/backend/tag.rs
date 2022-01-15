use super::{Color, Object};
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
    /// players try to move into them. Will also have a Name tag. May have
    /// an Inventory tag.
    Character,
    /// Will also have Character and Inventory tags.
    Player,
    /// Objects that a Character has picked up.
    Inventory(Vec<Object>),

    /// The object is something that can be picked up and placed into a
    /// Character's inventory.
    Portable,
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

#[allow(dead_code)]
impl Tag {
    pub fn is_character(&self) -> bool {
        matches!(self, Tag::Character)
    }

    pub fn is_player(&self) -> bool {
        matches!(self, Tag::Player)
    }
    pub fn is_inventory(&self) -> bool {
        matches!(self, Tag::Inventory(_))
    }
    pub fn is_portable(&self) -> bool {
        matches!(self, Tag::Portable)
    }
    pub fn is_sign(&self) -> bool {
        matches!(self, Tag::Sign)
    }
    pub fn is_closed_door(&self) -> bool {
        matches!(self, Tag::ClosedDoor)
    }
    pub fn is_ground(&self) -> bool {
        matches!(self, Tag::Ground)
    }
    pub fn is_liquid(&self) -> bool {
        matches!(self, Tag::Liquid { liquid: _, deep: _ })
    }
    pub fn is_open_door(&self) -> bool {
        matches!(self, Tag::OpenDoor)
    }
    pub fn is_terrain(&self) -> bool {
        matches!(self, Tag::Terrain)
    }
    pub fn is_tree(&self) -> bool {
        matches!(self, Tag::Tree)
    }
    pub fn is_wall(&self) -> bool {
        matches!(self, Tag::Wall)
    }
    pub fn is_background(&self) -> bool {
        matches!(self, Tag::Background(_))
    }
    pub fn is_durability(&self) -> bool {
        matches!(self, Tag::Durability { current: _, max: _ })
    }
    pub fn is_material(&self) -> bool {
        matches!(self, Tag::Material(_))
    }
    pub fn is_name(&self) -> bool {
        matches!(self, Tag::Name(_))
    }

    pub fn as_inventory(&self) -> Option<&Vec<Object>> {
        match self {
            Tag::Inventory(result) => Some(result),
            _ => None,
        }
    }
    pub fn as_mut_inventory(&mut self) -> Option<&mut Vec<Object>> {
        match self {
            Tag::Inventory(result) => Some(result),
            _ => None,
        }
    }
    pub fn as_liquid(&self) -> Option<(Liquid, bool)> {
        match *self {
            Tag::Liquid { liquid, deep } => Some((liquid, deep)),
            _ => None,
        }
    }
    pub fn as_background(&self) -> Option<Color> {
        match *self {
            Tag::Background(result) => Some(result),
            _ => None,
        }
    }
    pub fn as_durability(&self) -> Option<(i32, i32)> {
        match *self {
            Tag::Durability { current, max } => Some((current, max)),
            _ => None,
        }
    }
    pub fn as_material(&self) -> Option<Material> {
        match *self {
            Tag::Material(result) => Some(result),
            _ => None,
        }
    }
    pub fn as_name(&self) -> Option<&String> {
        match self {
            Tag::Name(result) => Some(result),
            _ => None,
        }
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Tag::Character => write!(f, "Character"),
            Tag::Player => write!(f, "Player"),
            Tag::Inventory(_) => write!(f, "Inventory"),
            Tag::Portable => write!(f, "Portable"),
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
