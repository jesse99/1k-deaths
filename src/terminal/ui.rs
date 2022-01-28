use super::{Game, GameState, InputAction, MainMode, Mode, RenderContext};
use std::io::Write;

pub struct UI {
    windows: Vec<Box<dyn Mode>>,
}

impl UI {
    pub fn new(width: i32, height: i32) -> UI {
        let windows = vec![MainMode::window(width, height)];
        UI { windows }
    }

    pub fn render(&self, stdout: &mut Box<dyn Write>, game: &mut Game) {
        let mut context = RenderContext {
            stdout,
            game,
            examined: None,
        };
        for window in self.windows.iter().rev() {
            if window.render(&mut context) {
                return;
            }
        }
        panic!("No windows rendered!")
    }

    pub fn handle_input(&mut self, game: &mut Game, key: termion::event::Key) -> GameState {
        let window = self.windows.last_mut().unwrap();
        match window.handle_input(game, key) {
            InputAction::UpdatedGame => (),
            InputAction::Quit => return GameState::Exiting,
            InputAction::Push(window) => self.windows.push(window),
            InputAction::Pop => {
                let _ = self.windows.pop();
                assert!(!self.windows.is_empty());
            }
            InputAction::NotHandled => {
                debug!("player pressed {key:?}"); // TODO: beep?
            }
        }
        GameState::Running
    }
}
