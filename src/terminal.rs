//! Rendering and UI using termion terminal module.
mod color;
mod map;
mod messages;

use super::backend::{Game, Point, Size};
use map::MapView;
use messages::MessagesView;
use slog::Logger;
use std::io::{stdin, stdout, Write};
use std::panic;
use std::process;
use termion::input::TermRead; // for keys trait
use termion::raw::IntoRawMode;

const NUM_MESSAGES: i32 = 4;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum GameState {
    Running,
    Exiting,
}

pub struct Terminal {
    root_logger: Logger,
    game: Game,
    stdout: Box<dyn Write>,

    map: MapView,
    messages: MessagesView,
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

        let (width, height) = termion::terminal_size().expect("couldn't get terminal size");
        let width = width as i32;
        let height = height as i32;
        debug!(root_logger, "terminal size"; "width" => width, "height" => height);

        Terminal {
            root_logger,
            game,
            stdout: Box::new(stdout),
            map: MapView {
                origin: Point::new(0, 0),
                size: Size::new(width, height - NUM_MESSAGES),
            },
            messages: MessagesView {
                origin: Point::new(0, height - NUM_MESSAGES),
                size: Size::new(width, NUM_MESSAGES),
            },
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
        self.map.render(&mut self.stdout, &mut self.game);
        self.messages.render(&mut self.stdout, &self.game);
        self.stdout.flush().unwrap();
    }

    fn handle_input(&mut self, key: termion::event::Key) -> GameState {
        match key {
            termion::event::Key::Left => self.game.probe(-1, 0),
            termion::event::Key::Right => self.game.probe(1, 0),
            termion::event::Key::Up => self.game.probe(0, -1),
            termion::event::Key::Down => self.game.probe(0, 1),
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
