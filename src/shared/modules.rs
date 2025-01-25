//! Traits used to wrap modules. This is essentially a modular monolith design, see
//! https://www.geeksforgeeks.org/what-is-a-modular-monolith for more.
use super::Point;
use std::collections::VecDeque;
use std::io;

#[derive(Debug)]
pub enum MessageKind {
    /// Player tried to do something but failed, e.g. move into a wall.
    PlayerFailed,
}

#[derive(Debug)]
pub struct Message {
    pub kind: MessageKind,
    pub text: String,
}

#[derive(Debug)]
pub enum Command {
    /// Typically this will be a move to an adjacent cell bu something like a chage can
    /// be used to move multiple cells in one go.
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
    Human,
}

#[derive(Copy, Clone)]
pub enum Terrain {
    Dirt,
    RockWall,
}

pub struct Tile {
    pub terrain: Terrain,
    pub items: Vec<Item>,
    pub character: Option<Species>,
    // equipped: Vec<Item>,    // TODO might also want something like aura
}

pub trait Game {
    fn player_loc(&self) -> Point;

    fn execute(&mut self, command: Command);

    fn tile(&self, loc: Point) -> Option<Tile>;

    /// Returns oldest to newest messages.
    fn messages(&self) -> &VecDeque<Message>;

    /// The tile to use when the tile function returns None.
    fn default(&self) -> &Tile;
}

pub trait UI {
    fn run(&mut self) -> io::Result<()>;
}
