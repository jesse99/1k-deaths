use super::*;
use arraystring::{typenum::U16, ArrayString};
use fnv::FnvHashMap;
// use once_cell::sync::OnceCell;
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum Character {
    #[default]
    Guard,
    Player,
}

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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InvItem {
    // pub slot: Option<Slot>, // None if not equipped
    pub oid: Oid,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum Portable {
    #[default]
    MightySword,
    WeakSword,
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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Message {
    pub kind: MessageKind,
    pub text: String, // TODO: intern these? probably quite a few duplicates
}

// TODO: put these into a sub-module?
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

impl Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for Character {
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

pub static PLAYER_ID: Oid = Oid::without_tag(0);
pub static DEFAULT_CELL_ID: Oid = Oid::without_tag(1);
pub static GAME_ID: Oid = Oid::without_tag(2);
pub static LAST_ID: u32 = 2;

/// Wrapper around a [`Store`] for cells within a level. These need some special casing
/// because the mapping from a location to an Oid has to be consistent and because level
/// sizes are not static (levels have a default cell which is typically rock and can be
/// dug into to create a new normal cell).
pub struct Level {
    pub store: Store<Oid>,
    cell_ids: FnvHashMap<Point, Oid>,
    current_id: u32,
}

impl Level {
    pub fn new() -> Level {
        let mut level = Level {
            store: Store::new(),
            cell_ids: FnvHashMap::default(),
            current_id: LAST_ID,
        };
        level.store.create(DEFAULT_CELL_ID, Terrain::Wall);
        level
    }

    pub fn create_player(&mut self, loc: Point) {
        let oid = self.get_or_create_cell(loc);
        self.store.create(PLAYER_ID, Character::Player);
        self.store.create(PLAYER_ID, loc);
        // also has an empty list of InvItem

        self.store.append(oid, PLAYER_ID);
    }

    pub fn find_char(&self, loc: Point) -> Option<Character> {
        if let Some(oid) = self.find_cell(loc) {
            self.store.find::<Character>(oid)
        } else {
            None
        }
    }

    pub fn create_terrain(&mut self, loc: Point, terrain: Terrain) {
        let oid = self.get_or_create_cell(loc);
        self.store.create(oid, terrain);
        self.store.create(oid, loc);
    }

    /// If the loc was never created then this will extend the level by creating a new
    /// cell.
    pub fn set_terrain(&mut self, loc: Point, terrain: Terrain) {
        if let Some(oid) = self.find_cell(loc) {
            let replaced = self.store.replace(oid, terrain);
            assert!(replaced);
        } else {
            let oid = self.get_or_create_cell(loc);
            self.store.create(oid, terrain);
            self.store.create(oid, loc);
        }
    }

    // TODO: If find_cell turns out to be a bottle-neck then could add a get_cell
    // function with methods like get_terrain, get_portables, and get_char.
    pub fn get_terrain(&self, loc: Point) -> Terrain {
        if let Some(oid) = self.find_cell(loc) {
            let terrain = self.store.find::<Terrain>(oid);
            terrain.expect(&format!("expected a terrain for {oid}"))
        } else {
            let terrain = self.store.find::<Terrain>(DEFAULT_CELL_ID);
            terrain.expect(&format!("expected a terrain for the default cell"))
        }
    }

    pub fn append_portable(&mut self, loc: Point, tag: &str, portable: Portable) {
        if let Some(oid) = self.find_cell(loc) {
            let obj_oid = self.new_oid(tag);
            self.store.create(obj_oid, portable);
            self.store.append(oid, obj_oid);
        } else {
            panic!("Can't add a portable to the default cell");
        }
    }

    pub fn get_portables(&self, loc: Point) -> Vec<Portable> {
        if let Some(oid) = self.find_cell(loc) {
            self.store.get_all::<Portable>(oid)
        } else {
            Vec::new()
        }
    }

    pub fn append_message(&mut self, message: Message) {
        const MAX_MESSAGES: usize = 1000;
        const EXTRA_MESSAGES: usize = 100; // trim messages once len is more than MAX + EXTRA

        self.store.append(GAME_ID, message);
        let len = self.store.len::<Message>(GAME_ID);
        if len >= MAX_MESSAGES + EXTRA_MESSAGES {
            self.store.remove_range::<Message>(GAME_ID, 0..(len - MAX_MESSAGES));
        }
    }

    /// Note that this has no checks for whether this is a legal move (e.g. moving the
    /// player into a stone wall is normally not allowed).
    pub fn move_char(&mut self, oid: Oid, new_loc: Point) {
        if let Some(old_loc) = self.store.find::<Point>(oid) {
            let cell_id = self.cell_ids.get(&old_loc).unwrap();
            self.store.remove_value(*cell_id, oid);
        }

        self.store.replace(oid, new_loc);

        let cell_id = self.get_or_create_cell(new_loc);
        self.store.append(cell_id, oid);
    }

    // TODO: move_obj would be very similar to move_char except the we need to insert the obj_oid before characters

    pub fn expect_location(&self, oid: Oid) -> Point {
        let loc = self.store.find::<Point>(oid);
        loc.expect(&format!("expected a loc for {oid}"))
    }

    // Extent cells always have a terrain and a list of oids associated with that location.
    fn find_cell(&self, loc: Point) -> Option<Oid> {
        self.cell_ids.get(&loc).copied()
    }

    fn get_or_create_cell(&mut self, loc: Point) -> Oid {
        // Can't use the entry API because we need to create a new Oid on insert which
        // would require two mutable refs to seld. Should be OK though because the normal
        // case is to have a cell.
        if let Some(oid) = self.find_cell(loc) {
            oid
        } else {
            let oid = self.new_oid(&format!("{loc}"));
            self.cell_ids.insert(loc, oid);
            oid
        }
    }

    fn new_oid(&mut self, tag: &str) -> Oid {
        self.current_id += 1;
        Oid::new(tag, self.current_id)
    }
}
