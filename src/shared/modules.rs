//! Traits used to wrap modules. This is essentially a modular monolith design, see
//! https://www.geeksforgeeks.org/what-is-a-modular-monolith for more.
use super::Point;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt;
use std::fmt::Display;
use std::io;

#[derive(Debug)]
pub enum MessageKind {
    /// Player tried to do something but failed, e.g. move into a wall.
    PlayerFailed,
    // /// Operation failed.
    // Error,

    // /// Player is near death, special message when entering a new level, etc.
    // Critical,

    // // Player took a critical hit, buff is wearing off, etc.
    // Important,

    // // Relatively spammy messages, e.g. player was hit.
    // #[default]
    // Normal,

    // // Messages that are not normally shown.
    // Debug,
}

#[derive(Debug)]
pub struct Message {
    pub kind: MessageKind,
    pub text: String,
}

/// Represents what the player wants to do next. Most of these will use up the player's
/// remaining time units, but some like (Examine) don't take any time.
#[derive(Debug)]
pub enum Command {
    /// Typically this will be a move to an adjacent cell but something like a charge can
    /// be used to move multiple cells in one go. TODO: probably need to also use this
    /// for interacting with objects: may want to rename it bump
    Move(Point),
}

#[derive(Copy, Clone)]
pub enum Item {
    Axe,
    Sword,
}

#[derive(Copy, Clone)]
pub enum Species {
    Ay,
    Bhederin,
    Human,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum Terrain {
    DeepWater,
    Dirt,

    #[default]
    RockWall,
    ShallowWater,
}

pub struct Tile {
    pub terrain: Terrain,
    pub items: Vec<Item>,
    pub character: Option<Species>,
    // equipped: Vec<Item>,    // TODO might also want something like aura
}

pub struct SnapshotArgs {
    /// Include cells within radius of the player. Defaults to 5.
    pub radius: i32, // TODO: add include_map_details, eg positions for characters, maybe make this an i32 for extra details

    /// Include the last N messages. Defaults to 4.
    pub num_messages: i32,
}

impl SnapshotArgs {
    pub fn new() -> SnapshotArgs {
        SnapshotArgs {
            radius: 5,
            num_messages: 4,
        }
    }
}

pub trait Game {
    fn player_loc(&self) -> Point;

    /// If this returns true then the UI should call player_action, otherwise the UI should
    /// call other_actions.
    fn players_turn(&self) -> bool;

    fn player_action(&mut self, command: Command);

    /// Allow all objects that are ready to act a chance to act. Then advance time and
    /// continue until the player accumulates enough time to act.
    fn other_actions(&mut self, replay: bool);

    fn tile(&self, loc: Point) -> Option<Tile>;

    /// Returns oldest to newest messages.
    fn messages(&self) -> &VecDeque<Message>;

    /// The tile to use when the tile function returns None.
    fn default(&self) -> Tile;

    /// Returns game state.
    fn snapshot(&self, args: SnapshotArgs) -> String;
}

pub trait UI {
    fn run(&mut self) -> io::Result<()>;
}

impl Display for Terrain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
