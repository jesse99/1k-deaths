//! Contains the game logic, i.e. everything but rendering, user input, and program initialization.
mod actions;
mod interactions;
mod make;
mod message;
mod object;
mod old_pov;
mod persistence;
mod pov;
mod primitives;
mod scheduler;
mod tag;
mod time;

pub use message::{Message, Topic};
pub use object::Symbol;
pub use primitives::Color;
pub use primitives::Point;
pub use primitives::Size;

use derive_more::Display;
use fnv::FnvHashMap;
use interactions::{Interactions, PreHandler, PreResult};
use object::{Object, TagValue};
use old_pov::OldPoV;
use pov::PoV;
use rand::prelude::*;
use rand::rngs::SmallRng;
use rand::RngCore;
use std::cell::{RefCell, RefMut};
// use std::os::unix::prelude::FileTypeExt;
use scheduler::Scheduler;
use std::fs::File;
use tag::*;
use tag::{Durability, Material, Tag};
use time::Time;

#[cfg(debug_assertions)]
use fnv::FnvHashSet;

const MAX_MESSAGES: usize = 1000;
const MAX_QUEUED_EVENTS: usize = 1_000; // TODO: make this even larger?

// TODO: These numbers are not very intelligible. If that becomes an issue we could use
// a newtype string (e.g. "wall 97") or a simple struct with a static string ref and a
// counter.
#[derive(Clone, Copy, Debug, Display, Eq, Hash, PartialEq)]
pub struct Oid(u64);

/// Represents what the player wants to do next. Most of these will use up the player's
/// remaining time units, but some like (Examine) don't take any time.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Action {
    /// Move the player to empty cells (or attempt to interact with an object at that cell).
    /// dx and dy must be 0, +1, or -1.
    Move { dx: i32, dy: i32 },

    /// Print descriptions for objects at the cell. Note that any cell can be examined but
    /// cells that are not in the player's PoV will have either an unhelpful description or
    /// a stale description.
    Examine { loc: Point, wizard: bool },
}

pub enum Tile {
    /// player can see this
    Visible { bg: Color, fg: Color, symbol: Symbol },
    /// player can't see this but has in the past, note that this may not reflect the current state
    Stale(Symbol),
    /// player has never seen this location (and it may not exist)
    NotVisible,
}

#[derive(Clone, Copy, Debug, Display, Eq, PartialEq, Serialize, Deserialize)]
pub enum State {
    Adventuring,
    KilledRhulad,
    WonGame,
    LostGame,
}

/// Top-level backend object encapsulating the game state.
pub struct Game {
    stream: Vec<Action>, // used to reconstruct games
    file: Option<File>,  // actions are perodically saved here
    state: State,        // game milestones, eg won game
    next_id: u64,        // 0 is the player
    rng: RefCell<SmallRng>,
    scheduler: Scheduler,

    player: Point,
    players_move: bool,
    default: Object, // object to use for a non-existent cell (can happen if a wall is destroyed)
    objects: FnvHashMap<Oid, Object>, // all existing objects are here
    cells: FnvHashMap<Point, Vec<Oid>>, // objects within each cell on the map
    constructing: bool, // level is in the process of being constructed

    messages: Vec<Message>,     // messages shown to the player
    interactions: Interactions, // double dispatch action tables, e.g. player vs door
    pov: PoV,                   // locations that the player can currently see
    old_pov: OldPoV,            // locations that the user has seen in the past (this will often be stale data)
    #[cfg(debug_assertions)]
    invariants: bool, // if true then expensive checks are enabled
}

// Public API.
impl Game {
    /// Start a brand new game and save it to path.
    pub fn new_game(path: &str, seed: u64) -> Game {
        let mut messages = Vec::new();

        info!("new {path}");
        let file = match persistence::new_game(path, seed) {
            Ok(se) => Some(se),
            Err(err) => {
                messages.push(Message::new(
                    Topic::Error,
                    &format!("Couldn't open {path} for writing: {err}"),
                ));
                None
            }
        };

        messages.push(Message {
            topic: Topic::Important,
            text: String::from("Welcome to 1k-deaths!"),
        });
        messages.push(Message {
            topic: Topic::Important,
            text: String::from("Are you the hero who will destroy the Crippled God's sword?"),
        });
        messages.push(Message {
            topic: Topic::Important,
            text: String::from("Press the '?' key for help."),
        });

        Game::new(messages, seed, file)
    }

    /// Load a saved game and return the actions so that they can be replayed.
    pub fn old_game(path: &str) -> (Game, Vec<Action>) {
        let mut seed = 1;
        let mut actions = Vec::new();
        let mut messages = Vec::new();

        let mut file = None;
        info!("loading {path}");
        match persistence::load_game(path) {
            Ok((s, a)) => {
                seed = s;
                actions = a;
            }
            Err(err) => {
                info!("loading file had err: {err}");
                messages.push(Message::new(
                    Topic::Error,
                    &format!("Couldn't open {path} for reading: {err}"),
                ));
            }
        };

        if !actions.is_empty() {
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
            (Game::new(messages, seed, file), actions)
        } else {
            let mut game = Game::new_game(path, seed);
            game.messages.extend(messages);
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

    pub fn player(&self) -> Point {
        self.player
    }

    pub fn in_progress(&self) -> bool {
        !matches!(self.state, State::LostGame)
    }

    /// If this returns true then the UI should call command, otherwise the UI should call
    /// advance_time.
    pub fn players_turn(&self) -> bool {
        self.players_move
    }

    fn obj_acted(&mut self, oid: Oid, units: Time) -> Option<(Time, Time)> {
        let base = time::FLOOD;
        let extra = self.extra_flood_delay();
        if units >= base + extra {
            let loc = self.loc(oid).unwrap();
            let obj = self.objects.get(&oid).unwrap();
            if obj.has(DEEP_WATER_ID) {
                trace!("{oid} at {loc} is deep flooding");
                self.do_flood_deep(oid, loc);
            } else {
                trace!("{oid} at {loc} is shallow flooding");
                self.do_flood_shallow(oid, loc);
            };
            {
                #[cfg(debug_assertions)]
                self.invariant();
            }
            Some((base, extra))
        } else {
            None
        }
    }

    // Either we need to allow the player to move or we need to re-render because an
    // obhect did something.
    pub fn advance_time(&mut self) {
        if Scheduler::players_turn(self) {
            self.players_move = true;
        } else {
            OldPoV::update(self);
            PoV::refresh(self);
        }
    }

    // TODO: need a new lookup table for this
    fn loc(&self, oid: Oid) -> Option<Point> {
        for (loc, oids) in &self.cells {
            for candidate in oids {
                if *candidate == oid {
                    return Some(*loc);
                }
            }
        }
        None
    }

    pub fn player_acted(&mut self, action: Action, replay: bool) {
        // TODO: probably want to return something to indicate whether a UI refresh is neccesary
        // TODO: maybe something fine grained, like only need to update messages
        trace!("player is doing {action:?}");
        match action {
            Action::Move { dx, dy } => {
                assert!(dx >= -1 && dx <= 1);
                assert!(dy >= -1 && dy <= 1);
                assert!(dx != 0 || dy != 0);
                if self.in_progress() {
                    let player = self.player;
                    let new_loc = Point::new(player.x + dx, player.y + dy);
                    let duration = match self.try_interact(&player, &new_loc) {
                        Some(PreResult::Acted(taken)) => {
                            assert!(taken > Time::zero());
                            taken
                        }
                        Some(PreResult::ZeroAction) => Time::zero(),
                        None => {
                            let old_loc = self.player;
                            self.do_move(Oid(0), &old_loc, &new_loc);
                            if old_loc.diagnol(&new_loc) {
                                time::DIAGNOL_MOVE + self.interact_post_move(&new_loc)
                            } else {
                                time::CARDINAL_MOVE + self.interact_post_move(&new_loc)
                            }
                        }
                    };

                    if duration > Time::zero() {
                        self.scheduler.player_acted(duration, &self.rng);
                        self.players_move = false;

                        OldPoV::update(self);
                        PoV::refresh(self);
                    }

                    {
                        #[cfg(debug_assertions)]
                        self.invariant();
                    }
                }
            }
            Action::Examine { loc, wizard } => {
                let suffix = if wizard { format!(" {}", loc) } else { "".to_string() };
                let text = if self.pov.visible(&loc) {
                    let descs: Vec<String> = self
                        .cell_iter(&loc)
                        .map(|(_, obj)| obj.description().to_string())
                        .collect();
                    let descs = descs.join(", and ");
                    format!("You see {descs}{suffix}.")
                } else if self.old_pov.get(&loc).is_some() {
                    format!("You can no longer see there{suffix}.")
                } else {
                    format!("You've never seen there{suffix}.")
                };
                self.messages.push(Message {
                    topic: Topic::Normal,
                    text,
                });
                if wizard {
                    let messages: Vec<String> = self.cell_iter(&loc).map(|(_, obj)| format!("   {obj:?}")).collect();
                    for text in messages.into_iter() {
                        self.messages.push(Message {
                            topic: Topic::Normal,
                            text,
                        });
                    }
                }
            }
        }

        if !replay {
            self.stream.push(action);
            if self.stream.len() >= MAX_QUEUED_EVENTS {
                self.save_actions();
            }
        }
        while self.messages.len() > MAX_MESSAGES {
            self.messages.remove(0); // TODO: this is an O(N) operation for Vec, may want to switch to circular_queue
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
                Some(symbol) => Tile::Stale(*symbol),
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
    #[cfg(debug_assertions)]
    pub fn set_invariants(&mut self, enable: bool) {
        // TODO: might want a wizard command to enable these
        self.invariants = enable;
    }
}

// Backend methods.
impl Game {
    fn new(messages: Vec<Message>, seed: u64, file: Option<File>) -> Game {
        info!("using seed {seed}");
        let mut game = Game {
            stream: Vec::new(),
            file,
            state: State::Adventuring,
            next_id: 2,
            scheduler: Scheduler::new(),

            // TODO: SmallRng is not guaranteed to be portable so results may
            // not be reproducible between platforms.
            rng: RefCell::new(SmallRng::seed_from_u64(seed)),

            player: Point::origin(),
            players_move: false,
            objects: FnvHashMap::default(),
            cells: FnvHashMap::default(),
            default: make::stone_wall(),
            constructing: true,

            messages,
            interactions: Interactions::new(),
            pov: PoV::new(),
            old_pov: OldPoV::new(),
            #[cfg(debug_assertions)]
            invariants: false,
        };
        game.init_game();
        game
    }

    fn init_game(&mut self) {
        let map = include_str!("backend/maps/start.txt");
        make::level(self, map);
        self.constructing = false;

        OldPoV::update(self);
        PoV::refresh(self);
    }

    fn has(&self, loc: &Point, tag: Tid) -> bool {
        if let Some(oids) = self.cells.get(loc) {
            for oid in oids {
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

    fn get(&self, loc: &Point, tag: Tid) -> Option<(Oid, &Object)> {
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
            Some((Oid(1), &self.default))
        } else {
            None
        }
    }

    fn get_mut(&mut self, loc: &Point, tag: Tid) -> Option<(Oid, &mut Object)> {
        if !self.cells.contains_key(loc) {
            self.add_default(loc);
        }

        let mut oid = None;
        if let Some(oids) = self.cells.get(loc) {
            for candidate in oids.iter().rev() {
                let obj = self
                    .objects
                    .get(candidate)
                    .expect("All objects in the level should still exist");
                if obj.has(tag) {
                    oid = Some(candidate);
                    break;
                }
            }
        }

        if let Some(oid) = oid {
            let obj = self.objects.get_mut(oid).unwrap();
            return Some((*oid, obj));
        }

        None
    }

    /// Typically this will be a terrain object.
    fn get_bottom(&self, loc: &Point) -> (Oid, &Object) {
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
            (Oid(1), &self.default)
        }
    }

    /// Character, item, door, or if all else fails terrain.
    fn get_top(&self, loc: &Point) -> (Oid, &Object) {
        if let Some(oids) = self.cells.get(loc) {
            let oid = oids.last().expect("cells should always have at least a terrain object");
            let obj = self
                .objects
                .get(oid)
                .expect("All objects in the level should still exist");
            (*oid, obj)
        } else {
            (Oid(1), &self.default)
        }
    }

    /// Iterates over the objects at loc starting with the topmost object.
    fn cell_iter(&self, loc: &Point) -> impl Iterator<Item = (Oid, &Object)> {
        CellIterator::new(self, loc)
    }

    fn player_inv_iter(&self) -> impl Iterator<Item = (Oid, &Object)> {
        InventoryIterator::new(self, &self.player)
    }

    // The RNG doesn't directly affect the game state so we use interior mutability for it.
    fn rng(&self) -> RefMut<'_, dyn RngCore> {
        self.rng.borrow_mut()
    }

    fn extra_flood_delay(&self) -> Time {
        let rng = &mut *self.rng();
        let t: i64 = 60 + rng.gen_range(0..(400 * 6));
        time::secs(t)
    }

    fn find_interact_handler(&self, tag0: &Tag, new_loc: &Point) -> Option<PreHandler> {
        for (_, obj) in self.cell_iter(new_loc) {
            for tag1 in obj.iter() {
                let handler = self.interactions.find_interact_handler(tag0, tag1);
                if handler.is_some() {
                    return handler;
                }
            }
        }
        None
    }

    // Player attempting to interact with an adjacent cell.
    fn try_interact(&mut self, player_loc: &Point, new_loc: &Point) -> Option<PreResult> {
        let mut handler = None;

        // First see if an inventory item can interact with the new cell.
        {
            'outer: for (_, obj) in self.player_inv_iter() {
                for tag0 in obj.iter() {
                    handler = self.find_interact_handler(tag0, new_loc);
                    if handler.is_some() {
                        break 'outer;
                    }
                }
            }
        }

        if handler.is_some() {
            return Some(handler.unwrap()(self, player_loc, new_loc));
        }

        // If we couldn't find a handler for an item or that handler returned None then
        // see if the player itself can interact with the cell.
        handler = self.find_interact_handler(&Tag::Player, new_loc);
        if handler.is_some() {
            Some(handler.unwrap()(self, player_loc, new_loc))
        } else {
            None
        }
    }

    // Player interacting with a cell he has just moved into.
    fn interact_post_move(&mut self, new_loc: &Point) -> Time {
        let mut handlers = Vec::new();
        {
            let oids = self.cells.get(new_loc).unwrap();
            for oid in oids.iter().rev() {
                let obj = self.objects.get(oid).unwrap();
                for tag in obj.iter() {
                    if let Some(handler) = self.interactions.find_post_handler(&Tag::Player, tag) {
                        handlers.push(*handler);
                    }
                }
            }
        }

        let mut extra = Time::zero();
        for handler in handlers.into_iter() {
            extra = extra + handler(self, new_loc);
        }
        extra
    }

    fn init_cell(&mut self, loc: Point, obj: Object) {
        let scheduled = obj.has(SCHEDULED_ID);
        let oid = if obj.has(PLAYER_ID) {
            self.player = loc;
            self.create_player(&loc, obj)
        } else {
            let oid = self.create_object(obj);
            let oids = self.cells.entry(loc).or_insert_with(Vec::new);
            oids.push(oid);
            oid
        };
        if scheduled {
            self.schedule_new_obj(oid);
        }
    }

    fn add_object(&mut self, loc: &Point, obj: Object) {
        trace!("adding {obj} to {loc}");
        let scheduled = obj.has(SCHEDULED_ID);
        let oid = if obj.has(PLAYER_ID) {
            self.player = *loc;
            self.create_player(&loc, obj)
        } else {
            let oid = self.create_object(obj);
            let oids = self.cells.entry(*loc).or_insert_with(Vec::new);
            oids.push(oid);
            oid
        };
        if scheduled {
            self.schedule_new_obj(oid);
        }
    }

    // This does not update cells (the object may go elsewhere).
    fn create_object(&mut self, obj: Object) -> Oid {
        let oid = Oid(self.next_id);
        self.next_id += 1;
        self.objects.insert(oid, obj); // TODO: dirty pov?
        oid
    }

    fn create_player(&mut self, loc: &Point, obj: Object) -> Oid {
        let oid = Oid(0);
        self.objects.insert(oid, obj);

        let oids = self.cells.entry(*loc).or_insert_with(Vec::new);
        oids.push(oid);
        oid
    }

    fn schedule_new_obj(&mut self, oid: Oid) {
        let obj = self.objects.get(&oid).unwrap();
        let initial = if oid.0 == 0 {
            time::DIAGNOL_MOVE
        } else if obj.has(SHALLOW_WATER_ID) || obj.has(DEEP_WATER_ID) {
            Time::zero() - self.extra_flood_delay()
        } else {
            Time::zero()
        };
        self.scheduler.add(oid, initial);
    }

    fn add_default(&mut self, new_loc: &Point) {
        let oid = Oid(self.next_id);
        self.next_id += 1;
        self.objects.insert(oid, self.default.clone());
        let old_oids = self.cells.insert(*new_loc, vec![oid]);
        assert!(old_oids.is_none());
    }

    fn ensure_neighbors(&mut self, loc: &Point) {
        let deltas = vec![(-1, -1), (-1, 1), (-1, 0), (1, -1), (1, 1), (1, 0), (0, -1), (0, 1)];
        for delta in deltas {
            let new_loc = Point::new(loc.x + delta.0, loc.y + delta.1);
            if !self.cells.contains_key(&new_loc) {
                self.add_default(&new_loc);
            }
        }
    }

    fn destroy_object(&mut self, loc: &Point, old_oid: Oid) {
        let obj = self.objects.get(&old_oid).unwrap();
        trace!("destroying {obj} at {loc}");
        if obj.has(SCHEDULED_ID) {
            self.scheduler.remove(old_oid);
        }

        let oids = self.cells.get_mut(&loc).unwrap();
        let index = oids.iter().position(|id| *id == old_oid).unwrap();
        if obj.has(TERRAIN_ID) {
            // Terrain cannot be destroyed but has to be mutated into something else.
            let new_obj = if obj.has(WALL_ID) {
                make::rubble()
            } else {
                error!("Need to better handle destroying Tid {obj}"); // Doors, trees, etc
                make::dirt()
            };
            let scheduled = new_obj.has(SCHEDULED_ID);

            let new_oid = Oid(self.next_id);
            self.next_id += 1;
            self.objects.insert(new_oid, new_obj);
            oids[index] = new_oid;

            if scheduled {
                self.schedule_new_obj(new_oid);
            }

            // The player may now be able to see through this cell so we need to ensure
            // that cells around it exist now. TODO: probably should have a LOS changed
            // check.
            self.ensure_neighbors(&loc);
        } else {
            // If it's just a normal object or character we can just nuke the object.
            oids.remove(index);
        }
        self.objects.remove(&old_oid);
    }

    fn replace_object(&mut self, loc: &Point, old_oid: Oid, new_obj: Object) {
        let old_obj = self.objects.get(&old_oid).unwrap();
        trace!("replacing {old_obj} at {loc} with {new_obj}");
        if old_obj.has(SCHEDULED_ID) {
            self.scheduler.remove(old_oid);
        }

        let scheduled = new_obj.has(SCHEDULED_ID);
        let new_oid = self.create_object(new_obj);
        let oids = self.cells.get_mut(&loc).unwrap();
        let index = oids.iter().position(|id| *id == old_oid).unwrap();
        oids[index] = new_oid;
        self.objects.remove(&old_oid);

        if scheduled {
            self.schedule_new_obj(new_oid);
        }
    }

    fn find_neighbor<F>(&self, loc: &Point, predicate: F) -> Option<Point>
    where
        F: Fn(&Point) -> bool,
    {
        let mut deltas = vec![(-1, -1), (-1, 1), (-1, 0), (1, -1), (1, 1), (1, 0), (0, -1), (0, 1)];
        deltas.shuffle(&mut *self.rng());
        for delta in deltas {
            let new_loc = Point::new(loc.x + delta.0, loc.y + delta.1);
            if predicate(&new_loc) {
                return Some(new_loc);
            }
        }
        None
    }

    fn find_empty_cell(&self, ch: &Object, loc: &Point) -> Option<Point> {
        let mut deltas = vec![(-1, -1), (-1, 1), (-1, 0), (1, -1), (1, 1), (1, 0), (0, -1), (0, 1)];
        deltas.shuffle(&mut *self.rng());
        for delta in deltas {
            let new_loc = Point::new(loc.x + delta.0, loc.y + delta.1);
            let character = &self.get(&new_loc, CHARACTER_ID);
            if character.is_none() {
                let (_, terrain) = self.get_bottom(&new_loc);
                if ch.impassible_terrain(terrain).is_none() {
                    return Some(new_loc);
                }
            }
        }
        None
    }

    fn save_actions(&mut self) {
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
}

// Debugging support
impl Game {
    #[cfg(debug_assertions)]
    fn invariant(&self) {
        if self.constructing {
            return;
        }

        // Check what we can that isn't very expensive to do.
        let obj = self.objects.get(&Oid(0)).expect("oid 0 should always exist");
        assert!(obj.has(PLAYER_ID), "oid 0 should be the player not {obj}");

        let obj = self.objects.get(&Oid(1));
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

        self.cheap_invariants(&self.player);
        if self.invariants {
            self.expensive_invariants(); // some overlap with cheap_invariants but that should be OK
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

        if let Some((_, ch)) = self.get(loc, CHARACTER_ID) {
            let terrain = self.get(loc, TERRAIN_ID).unwrap().1;
            assert!(
                ch.impassible_terrain(terrain).is_none(),
                "{ch} shouldn't be in {terrain}"
            );
        }

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

struct CellIterator<'a> {
    game: &'a Game,
    oids: Option<&'a Vec<Oid>>,
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
    type Item = (Oid, &'a Object);

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
    oids: &'a Vec<Oid>,
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
    type Item = (Oid, &'a Object);

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

impl Drop for Game {
    fn drop(&mut self) {
        self.save_actions();
    }
}
