//! Rendering and UI using termion terminal module.
use super::backend::{Game, Point, Terrain};
use slog::Logger;
use std::cmp::min;
use std::io::{stdin, stdout, Write};
use std::panic;
use std::process;
use termion::input::TermRead; // for keys trait
use termion::raw::IntoRawMode;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum GameState {
    Running,
    Exiting,
}

pub struct Terminal {
    root_logger: Logger,
    game: Game,
    stdout: Box<dyn Write>,
}

impl Terminal {
    pub fn new(root_logger: Logger, game: Game) -> Terminal {
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

        Terminal {
            root_logger,
            game,
            stdout: Box::new(stdout),
        }
    }

    pub fn run(&mut self) {
        let stdin = stdin();
        let stdin = stdin.lock();
        let mut key_iter = stdin.keys();
        let mut state = GameState::Running;
        while state != GameState::Exiting {
            self.render();
            if let Some(c) = key_iter.next() {
                let c = c.unwrap();
                debug!(self.root_logger, "input"; "key" => ?c);
                state = self.handle_input(c);
            } else {
                panic!("Couldn't read the next key");
            }
        }
    }

    fn render(&mut self) {
        let (width, height) = termion::terminal_size().expect("couldn't get terminal size");
        for y in 0..min(self.game.height() as u16, height) {
            for x in 0..min(self.game.width() as u16, width) {
                let pt = Point::new(x as i32, y as i32);
                if pt == self.game.player() {
                    let color = termion::color::AnsiValue::rgb(0, 0, 4);
                    let _ = write!(
                        self.stdout,
                        "{}{}@",
                        termion::cursor::Goto(x + 1, y + 1), // termion is 1-based
                        // termion::color::Bg(view.bg),
                        termion::color::Fg(color)
                    );
                } else {
                    let color = termion::color::AnsiValue::grayscale(0);
                    let symbol = match self.game.terrain(&pt) {
                        Terrain::DeepWater => "W",
                        Terrain::ShallowWater => "w",
                        Terrain::Wall => "#",
                        Terrain::Ground => ".",
                    };
                    let _ = write!(
                        self.stdout,
                        "{}{}{}",
                        termion::cursor::Goto(x + 1, y + 1), // termion is 1-based
                        // termion::color::Bg(view.bg),
                        termion::color::Fg(color),
                        symbol
                    );
                }
            }
        }
        self.stdout.flush().unwrap();
    }

    fn handle_input(&mut self, key: termion::event::Key) -> GameState {
        match key {
            termion::event::Key::Left => self.game.move_player(-1, 0),
            termion::event::Key::Right => self.game.move_player(1, 0),
            termion::event::Key::Up => self.game.move_player(0, -1),
            termion::event::Key::Down => self.game.move_player(0, 1),
            termion::event::Key::Char('q') => return GameState::Exiting,
            _ => (),
        };
        GameState::Running
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
