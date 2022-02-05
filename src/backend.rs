//! Contains the game logic, i.e. everything but rendering, user input, and program initialization.
mod event;
mod events;
mod interactions;
mod make;
mod message;
mod object;
mod old_pov;
mod persistence;
mod pov;
mod primitives;
mod tag;

pub use event::Event;
pub use message::{Message, Topic};
pub use primitives::Color;
pub use primitives::Point;
pub use primitives::Size;

use derive_more::Display;
use fnv::{FnvHashMap, FnvHashSet};
use interactions::Interactions;
use object::{Object, TagValue};
use old_pov::OldPoV;
use pov::PoV;
use rand::prelude::*;
use rand::rngs::SmallRng;
use rand::RngCore;
use std::cell::{RefCell, RefMut};
use std::fs::File;
use tag::*;
use tag::{Durability, Material, Tag};

const MAX_MESSAGES: usize = 1000;

// TODO: These numbers are not very intelligible. If that becomes an issue we could use
// a newtype string (e.g. "wall 97") or a simple struct with a static string ref and a
// counter.
#[derive(Clone, Copy, Debug, Display, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ObjId(u64);

#[derive(Clone, Copy, Debug)]
pub enum Command {
    /// Move the player to empty cells (or attempt to interact with an object at that cell).
    /// dx and dy must be 0, +1, or -1.
    Move { dx: i32, dy: i32 },
    /// Print descriptions for objects at the cell. Note that any cell can be examined but
    /// cells that are not in the player's PoV will have either an unhelpful description or
    /// a stale description.
    Examine(Point),
}

pub enum Tile {
    /// player can see this
    Visible { bg: Color, fg: Color, symbol: char },
    /// player can't see this but has in the past, note that this may not reflect the current state
    Stale(char),
    /// player has never seen this location (and it may not exist)
    NotVisible,
}

#[derive(Clone, Copy, Debug, Display, Eq, PartialEq, Serialize, Deserialize)]
pub enum State {
    Adventuring,
    KilledRhulad,
    WonGame,
}

/// Top-level backend object encapsulating the game state.
pub struct Game {
    stream: Vec<Event>, // used to reconstruct games
    file: Option<File>, // events are perodically saved here
    state: State,       // game milestones, eg won game
    posting: bool,      // prevents re-entrant posting events
    next_id: u64,       // 0 is the player
    rng: RefCell<SmallRng>,

    player: Point,
    default: Object, // object to use for a non-existent cell (can happen if a wall is destroyed)
    objects: FnvHashMap<ObjId, Object>, // all existing objects are here
    cells: FnvHashMap<Point, Vec<ObjId>>, // objects within each cell on the map
    constructing: bool, // level is in the process of being constructed

    messages: Vec<Message>,     // messages shown to the player
    interactions: Interactions, // double dispatch action tables, e.g. player vs door

    pov: PoV,         // locations that the player can currently see
    old_pov: OldPoV,  // locations that the user has seen in the past (this will often be stale data)
    invariants: bool, // if true then expensive checks are enabled
}

// Public API.
impl Game {
    /// Start a brand new game and save it to path.
    pub fn new_game(path: &str, seed: u64) -> Game {
        let mut messages = Vec::new();

        info!("new {path}");
        let file = match persistence::new_game(path) {
            Ok(se) => Some(se),
            Err(err) => {
                messages.push(Message::new(
                    Topic::Error,
                    &format!("Couldn't open {path} for writing: {err}"),
                ));
                None
            }
        };

        let mut events = Vec::new();
        events.reserve(1000); // TODO: probably should tune this

        events.push(Event::NewGame);
        events.push(Event::BeginConstructLevel);
        events.push(Event::AddMessage(Message {
            topic: Topic::Important,
            text: String::from("Welcome to 1k-deaths!"),
        }));
        events.push(Event::AddMessage(Message {
            topic: Topic::Important,
            text: String::from("Are you the hero will will destroy the Crippled God's sword?"),
        }));
        events.push(Event::AddMessage(Message {
            topic: Topic::Important,
            text: String::from("Press the '?' key for help."),
        }));

        // TODO: may want a SetAllTerrain variant to avoid a zillion events
        // TODO: or have NewLevel take a default terrain
        let mut game = Game::new(messages, seed, file);
        let map = include_str!("backend/maps/start.txt");
        make::level(&game, map, &mut events);
        events.push(Event::EndConstructLevel);

        game.post(events, false);
        game
    }

    /// Load a saved game and return the events so that they can be replayed.
    pub fn old_game(path: &str, seed: u64) -> (Game, Vec<Event>) {
        let mut events = Vec::new();

        let mut messages = Vec::new();
        let mut file = None;
        info!("loading {path}");
        match persistence::load_game(path) {
            Ok(e) => events = e,
            Err(err) => {
                info!("loading file had err: {err}");
                messages.push(Message::new(
                    Topic::Error,
                    &format!("Couldn't open {path} for reading: {err}"),
                ));
            }
        };

        if !events.is_empty() {
            info!("opening {path}");
            file = match persistence::open_game(path) {
                Ok(se) => Some(se),
                Err(err) => {
                    messages.push(Message::new(
                        Topic::Error,
                        &format!("Couldn't open {path} for appending: {err}"),
                    ));
                    None
                }
            };
        }

        if file.is_some() {
            (Game::new(messages, seed, file), events)
        } else {
            let mut game = Game::new_game(path, seed);

            events.clear();
            events.extend(messages.into_iter().map(Event::AddMessage));
            game.post(events, false);

            (game, Vec::new())
        }
    }

    pub fn recent_messages(&self, limit: usize) -> impl Iterator<Item = &Message> {
        let iter = self.messages.iter();
        if limit < self.messages.len() {
            iter.skip(self.messages.len() - limit)
        } else {
            iter.skip(0)
        }
    }

    pub fn events(&self) -> Vec<String> {
        self.stream.iter().map(|e| format!("{:?}", e)).collect()
    }

    pub fn player(&self) -> Point {
        self.player
    }

    pub fn command(&self, command: Command, events: &mut Vec<Event>) {
        // TODO: probably want to return something to indicate whether a UI refresh is neccesary
        // TODO: maybe something fine grained, like only need to update messages
        match command {
            Command::Move { dx, dy } => {
                assert!(dx >= -1 && dx <= 1);
                assert!(dy >= -1 && dy <= 1);
                assert!(dx != 0 || dy != 0); // TODO: should this be a short rest?
                let player = self.player;
                let new_loc = Point::new(player.x + dx, player.y + dy);
                if !self.interact_pre_move(&player, &new_loc, events) {
                    events.push(Event::PlayerMoved(new_loc));
                }
            }
            Command::Examine(new_loc) => {
                if self.pov.visible(&new_loc) {
                    let descs: Vec<String> = self
                        .cell_iter(&new_loc)
                        .map(|(_, obj)| obj.description().to_string())
                        .collect();
                    let descs = descs.join(", and ");
                    let text = format!("You see {descs}.");
                    events.push(Event::AddMessage(Message {
                        topic: Topic::Normal,
                        text,
                    }));
                } else if self.old_pov.get(&new_loc).is_some() {
                    let text = "You can no longer see there.".to_string();
                    events.push(Event::AddMessage(Message {
                        topic: Topic::Normal,
                        text,
                    }));
                } else {
                    let text = "You've never seen there.".to_string();
                    events.push(Event::AddMessage(Message {
                        topic: Topic::Normal,
                        text,
                    }));
                }
            }
        }
    }

    /// If loc is valid and within the player's Field if View (FoV) then return the terrain.
    /// Otherwise return None.
    pub fn tile(&self, loc: &Point) -> Tile {
        let tile = if self.pov.visible(loc) {
            let (_, obj) = self.get_bottom(loc);
            let bg = obj.to_bg_color();

            let (_, obj) = self.get_top(loc);
            let (fg, symbol) = obj.to_fg_symbol();

            Tile::Visible { bg, fg, symbol }
        } else {
            match self.old_pov.get(loc) {
                Some(symbol) => Tile::Stale(symbol),
                None => Tile::NotVisible, // not visible and never seen
            }
        };

        tile
    }

    pub fn target_next(&self, old_loc: &Point, delta: i32) -> Option<Point> {
        // Find the cells with Characters in the player's PoV.
        let mut chars = Vec::new();
        for loc in self.pov.locations() {
            if self.has(loc, CHARACTER_ID) {
                chars.push(*loc);
            }
        }

        // Sort those cells by distance from the player.
        let p = self.player();
        chars.sort_by_key(|a| a.distance2(&p));

        // Find the Character closest to old_loc.
        let mut index = 0;
        let mut dist = i32::MAX;
        for (i, loc) in chars.iter().enumerate() {
            let d = loc.distance2(old_loc);
            if d < dist {
                index = i;
                dist = d;
            }
        }

        // Find the next Character to examine accounting for lame unsized math.
        if delta > 0 {
            if index < chars.len() && chars[index] != *old_loc {
                // we don't want to apply delta in this case
                assert_eq!(delta, 1);
            } else if index + (delta as usize) < chars.len() {
                index += delta as usize;
            } else {
                index = 0;
            }
        } else if (-delta as usize) <= index {
            index -= -delta as usize;
        } else {
            index = chars.len() - 1;
        }

        if index < chars.len() {
            Some(chars[index])
        } else {
            // We'll only land here in the unusual case of the player not able to see himself.
            None
        }
    }

    // This does not affect game state at all so it's OK that it's mutable.
    pub fn set_invariants(&mut self, enable: bool) {
        self.invariants = enable;
    }
}

// Backend methods. Note that mutable methods should only be in the events module.
impl Game {
    fn new(messages: Vec<Message>, seed: u64, file: Option<File>) -> Game {
        info!("using seed {seed}");
        Game {
            stream: Vec::new(),
            file,
            state: State::Adventuring,
            posting: false,
            next_id: 2,

            // TODO:
            // 1) SmallRng is not guaranteed to be portable so results may
            // not be reproducible between platforms.
            // 2) We're going to have to be able to persist the RNG. rand_pcg
            // supports serde so that would likely work. If not we could
            // create our own simple RNG.
            rng: RefCell::new(SmallRng::seed_from_u64(seed)),

            player: Point::origin(),
            objects: FnvHashMap::default(),
            cells: FnvHashMap::default(),
            default: make::stone_wall(),
            constructing: true,

            messages,
            interactions: Interactions::new(),

            pov: PoV::new(),
            old_pov: OldPoV::new(),
            invariants: false,
        }
    }

    fn has(&self, loc: &Point, tag: TagId) -> bool {
        if let Some(oids) = self.cells.get(loc) {
            for oid in oids.iter() {
                let obj = self
                    .objects
                    .get(oid)
                    .expect("All objects in the level should still exist");
                if obj.has(tag) {
                    return true;
                }
            }
        }
        self.default.has(tag)
    }

    fn get(&self, loc: &Point, tag: TagId) -> Option<(ObjId, &Object)> {
        if let Some(oids) = self.cells.get(loc) {
            for oid in oids.iter().rev() {
                let obj = self
                    .objects
                    .get(oid)
                    .expect("All objects in the level should still exist");
                if obj.has(tag) {
                    return Some((*oid, obj));
                }
            }
        }
        if self.default.has(tag) {
            // Note that if this cell is converted into a real cell the oid will change.
            // I don't think that this will be a problem in practice...
            Some((ObjId(1), &self.default))
        } else {
            None
        }
    }

    /// Typically this will be a terrain object.
    fn get_bottom(&self, loc: &Point) -> (ObjId, &Object) {
        if let Some(oids) = self.cells.get(loc) {
            let oid = oids
                .first()
                .expect("cells should always have at least a terrain object");
            let obj = self
                .objects
                .get(oid)
                .expect("All objects in the level should still exist");
            (*oid, obj)
        } else {
            (ObjId(1), &self.default)
        }
    }

    /// Character, item, door, or if all else fails terrain.
    fn get_top(&self, loc: &Point) -> (ObjId, &Object) {
        if let Some(oids) = self.cells.get(loc) {
            let oid = oids.last().expect("cells should always have at least a terrain object");
            let obj = self
                .objects
                .get(oid)
                .expect("All objects in the level should still exist");
            (*oid, obj)
        } else {
            (ObjId(1), &self.default)
        }
    }

    /// Iterates over the objects at loc starting with the topmost object.
    fn cell_iter(&self, loc: &Point) -> impl Iterator<Item = (ObjId, &Object)> {
        CellIterator::new(self, loc)
    }

    fn player_inv_iter(&self) -> impl Iterator<Item = (ObjId, &Object)> {
        InventoryIterator::new(self, &self.player)
    }

    // The RNG doesn't directly affect the game state so we use interior mutability for it.
    fn rng(&self) -> RefMut<'_, dyn RngCore> {
        self.rng.borrow_mut()
    }

    fn interact_pre_move_with_tag(
        &self,
        tag0: &Tag,
        player_loc: &Point,
        new_loc: &Point,
        events: &mut Vec<Event>,
    ) -> bool {
        for (_, obj) in self.cell_iter(new_loc) {
            for tag1 in obj.iter() {
                if self
                    .interactions
                    .pre_move(tag0, tag1, self, player_loc, new_loc, events)
                {
                    return true;
                }
            }
        }
        false
    }
    // Player attempting to interact with an adjacent cell.
    fn interact_pre_move(&self, player_loc: &Point, new_loc: &Point, events: &mut Vec<Event>) -> bool {
        // First see if an inventory item can interact with the new cell.
        for (_, obj) in self.player_inv_iter() {
            for tag0 in obj.iter() {
                if self.interact_pre_move_with_tag(tag0, player_loc, new_loc, events) {
                    return true;
                }
            }
        }

        // Failing that see if the player itself can interact with the cell.
        if self.interact_pre_move_with_tag(&Tag::Player, player_loc, new_loc, events) {
            return true;
        }
        false
    }

    // Player interacting with a cell he has just moved into.
    fn interact_post_move(&self, new_loc: &Point, events: &mut Vec<Event>) {
        let oids = self.cells.get(new_loc).unwrap();
        for oid in oids.iter().rev() {
            let obj = self.objects.get(oid).unwrap();
            for tag in obj.iter() {
                self.interactions.post_move(tag, self, new_loc, events)
            }
        }
    }
}

// Debugging support
impl Game {
    #[cfg(debug_assertions)]
    fn invariant(&self) {
        if self.constructing {
            return;
        }

        // Check what we can that isn't very expensive to do.
        let obj = self.objects.get(&ObjId(0)).expect("oid 0 should always exist");
        assert!(obj.has(PLAYER_ID), "oid 0 should be the player not {obj}");

        let obj = self.objects.get(&ObjId(1));
        assert!(
            obj.is_none(),
            "oid 1 should be the default object, not {}",
            obj.unwrap()
        );

        let oids = self.cells.get(&self.player).expect("player should be on the map");
        assert!(
            oids.iter().any(|oid| self.objects.get(oid).unwrap().has(PLAYER_ID)),
            "player isn't at {}",
            self.player
        );

        if self.invariants {
            self.expensive_invariants();
        } else {
            self.cheap_invariants(&self.player);
        }
    }

    // This only checks invariants at one cell. Not ideal but it does give us some coverage
    // of the level without being really expensive.
    #[cfg(debug_assertions)]
    fn cheap_invariants(&self, loc: &Point) {
        let oids = self.cells.get(loc).expect("cell at {loc} should exist");
        assert!(
            !oids.is_empty(),
            "cell at {loc} is empty (should have at least a terrain object)"
        );

        for (i, oid) in oids.iter().enumerate() {
            let obj = self.objects.get(oid).expect("oid {oid} at {loc} is not in objects");

            if i == 0 {
                assert!(
                    obj.has(TERRAIN_ID),
                    "cell at {loc} has {obj} for the first object instead of a terrain object"
                );
            } else {
                assert!(
                    !obj.has(TERRAIN_ID),
                    "cell at {loc} has {obj} which isn't at the bottom"
                );
            }

            if i < oids.len() - 1 {
                assert!(!obj.has(CHARACTER_ID), "cell at {loc} has {obj} which isn't at the top");
            }

            obj.invariant();
        }
    }

    // This checks every cell and every object so it is pretty slow.
    #[cfg(debug_assertions)]
    fn expensive_invariants(&self) {
        // First we'll check global constraints.
        let mut all_oids = FnvHashSet::default();
        for (loc, oids) in &self.cells {
            for oid in oids {
                assert!(all_oids.insert(oid), "{loc} has oid {oid} which exists elsewhere");
                assert!(self.objects.contains_key(oid), "oid {oid} is not in objects");
            }
        }

        for obj in self.objects.values() {
            if let Some(oids) = obj.as_ref(INVENTORY_ID) {
                for oid in oids {
                    assert!(all_oids.insert(oid), "{obj} has oid {oid} which exists elsewhere");
                    assert!(self.objects.contains_key(oid), "oid {oid} is not in objects");
                }
            }
        }

        assert_eq!(
            all_oids.len(),
            self.objects.len(),
            "all objects should be used somewhere"
        );

        // Then we'll verify that the objects in a cell are legit.
        for (loc, oids) in &self.cells {
            assert!(
                !oids.is_empty(),
                "cell at {loc} is empty (should have at least a terrain object)"
            );
            let obj = self.objects.get(&oids[0]).unwrap();
            assert!(
                obj.has(TERRAIN_ID),
                "cell at {loc} has {obj} for the first object instead of a terrain object"
            );
            assert!(
                !oids
                    .iter()
                    .skip(1)
                    .any(|oid| self.objects.get(oid).unwrap().has(TERRAIN_ID)),
                "cell at {loc} has multiple terrain objects"
            );

            let index = oids.iter().position(|oid| {
                let obj = self.objects.get(oid).unwrap();
                obj.has(CHARACTER_ID)
            });
            if let Some(index) = index {
                // If not cells won't render quite right.
                assert!(index == oids.len() - 1, "{loc} has a Character that is not at the top")
            }
        }

        // Finally we'll check each individual object.
        for obj in self.objects.values() {
            obj.invariant();
        }
    }
}

mod details {
    use super::{FnvHashMap, ObjId, Object, PoV, Point};

    /// View into game after posting an event to Level.
    pub struct Game1<'a> {
        pub objects: &'a FnvHashMap<ObjId, Object>,
        pub cells: &'a FnvHashMap<Point, Vec<ObjId>>,
    }

    pub struct Game2<'a> {
        pub objects: &'a FnvHashMap<ObjId, Object>,
        pub cells: &'a FnvHashMap<Point, Vec<ObjId>>,
        pub pov: &'a PoV,
    }
}

struct CellIterator<'a> {
    game: &'a Game,
    oids: Option<&'a Vec<ObjId>>,
    index: i32,
}

impl<'a> CellIterator<'a> {
    fn new(game: &'a Game, loc: &Point) -> CellIterator<'a> {
        let oids = game.cells.get(loc);
        CellIterator {
            game,
            oids,
            index: oids.map(|list| list.len() as i32).unwrap_or(-1),
        }
    }
}

impl<'a> Iterator for CellIterator<'a> {
    type Item = (ObjId, &'a Object);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(oids) = self.oids {
            self.index -= 1;
            if self.index >= 0 {
                let index = self.index as usize;
                let oid = oids[index];
                Some((oid, self.game.objects.get(&oid).unwrap()))
            } else {
                None // finished iteration
            }
        } else {
            None // nothing at the loc
        }
    }
}

struct InventoryIterator<'a> {
    game: &'a Game,
    oids: &'a Vec<ObjId>,
    index: i32,
}

impl<'a> InventoryIterator<'a> {
    fn new(game: &'a Game, loc: &Point) -> InventoryIterator<'a> {
        let (_, inv) = game.get(loc, INVENTORY_ID).unwrap();
        let oids = inv.as_ref(INVENTORY_ID).unwrap();
        InventoryIterator {
            game,
            oids,
            index: oids.len() as i32,
        }
    }
}

impl<'a> Iterator for InventoryIterator<'a> {
    type Item = (ObjId, &'a Object);

    fn next(&mut self) -> Option<Self::Item> {
        self.index -= 1; // no real need to iterate in reverse order but it is consistent with CellIterator
        if self.index >= 0 {
            let index = self.index as usize;
            let oid = self.oids[index];
            Some((oid, self.game.objects.get(&oid).unwrap()))
        } else {
            None // finished iteration
        }
    }
}
