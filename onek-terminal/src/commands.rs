use super::*;
use fnv::FnvHashMap;
use serde::{Deserialize, Serialize};
use termion::event::Key;

pub enum CommandResult {
    /// Used to push a transient mode (e.g. [`ExamineMode`]) onto a [`Window`].
    Push(Box<dyn Mode>),

    /// Used to pop a transient mode from a [`Window`].
    Pop,

    /// Exit a transient mode or the game itself.
    Quit,

    /// Command mutated the backend.
    UpdatedGame,
}

/// Key strokes are mapped to commands which are then persisted and executed. Some
/// commands (like Scroll) apply to the UI. Others (like Bump) will mutate the backend.
///
/// Note that the more efficient serialization backends require new enum variants to be
/// added to the end in order to avoid breaking deserialization.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Command {
    /// Move player or interact with adjacent object (e.g. opening a door).
    Bump(i32, i32),

    /// Show help for a mode.
    Help,

    /// Scroll a [`TextMode`] up or down by a page.
    Page(i32),

    /// Scroll a [`TextMode`] up or down N lines.
    Scroll(i32),

    /// Scroll a [`TextMode`] up or down by a multiple of N lines where the multiple
    /// defaults to 1.
    ScrollBy(i32),

    /// Exit a transient mode or the game itself.
    Quit,
}

pub type CommandTable = FnvHashMap<Key, Command>;
