//! Rendering and UI using termion terminal module.
mod color;

use super::backend::{Color, Game, Point, Terrain, Tile};
use slog::Logger;
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
        let width = width as i32;
        let height = height as i32;

        let start_loc = Point::new(
            self.game.player().x - width / 2,
            self.game.player().y - height / 2,
        );
        for y in 0..height {
            for x in 0..width {
                let pt = Point::new(start_loc.x + x, start_loc.y + y);
                if pt == self.game.player() {
                    let color = termion::color::AnsiValue::rgb(0, 0, 4);
                    let _ = write!(
                        self.stdout,
                        "{}{}@",
                        termion::cursor::Goto(x as u16 + 1, y as u16 + 1), // termion is 1-based
                        // termion::color::Bg(view.bg),
                        termion::color::Fg(color)
                    );
                } else {
                    let tile = self.game.tile(&pt);
                    let bg = match tile {
                        Tile::Visible(terrain) => to_back_color(terrain), // TODO: use black if there is a character or item?
                        Tile::Stale(_terrain) => Color::LightGrey,
                        Tile::NotVisible => Color::Black,
                    };
                    let fg = match tile {
                        Tile::Visible(terrain) => to_fore_color(terrain),
                        Tile::Stale(_terrain) => Color::DarkGray,
                        Tile::NotVisible => Color::Black,
                    };
                    let symbol = match tile {
                        Tile::Visible(terrain) => to_symbol(terrain),
                        Tile::Stale(terrain) => to_symbol(terrain),
                        Tile::NotVisible => ' ',
                    };
                    let _ = write!(
                        self.stdout,
                        "{}{}{}{}",
                        termion::cursor::Goto(x as u16 + 1, y as u16 + 1), // termion is 1-based
                        termion::color::Bg(color::to_termion(bg)),
                        termion::color::Fg(color::to_termion(fg)),
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

fn to_symbol(terrain: Terrain) -> char {
    match terrain {
        Terrain::ClosedDoor => '+',
        Terrain::DeepWater => 'W',
        Terrain::ShallowWater => '~',
        Terrain::Wall => '#',
        Terrain::Ground => '.',
    }
}

fn to_back_color(terrain: Terrain) -> Color {
    match terrain {
        Terrain::ClosedDoor => Color::Black,
        Terrain::DeepWater => Color::LightBlue,
        Terrain::ShallowWater => Color::LightBlue,
        Terrain::Wall => Color::Black,
        Terrain::Ground => Color::Black,
    }
}

fn to_fore_color(terrain: Terrain) -> Color {
    match terrain {
        Terrain::ClosedDoor => Color::Red,
        Terrain::DeepWater => Color::Blue,
        Terrain::ShallowWater => Color::Blue,
        Terrain::Wall => Color::Chocolate,
        Terrain::Ground => Color::LightSlateGray,
    }
}
