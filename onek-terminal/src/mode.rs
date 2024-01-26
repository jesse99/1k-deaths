use super::*;
use std::io::Write;
use termion::event::Key;

/// Modes are arranged in a stack and user input is directed to the topmost mode. Rendering
/// is also handled by modes although they may delegate rendering to a lower layer mode.
pub trait Mode {
    /// Modes are stacked in layers and will always handle input but may defer rendering
    /// to a lower layer mode. If a mode does render it should return true, otherwise
    /// it should return false (and possibly augment context).
    fn render(&self, context: &mut RenderContext) -> bool;

    fn handle_input(&mut self, ipc: &IPC, key: Key) -> InputAction;

    /// Normally this will return None so we'll block forever waiting for the player to
    /// press a key. But ReplayMode will set this to a smaller value so that Terminal can
    /// render the game without waiting for the player.
    fn input_timeout_ms(&self) -> Option<i32>; // TODO: probably better to use a ms newtype

    fn replaying(&self) -> bool {
        false
    }
}
pub struct RenderContext<'a> {
    pub stdout: &'a mut Box<dyn Write>,
    pub ipc: &'a IPC,
    pub examined: Option<Point>, // ExamineMode will set this
}

pub enum InputAction {
    UpdatedGame,
    Quit,

    /// This is used for transient modes, e.g. [`ExamineMode`].
    Push(Box<dyn Mode>),
    Pop,

    NotHandled,
}
