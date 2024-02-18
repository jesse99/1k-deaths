use super::*;
use fnv::FnvHashMap;
use termion::event::Key;

pub enum CommandResult {
    UpdatedGame,
    Quit,

    /// This is used for transient modes, e.g. [`ExamineMode`].
    Push(Box<dyn Mode>),
    Pop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Command {
    Bump(i32, i32),
    Help,
    Page(i32),
    Scroll(i32),
    ScrollBy(i32),
    Quit,
}

pub type CommandTable = FnvHashMap<Key, Command>;
