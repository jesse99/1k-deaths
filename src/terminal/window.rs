//! Trait used to render views and process user input. There are different window instances
//! for things like the main view and help screens.
use super::{Game, Point};
use std::io::Write;

pub struct RenderContext<'a> {
    pub stdout: &'a mut Box<dyn Write>,
    pub game: &'a mut Game,
    pub examined: Option<Point>, // ExamineWindow will set this
}

pub enum InputAction {
    UpdatedGame,
    Quit,
    Push(Box<dyn Window>),
    Pop,
    NotHandled,
}

pub trait Window {
    /// Windows are stacked in layers and will always handle input but may defer rendering
    /// to a lower layer. if a window does render it should return true, otherwise it
    /// should return false (and possibly augment context).
    fn render(&self, context: &mut RenderContext) -> bool;

    fn handle_input(&mut self, game: &mut Game, key: termion::event::Key) -> InputAction;
}
