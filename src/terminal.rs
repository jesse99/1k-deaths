mod color;
mod help;
mod main_mode;
mod map_view;
mod mode;
mod text_mode;
mod text_view;
mod ui;

use crate::backend::Game;
use color::*;
use help::*;
use main_mode::*;
use map_view::*;
use mode::*;
use std::io::{self, Write};
use std::process;
use termion::raw::IntoRawMode;
use text_mode::*;
use text_view::*;
use ui::*;

pub struct Terminal {
    ui: UI,
    game: Game,
    stdout: Box<dyn Write>,
}

impl Terminal {
    pub fn new(game: Game) -> Terminal {
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
            game,
            stdout: Box::new(stdout),
        }
    }

    pub fn run(&mut self) {
        loop {
            self.render();
            if self.ui.handle_input(&mut self.stdout, &mut self.game) != LifeCycle::Running {
                break;
            }
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
