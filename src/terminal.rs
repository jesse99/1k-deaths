//! Rendering and UI using termion terminal module.
mod color;
mod main_mode;
mod map_view;
mod mode;
mod ui;

use one_thousand_deaths::State;
use std::io::{self, Write};
use std::process;
use termion::raw::IntoRawMode;
use ui::UI;

pub struct Terminal {
    ui: UI,
    state: State,
    stdout: Box<dyn Write>,
}

/// Note that the player can continue to do things even after he dies (although he can't
/// do anything that would change game state)
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GameLoop {
    Running,
    Quitting,
}

impl Terminal {
    /// Used for brand new games.
    pub fn new(state: State) -> Terminal {
        let stdout = io::stdout();
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
            ui: UI::new(width, height),
            state,
            stdout: Box::new(stdout),
        }
    }

    pub fn run(&mut self) {
        let mut state = GameLoop::Running;

        while state != GameLoop::Quitting {
            self.render();
            state = self.ui.handle_input(&mut self.stdout, &mut self.state);
            // if self.game.players_turn() {
            //     state = self.ui.handle_input(&mut self.stdout, &mut self.game);
            // } else {
            //     let replaying = self.ui.replaying();
            //     self.game.advance_time(replaying);
            // }
        }
    }

    fn render(&mut self) {
        self.ui.render(&mut self.stdout, &mut self.state);
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
