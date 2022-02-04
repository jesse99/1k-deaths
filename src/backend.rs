//! Contains the game logic, i.e. everything but rendering, user input, and program initialization.
mod event;
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
const MAX_QUEUED_EVENTS: usize = 1_000; // TODO: make this even larger?

#[derive(Clone, Copy, Debug, Display, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ObjId(u64); // TODO: probably want something more intelligible

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
    default: Object, // object to insert when querying for a non-existent cell (can happen for stuff like digging)
    objects: FnvHashMap<ObjId, Object>, // all existing objects are here
    cells: FnvHashMap<Point, Vec<ObjId>>, // objects within each cell on the map
    constructing: bool, // level is in the process of being constructed

    messages: Vec<Message>,     // messages shown to the player
    interactions: Interactions, // double dispatch action tables, e.g. player vs door

    pov: PoV,        // locations that the player can currently see
    old_pov: OldPoV, // locations that the user has seen in the past (this will often be stale data)
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

impl Game {
    fn new(messages: Vec<Message>, seed: u64, file: Option<File>) -> Game {
        info!("using seed {seed}");
        Game {
            stream: Vec::new(),
            file,
            state: State::Adventuring,
            posting: false,
            next_id: 1,

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
        }
    }

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

    fn has(&self, loc: &Point, tag: TagId) -> bool {
        let oids = self
            .cells
            .get(loc)
            .expect("get methods should only be called for valid locations");
        for oid in oids.iter() {
            let obj = self
                .objects
                .get(oid)
                .expect("All objects in the level should still exist");
            if obj.has(tag) {
                return true;
            }
        }
        false
    }

    fn get(&self, loc: &Point, tag: TagId) -> Option<(ObjId, &Object)> {
        let oids = self
            .cells
            .get(loc)
            .expect("get methods should only be called for valid locations");
        for oid in oids.iter().rev() {
            let obj = self
                .objects
                .get(oid)
                .expect("All objects in the level should still exist");
            if obj.has(tag) {
                return Some((*oid, obj));
            }
        }
        None
    }

    // get_mut would be nicer but couldn't figure out how to write that.
    fn mutate<F>(&mut self, loc: &Point, tag: TagId, callback: F)
    where
        F: Fn(&mut Object),
    {
        let oids = self
            .cells
            .get(loc)
            .expect("get methods should only be called for valid locations");
        for oid in oids.iter() {
            let obj = self
                .objects
                .get_mut(oid)
                .expect("All objects in the level should still exist");
            if obj.has(tag) {
                callback(obj);
                return;
            }
        }
        panic!("Didn't find {tag} at {loc}");
    }

    /// Typically this will be a terrain object.
    fn get_bottom(&self, loc: &Point) -> (ObjId, &Object) {
        let oids = self
            .cells
            .get(loc)
            .expect("get methods should only be called for valid locations");
        let oid = oids
            .first()
            .expect("cells should always have at least a terrain object");
        let obj = self
            .objects
            .get(oid)
            .expect("All objects in the level should still exist");
        (*oid, obj)
    }

    /// Character, item, door, or if all else fails terrain.
    fn get_top(&self, loc: &Point) -> (ObjId, &Object) {
        let oids = self
            .cells
            .get(loc)
            .expect("get methods should only be called for valid locations");
        let oid = oids.last().expect("cells should always have at least a terrain object");
        let obj = self
            .objects
            .get(oid)
            .expect("All objects in the level should still exist");
        (*oid, obj)
    }

    /// Iterates over the objects at loc starting with the topmost object.
    fn cell_iter(&self, loc: &Point) -> impl Iterator<Item = (ObjId, &Object)> {
        CellIterator::new(self, loc)
    }

    fn player_inv_iter(&self) -> impl Iterator<Item = (ObjId, &Object)> {
        InventoryIterator::new(self, &self.player)
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

    // In order to ensure that games are replayable mutation should only happen
    // because of an event. To help ensure that this should be the only public
    // mutable Game method. TODO: this doesn't really help because child modules
    // can still call our private methods
    pub fn post(&mut self, events: Vec<Event>, replay: bool) {
        // This is bad because it messes up replay: if it is allowed then an event will
        // post a new event X both of which will be persisted. Then on replay the event
        // will post X but X will have been also saved so X is done twice.
        assert!(!self.posting, "Cannot post an event in response to an event");

        self.posting = true;
        for event in events {
            trace!("posting {event}");
            self.internal_post(event, replay);
        }

        OldPoV::update(self);
        PoV::refresh(self);
        self.posting = false;
        self.invariant();
    }
}

impl Game {
    // This should only be called by the post method.
    fn internal_post(&mut self, event: Event, replay: bool) {
        // It'd be slicker to use a different Game type when replaying. This would prevent
        // us, at compile time, from touching fields like stream or rng. In practice however
        // this isn't much of an issue because the bulk of the code is already prevented
        // from doing bad things by the Game1, Game2, etc structs.
        if !replay {
            self.stream.push(event.clone());

            if self.stream.len() >= MAX_QUEUED_EVENTS {
                self.append_stream();
            }
        }

        let mut events = vec![event];
        while !events.is_empty() {
            let event = events.remove(0); // icky remove from front but the vector shouldn't be very large...
            let moved_to = match event {
                Event::PlayerMoved(new_loc) => Some(new_loc),
                _ => None,
            };

            if let Some(event) = self.do_event(event) {
                // This is the type state pattern: as events are posted new state
                // objects are updated and upcoming state objects can safely reference
                // them. To enforce this at compile time Game1, Game2, etc objects
                // are used to provide views into Game.
                let game1 = details::Game1 {
                    objects: &self.objects,
                    cells: &self.cells,
                };
                self.pov.posted(&game1, &event);

                let game2 = details::Game2 {
                    objects: &self.objects,
                    cells: &self.cells,
                    pov: &self.pov,
                };
                self.old_pov.posted(&game2, &event);
            }

            if let Some(new_loc) = moved_to {
                // Icky recursion: when we do stuff like move into a square
                // we want to immediately take various actions, like printing
                // "You splash through the water".
                self.interact_post_move(&new_loc, &mut events);
            }
        }
    }

    fn append_stream(&mut self) {
        if let Some(se) = &mut self.file {
            if let Err(err) = persistence::append_game(se, &self.stream) {
                self.messages
                    .push(Message::new(Topic::Error, &format!("Couldn't save game: {err}")));
            }
        }
        // If we can't save there's not much we can do other than clear. (Still worthwhile
        // appending onto the stream because we may want a wizard command to show the last
        // few events).
        self.stream.clear();
    }

    // We're using a RefCell to avoid taking too many mutable Game references.
    fn rng(&self) -> RefMut<'_, dyn RngCore> {
        self.rng.borrow_mut()
    }

    fn do_event(&mut self, event: Event) -> Option<Event> {
        match event {
            Event::AddMessage(message) => {
                if let Topic::Error = message.topic {
                    // TODO: do we really want to do this?
                    error!("Logged error '{}'", message.text);
                }
                self.messages.push(message);
                while self.messages.len() > MAX_MESSAGES {
                    self.messages.remove(0); // TODO: this is an O(N) operation for Vec, may want to switch to circular_queue
                }
                None
            }
            Event::AddObject(loc, obj) => {
                if obj.has(PLAYER_ID) {
                    self.player = loc;
                    self.create_player(&loc, obj);
                } else {
                    let oid = self.create_object(obj);
                    let oids = self.cells.entry(loc).or_insert_with(Vec::new);
                    oids.push(oid);
                };
                None
            }
            Event::AddToInventory(loc) => {
                let oid = {
                    let (oid, obj) = self.get(&loc, PORTABLE_ID).unwrap(); // TODO: this only picks up the topmost item
                    let name: String = obj.value(NAME_ID).unwrap();
                    let mesg = Message {
                        topic: Topic::Normal,
                        text: format!("You pick up the {name}."),
                    };
                    self.messages.push(mesg);
                    oid
                };

                {
                    let oids = self.cells.get_mut(&loc).unwrap();
                    let index = oids.iter().position(|id| *id == oid).unwrap();
                    oids.remove(index);
                }

                let loc = self.player;
                self.mutate(&loc, INVENTORY_ID, |obj| {
                    let inv = obj.as_mut_ref(INVENTORY_ID).unwrap();
                    inv.push(oid);
                });

                Some(event)
            }
            Event::BeginConstructLevel => {
                let oid = ObjId(0);
                let player = self.objects.remove(&oid);
                self.objects = FnvHashMap::default();
                self.cells = FnvHashMap::default();
                if let Some(player) = player {
                    self.objects.insert(oid, player);
                }
                self.constructing = true;
                Some(event)
            }
            Event::DestroyObjectId(loc, oid) => {
                self.destroy_object(&loc, oid);
                None
            }
            Event::EndConstructLevel => {
                self.constructing = false;
                Some(event)
            }
            Event::NewGame => {
                // TODO: do we want this event?
                Some(event)
            }
            Event::NPCMoved(old, new) => {
                let (oid, _) = self.get(&old, CHARACTER_ID).unwrap();

                let oids = self.cells.get_mut(&old).unwrap();
                let index = oids.iter().position(|id| *id == oid).unwrap();
                oids.remove(index);

                let cell = self.cells.entry(new).or_insert_with(Vec::new);
                cell.push(oid);
                self.ensure_neighbors(&new);
                Some(event)
            }
            Event::PlayerMoved(loc) => {
                assert!(!self.constructing); // make sure this is reset once things start happening
                let oid = ObjId(0);
                let oids = self.cells.get_mut(&self.player).unwrap();
                let index = oids.iter().position(|id| *id == oid).unwrap();
                oids.remove(index);

                self.player = loc;
                let cell = self.cells.entry(self.player).or_insert_with(Vec::new);
                cell.push(oid);

                self.ensure_neighbors(&loc);
                Some(event)
            }
            Event::ReplaceObject(loc, old_oid, obj) => {
                let new_oid = self.create_object(obj);

                let oids = self.cells.get_mut(&loc).unwrap();
                let index = oids.iter().position(|id| *id == old_oid).unwrap();
                info!("removing {} with id {old_oid}", self.objects.get(&old_oid).unwrap());
                oids[index] = new_oid;

                self.objects.remove(&old_oid);
                None
            }
            Event::StateChanged(state) => {
                self.state = state;
                Some(event)
            }
        }
    }

    fn create_player(&mut self, loc: &Point, obj: Object) -> ObjId {
        let oid = ObjId(0);
        info!("creating new {obj} with id {oid}");
        self.objects.insert(oid, obj);

        let oids = self.cells.entry(*loc).or_insert_with(Vec::new);
        oids.push(oid);
        oid
    }

    // This does not update cells (the object may go elsewhere).
    fn create_object(&mut self, obj: Object) -> ObjId {
        let oid = ObjId(self.next_id);
        info!("creating new {obj} with id {oid}");
        self.next_id += 1;
        self.objects.insert(oid, obj);
        oid
    }

    fn destroy_object(&mut self, loc: &Point, old_oid: ObjId) {
        let oids = self.cells.get_mut(loc).unwrap();
        let index = oids.iter().position(|id| *id == old_oid).unwrap();
        let obj = self.objects.get(&old_oid).unwrap();
        info!("destroying {obj} with id {old_oid}");
        if obj.has(TERRAIN_ID) {
            // Terrain cannot be destroyed but has to be mutated into something else.
            let new_obj = if obj.has(WALL_ID) {
                make::rubble()
            } else {
                error!("Need to better handle destroying TagId {obj}"); // Doors, trees, etc
                make::dirt()
            };
            let new_oid = ObjId(self.next_id);
            info!("creating new {new_obj} with id {new_oid}");
            self.next_id += 1;
            self.objects.insert(new_oid, new_obj);
            oids[index] = new_oid;
        } else {
            // If it's just a normal object or character we can just nuke the object.
            oids.remove(index);
        }
        self.objects.remove(&old_oid);
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

    // This should only be called by the pov code.
    fn ensure_cell(&mut self, loc: &Point) -> bool {
        if self.constructing {
            self.cells.contains_key(loc)
        } else {
            self.ensure_neighbors(loc);
            true
        }
    }

    // Ideally we would have get_mut and get create a new default cell for
    // the given location. That's easy for get_mut but get would require
    // interior mutability. Also easy..until you start handing out references
    // as get wants to do. We could do that too but then clients have a really
    // annoying constraint: they cannot call get if code anywhere in the call
    // chain has an outstanding cell reference (because get requires that a
    // new mutable reference be taken).
    //
    // So what we do instead is ensure that:
    // 1) When we modify a cell (via get_mut) that all the neighbors are
    // present. This case is for something like destroying a wall.
    // 2) When a character moves we ensure that the new location has all
    // neighbors. This is for something like being able to move into a wall
    // (or something like deep shadow).
    fn ensure_neighbors(&mut self, loc: &Point) {
        if !self.constructing {
            let deltas = vec![(-1, -1), (-1, 1), (-1, 0), (1, -1), (1, 1), (1, 0), (0, -1), (0, 1)];
            for delta in deltas {
                let new_loc = Point::new(loc.x + delta.0, loc.y + delta.1);
                let _ = self.cells.entry(new_loc).or_insert_with(|| {
                    let oid = ObjId(self.next_id);
                    info!("creating new {} with id {oid}", self.default);
                    self.next_id += 1;
                    self.objects.insert(oid, self.default.clone());
                    vec![oid]
                });
            }
        }
    }

    fn invariant(&self) {
        if self.constructing {
            return;
        }

        // First we'll check global constraints.
        let obj = self.objects.get(&ObjId(0)).expect("oid 0 should always exist");
        assert!(obj.has(PLAYER_ID), "oid 0 should be the player not {obj}");

        // let obj = self.objects.get(&ObjId(1));   // TODO: enable these checks
        // assert!(
        //     obj.is_none(),
        //     "oid 1 should be the default object, not {}",
        //     obj.unwrap()
        // );

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
        let oids = self.cells.get(&self.player).expect("player should be on the map");
        assert!(
            oids.iter().any(|oid| self.objects.get(oid).unwrap().has(PLAYER_ID)),
            "player isn't at {}",
            self.player
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

impl Drop for Game {
    fn drop(&mut self) {
        self.append_stream();
    }
}
