use one_thousand_deaths::{Point, State};
use std::io::Write;
use termion::event::Key;

pub struct RenderContext<'a> {
    pub stdout: &'a mut Box<dyn Write>,
    pub state: &'a mut State,
    pub examined: Option<Point>, // ExamineWindow will set this
}

pub enum InputAction {
    UpdatedGame,
    Quit,
    // Push(Box<dyn Mode>),
    // Pop,
    NotHandled,
}

/// Modes are arranged in a stack and user input is directed to the topmost mode. Rendering
/// is also handled by modes although they may delegate rendering to a lower layer mode.
pub trait Mode {
    /// Windows are stacked in layers and will always handle input but may defer rendering
    /// to a lower layer. if a window does render it should return true, otherwise it
    /// should return false (and possibly augment context).
    fn render(&self, context: &mut RenderContext) -> bool;

    /// Normally this will return None so we'll block forever waiting for the player to
    /// press a key. But ReplayMode will set this to a smaller value so that Terminal can
    /// render the game without waiting for the player.
    fn input_timeout_ms(&self) -> Option<i32>;

    fn handle_input(&mut self, state: &mut State, key: Key) -> InputAction;

    fn replaying(&self) -> bool {
        false
    }
}
