//! Rendering and UI using termion terminal module.
mod color;
mod examine_mode;
mod main_mode;
mod map_view;
mod messages_view;
mod mode;
mod replay_mode;
mod text;
mod ui;

// TODO: I think we can do better with these. Some are here only for sub-modules (which can
// pull them in with somewhat longer paths). May also be able to leverage the pub modifiers.
// See https://doc.rust-lang.org/stable/rust-by-example/mod/visibility.html
// Backend could do similar things.
use super::backend::{Command, Event, Game, Point, Size};
use main_mode::MainMode;
use map_view::MapView;
use messages_view::MessagesView;
use mode::{InputAction, Mode, RenderContext};
use replay_mode::ReplayMode;
use std::io::{stdout, Write};
use std::process;
use termion::raw::IntoRawMode;
use ui::UI;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum GameState {
    // TODO: do we really need this?
    Running,
    Exiting,
}

pub struct Terminal {
    ui: UI,
    game: Game,
    stdout: Box<dyn Write>,
}

impl Terminal {
    pub fn new(mut game: Game, replay: Vec<Event>) -> Terminal {
        let stdout = stdout();
        let mut stdout = stdout.into_raw_mode().unwrap();
        write!(
            stdout,
            "{}{}{}",
            termion::style::Reset,
            termion::cursor::Hide,
            termion::clear::All
        )
        .unwrap();

        let (width, height) = termion::terminal_size().expect("couldn't get terminal size");
        let width = width as i32;
        let height = height as i32;
        info!("terminal size is {} x {}", width, height);

        Terminal {
            ui: UI::new(width, height, &mut game, replay),
            game,
            stdout: Box::new(stdout),
        }
    }

    pub fn run(&mut self) {
        let mut state = GameState::Running;

        while state != GameState::Exiting {
            self.render();
            state = self.ui.handle_input(&mut self.game);
        }
    }

    fn render(&mut self) {
        self.ui.render(&mut self.stdout, &mut self.game);
        self.stdout.flush().unwrap();
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = write!(
            self.stdout,
            "{}{}{}{}",
            termion::style::Reset,
            termion::cursor::Restore,
            termion::cursor::Show,
            termion::cursor::Goto(1, 1)
        );
        let _ = write!(self.stdout, "{}", termion::clear::All);
        self.stdout.flush().unwrap();
        let _ = process::Command::new("reset").output(); // new line mode isn't reset w/o this
    }
}
