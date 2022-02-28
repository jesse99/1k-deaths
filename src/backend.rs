//! Contains the game logic, i.e. everything but rendering, user input, and program initialization.
mod actions;
mod ai;
mod interactions;
mod level;
mod make;
mod melee;
mod message;
mod object;
mod old_pov;
mod persistence;
mod pov;
mod primitives;
mod scheduler;
mod sound;
mod tag;
mod time;

pub use message::{Message, Topic};
pub use object::Symbol;
pub use primitives::Color;
pub use primitives::Point;
pub use primitives::Size;

use derive_more::Display;
use interactions::{Interactions, PreHandler, PreResult};
use level::Level;
use object::{Object, TagValue};
use old_pov::OldPoV;
use pov::PoV;
use rand::prelude::*;
use rand::rngs::SmallRng;
use rand::RngCore;
use rand_distr::StandardNormal;
use scheduler::Scheduler;
// use simplelog::TermLogger;
use sound::Sound;
use std::cell::{RefCell, RefMut};
use std::cmp::{max, min};
use std::fs::File;
use std::io::{Error, Write};
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
    /// Print descriptions for objects at the cell. Note that any cell can be examined but
    /// cells that are not in the player's PoV will have either an unhelpful description or
    /// a stale description.
    Examine {
        loc: Point,
        wizard: bool,
    },

    /// Move the player to empty cells (or attempt to interact with an object at that cell).
    /// dx and dy must be 0, +1, or -1.
    Move {
        dx: i32,
        dy: i32,
    },

    /// Something other than the player did something.
    Object,
    // Be sure to add new actions to the end (or saved games will break).
    Rest,
}

#[derive(Eq, PartialEq)]
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
    rng: RefCell<SmallRng>,
    scheduler: Scheduler,

    level: Level,
    players_move: bool,

    messages: Vec<Message>,     // messages shown to the player
    interactions: Interactions, // double dispatch action tables, e.g. player vs door
    pov: PoV,                   // locations that the player can currently see
    old_pov: OldPoV,            // locations that the user has seen in the past (this will often be stale data)
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
    pub fn old_game(path: &str, warnings: Vec<String>) -> (Game, Vec<Action>) {
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

        messages.extend(warnings.iter().map(|w| Message::new(Topic::Warning, w)));

        if file.is_some() {
            (Game::new(messages, seed, file), actions)
        } else {
            let mut game = Game::new_game(path, seed);
            game.messages.extend(messages);
            (game, Vec::new())
        }
    }

    pub fn dump_state<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.dump_pov(writer)?;
        self.scheduler.dump(writer, self)
    }

    pub fn recent_messages(&self, limit: usize) -> impl Iterator<Item = &Message> {
        let iter = self.messages.iter();
        if limit < self.messages.len() {
            iter.skip(self.messages.len() - limit)
        } else {
            iter.skip(0)
        }
    }

    pub fn add_mesg(&mut self, mesg: Message) {
        self.messages.push(mesg);
    }

    pub fn player_loc(&self) -> Point {
        self.level.player_loc()
    }

    /// If this returns true then the UI should call player_acted, otherwise the UI should
    /// call advance_time.
    pub fn players_turn(&self) -> bool {
        self.players_move || self.game_over()
    }

    // Either we need to allow the player to move or we need to re-render because an
    // obhect did something.
    pub fn advance_time(&mut self, replay: bool) {
        if Scheduler::player_is_ready(self) {
            self.players_move = true;
        } else {
            if !replay {
                self.stream.push(Action::Object);
            }
            OldPoV::update(self);
            PoV::refresh(self);
        }
    }

    pub fn player_acted(&mut self, action: Action) {
        self.do_player_acted(action, false);
    }

    fn do_player_acted(&mut self, action: Action, replay: bool) {
        // TODO: probably want to return something to indicate whether a UI refresh is neccesary
        // TODO: maybe something fine grained, like only need to update messages
        trace!("player is doing {action:?}");
        match action {
            Action::Examine { loc, wizard } => {
                self.examine(&loc, wizard);
            }
            Action::Move { dx, dy } => {
                assert!(dx >= -1 && dx <= 1);
                assert!(dy >= -1 && dy <= 1);
                assert!(dx != 0 || dy != 0);
                if !self.game_over() {
                    let player = self.player_loc();
                    let new_loc = Point::new(player.x + dx, player.y + dy);
                    let duration = match self.try_interact(&player, &new_loc) {
                        PreResult::Acted(taken, sound) => {
                            assert!(taken > Time::zero());
                            self.handle_noise(&self.player_loc(), sound);
                            taken
                        }
                        PreResult::ZeroAction => Time::zero(),
                        PreResult::DidntAct => {
                            let old_loc = self.player_loc();
                            self.do_move(Oid(0), &old_loc, &new_loc);
                            let (duration, volume) = self.interact_post_move(&new_loc);
                            self.handle_noise(&new_loc, sound::QUIET + volume);
                            if old_loc.diagnol(&new_loc) {
                                time::DIAGNOL_MOVE + duration
                            } else {
                                time::CARDINAL_MOVE + duration
                            }
                        }
                    };

                    if duration > Time::zero() {
                        self.scheduler.player_acted(duration, &self.rng);
                        self.players_move = false;

                        OldPoV::update(self);
                        PoV::refresh(self);
                    }
                }
            }
            Action::Object => panic!("Action::Object should only be used with replay_action"),
            Action::Rest => {
                self.scheduler.player_acted(time::DIAGNOL_MOVE, &self.rng);
                self.players_move = false;
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

    pub fn replay_action(&mut self, action: Action) {
        if let Action::Object = action {
            self.advance_time(true);
        } else {
            self.do_player_acted(action, true);
        }
    }

    fn examine(&mut self, loc: &Point, wizard: bool) {
        let suffix = if wizard { format!(" {}", loc) } else { "".to_string() };
        if self.pov.visible(self, &loc) {
            let descs: Vec<String> = self
                .level
                .cell_iter(&loc)
                .map(|(_, obj)| {
                    if wizard {
                        format!("{} {obj:?}", obj.description())
                    } else {
                        obj.description().to_string()
                    }
                })
                .collect();
            if descs.len() == 1 {
                self.messages.push(Message {
                    topic: Topic::Normal,
                    text: format!("You see {}{suffix}.", descs[0]),
                });
            } else {
                self.messages.push(Message {
                    topic: Topic::Normal,
                    text: format!("You see{suffix}"),
                });
                for desc in descs {
                    // TODO: at some point we'll want to cap the number of lines
                    self.messages.push(Message {
                        topic: Topic::Normal,
                        text: format!("   {desc}."),
                    });
                }
            }
        } else if self.old_pov.get(&loc).is_some() {
            self.messages.push(Message {
                topic: Topic::Normal,
                text: format!("You can no longer see there{suffix}."),
            });
        } else {
            self.messages.push(Message {
                topic: Topic::Normal,
                text: format!("You've never seen there{suffix}."),
            });
        };
    }

    /// If loc is valid and within the player's Field if View (FoV) then return the terrain.
    /// Otherwise return None.
    pub fn tile(&self, loc: &Point) -> Tile {
        let tile = if self.pov.visible(self, loc) {
            let (_, obj) = self.level.get_bottom(loc);
            let bg = obj.to_bg_color();

            let (_, obj) = self.level.get_top(loc);
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
        // Find the NPCs near the player that are actually visible to the player.
        let chars: Vec<Point> = self
            .level
            .npcs()
            .map_while(|oid| {
                let loc = self.level.obj(oid).1.unwrap();
                let dist = loc.distance2(&self.player_loc());
                if dist <= pov::RADIUS * pov::RADIUS {
                    Some(loc)
                } else {
                    None
                }
            })
            .filter(|loc| self.pov.visible(self, &loc))
            .collect();

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

    #[cfg(debug_assertions)]
    pub fn set_invariants(&mut self, enable: bool) {
        // TODO: might want a wizard command to enable these
        self.level.set_invariants(enable)
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
            scheduler: Scheduler::new(),

            // TODO: SmallRng is not guaranteed to be portable so results may
            // not be reproducible between platforms.
            rng: RefCell::new(SmallRng::seed_from_u64(seed)),

            level: Level::new(),
            players_move: false,

            messages,
            interactions: Interactions::new(),
            pov: PoV::new(),
            old_pov: OldPoV::new(),
        };
        game.init_game();
        game
    }

    fn init_game(&mut self) {
        let map = include_str!("backend/maps/start.txt");
        make::level(self, map);
        self.level.set_constructing(false);

        OldPoV::update(self);
        PoV::refresh(self);
    }

    fn game_over(&self) -> bool {
        matches!(self.state, State::LostGame | State::WonGame)
    }

    // TODO: Not sure we'll need this in the future.
    fn loc(&self, oid: Oid) -> Option<Point> {
        self.level.try_loc(oid)
    }

    fn player_inv_iter(&self) -> impl Iterator<Item = (Oid, &Object)> {
        InventoryIterator::new(self, &self.player_loc())
    }

    pub fn in_inv(&self, ch: &Object, id: Tid) -> bool {
        if let Some(oids) = ch.as_ref(INVENTORY_ID) {
            for oid in oids {
                let obj = self.level.obj(*oid).0;
                if obj.has(id) {
                    return true;
                }
            }
        }
        false
    }

    // The RNG doesn't directly affect the game state so we use interior mutability for it.
    fn rng(&self) -> RefMut<'_, dyn RngCore> {
        self.rng.borrow_mut()
    }

    fn find_interact_handler(&self, tag0: &Tag, new_loc: &Point) -> Option<PreHandler> {
        for (_, obj) in self.level.cell_iter(new_loc) {
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
    fn try_interact(&mut self, player_loc: &Point, new_loc: &Point) -> PreResult {
        let handler = self.find_interact_handler(&Tag::Player, new_loc);
        if handler.is_some() {
            handler.unwrap()(self, player_loc, new_loc)
        } else {
            PreResult::DidntAct
        }
    }

    // Player interacting with a cell he has just moved into.
    fn interact_post_move(&mut self, new_loc: &Point) -> (Time, Sound) {
        let mut handlers = Vec::new();
        {
            let oids = self.level.cell(new_loc);
            for oid in oids.iter().rev() {
                let obj = self.level.obj(*oid).0;
                for tag in obj.iter() {
                    if let Some(handler) = self.interactions.find_post_handler(&Tag::Player, tag) {
                        handlers.push(*handler);
                    }
                }
            }
        }

        let mut duration = Time::zero();
        let mut volume = sound::NONE;
        for handler in handlers.into_iter() {
            let (d, v) = handler(self, new_loc);
            duration += d;
            volume += v;
        }
        (duration, volume)
    }

    fn add_object(&mut self, loc: &Point, obj: Object) {
        let behavior = obj.value(BEHAVIOR_ID);
        let scheduled = obj.has(SCHEDULED_ID) && !matches!(behavior, Some(Behavior::Sleeping));
        let oid = self.level.add(obj, Some(*loc));
        if scheduled {
            self.schedule_new_obj(oid);
        }
    }

    fn schedule_new_obj(&mut self, oid: Oid) {
        let obj = self.level.obj(oid).0;
        let terrain = object::terrain_value(obj);
        let initial = if oid.0 == 0 {
            time::DIAGNOL_MOVE
        } else if terrain.is_some()
            && (terrain.unwrap() == Terrain::ShallowWater || terrain.unwrap() == Terrain::DeepWater)
        {
            Time::zero() - ai::extra_flood_delay(self)
        } else {
            Time::zero()
        };
        self.scheduler.add(oid, initial);
    }

    fn replace_behavior(&mut self, loc: &Point, new_behavior: Behavior) {
        let (oid, obj) = self.level.get_mut(&loc, BEHAVIOR_ID).unwrap();
        let old_behavior = obj.value(BEHAVIOR_ID).unwrap();
        assert_ne!(old_behavior, new_behavior);
        obj.replace(Tag::Behavior(new_behavior));

        if obj.has(SCHEDULED_ID) {
            if let Behavior::Sleeping = old_behavior {
                self.scheduler.add(oid, Time::zero())
            }
            if let Behavior::Sleeping = new_behavior {
                self.scheduler.remove(oid);
            }
        }
    }

    fn destroy_object(&mut self, loc: &Point, old_oid: Oid) {
        let (obj, pos) = self.level.obj(old_oid);
        assert_eq!(pos.unwrap(), *loc);

        if obj.has(SCHEDULED_ID) {
            self.scheduler.remove(old_oid);
        }

        if let Some(terrain) = object::terrain_value(obj) {
            // Terrain cannot be destroyed but has to be mutated into something else.
            let new_obj = if terrain == Terrain::Wall {
                make::rubble()
            } else {
                error!("Need to better handle destroying Tid {obj}"); // Doors, trees, etc
                make::dirt()
            };
            let scheduled = new_obj.has(SCHEDULED_ID);
            let new_oid = self.level.replace(loc, old_oid, new_obj);
            if scheduled {
                self.schedule_new_obj(new_oid);
            }

            // The player may now be able to see through this cell so we need to ensure
            // that cells around it exist now. TODO: probably should have a LOS changed
            // check.
            self.level.ensure_neighbors(&loc);
        } else {
            // If it's just a normal object or character we can just nuke the object.
            self.level.remove(old_oid);
        }
    }

    fn replace_object(&mut self, loc: &Point, old_oid: Oid, new_obj: Object) {
        let (old_obj, pos) = self.level.obj(old_oid);
        assert_eq!(pos.unwrap(), *loc);

        trace!("replacing {old_obj} at {loc} with {new_obj}");
        if old_obj.has(SCHEDULED_ID) {
            self.scheduler.remove(old_oid);
        }

        let scheduled = new_obj.has(SCHEDULED_ID);
        let new_oid = self.level.replace(loc, old_oid, new_obj);
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
            let character = &self.level.get(&new_loc, CHARACTER_ID);
            if character.is_none() {
                let (_, terrain) = self.level.get_bottom(&new_loc);
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

    fn dump_cell<W: Write>(&self, writer: &mut W, loc: &Point) -> Result<(), Error> {
        for (oid, obj) in self.level.cell_iter(loc) {
            write!(writer, "   dname: {} oid: {oid}\n", obj.dname())?;
            for tag in obj.iter() {
                write!(writer, "   {tag:?}\n")?;
            }
            write!(writer, "\n")?;
        }
        Ok(())
    }

    fn dump_pov<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        // Find the dimensions of the player's pov.
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;
        for loc in self.pov.locations() {
            min_x = min(loc.x, min_x);
            min_y = min(loc.y, min_y);
            max_x = max(loc.x, max_x);
            max_y = max(loc.y, max_y);
        }

        // Render the PoV cells remembering which cells have characters and which have objects.
        let mut chars = Vec::new();
        let mut objs = Vec::new();
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let loc = Point::new(x, y);
                let obj = self.level.get_top(&loc).1;
                if obj.has(CHARACTER_ID) {
                    if chars.len() < 10 {
                        write!(writer, " c{}", chars.len())?;
                    } else {
                        write!(writer, "c{}", chars.len())?;
                    }
                    chars.push(loc);
                } else if !obj.has(TERRAIN_ID) {
                    write!(writer, "{:>3}", objs.len())?;
                    objs.push(loc);
                } else if self.pov.visible(self, &loc) {
                    write!(writer, "...")?;
                } else {
                    write!(writer, "   ")?;
                }
            }
            write!(writer, "\n")?;
        }

        // Write out details for each character and object.
        if !chars.is_empty() {
            write!(writer, "\n")?;
            for (i, loc) in chars.iter().enumerate() {
                write!(writer, "c{i} at {loc}:\n")?;
                self.dump_cell(writer, &loc)?;
            }
        }

        if !objs.is_empty() {
            write!(writer, "\n")?;
            for (i, loc) in objs.iter().enumerate() {
                write!(writer, "{i} at {loc}:\n")?;
                self.dump_cell(writer, &loc)?;
            }
        }

        Ok(())
    }
}

/// Returns a number with the standard normal distribution centered on x where the
/// values are all within +/- the given percentage.
fn rand_normal64(x: i64, percent: i32, rng: &RefCell<SmallRng>) -> i64 {
    assert!(percent > 0);
    assert!(percent <= 100);

    // Could use a generic for this but the type bounds get pretty gnarly.
    let rng = &mut *rng.borrow_mut();
    let scaling: f64 = rng.sample(StandardNormal); // ~95% are in -2..2
    let scaling = if scaling >= -2.0 && scaling <= 2.0 {
        scaling / 2.0 // all are in -1..1
    } else {
        0.0 // the few outliers are mapped to the mode
    };
    let scaling = scaling * (percent as f64) / 100.0; // all are in +/- percent
    let delta = (x as f64) * scaling; // all are in +/- percent of x
    x + (delta as i64)
}

fn rand_normal32(x: i32, percent: i32, rng: &RefCell<SmallRng>) -> i32 {
    rand_normal64(x as i64, percent, rng) as i32
}

struct InventoryIterator<'a> {
    game: &'a Game,
    oids: &'a Vec<Oid>,
    index: i32,
}

impl<'a> InventoryIterator<'a> {
    fn new(game: &'a Game, loc: &Point) -> InventoryIterator<'a> {
        let (_, inv) = game.level.get(loc, INVENTORY_ID).unwrap();
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
            Some((oid, self.game.level.obj(oid).0))
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
