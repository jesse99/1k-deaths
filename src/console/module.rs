use super::color::*;
use crate::shared::*;
use crossterm::{
    cursor,
    event::{self, KeyEvent},
    style::{self, Stylize},
    terminal, QueueableCommand,
};
use std::io::{self, Write};

struct Console {
    game: Box<dyn Backend>,
    running: bool,
}

pub fn new(game: Box<dyn Backend>) -> Box<dyn UI> {
    let running = true;
    Box::new(Console { game, running })
}

// Note that there is support for non-blocking reads
impl UI for Console {
    fn run(&mut self) -> io::Result<()> {
        self.setup()?;
        while self.running {
            self.render()?;

            let event = event::read()?;
            match event {
                event::Event::FocusGained => (),
                event::Event::FocusLost => (),
                event::Event::Key(e) => self.handle_key(e),
                event::Event::Mouse(_) => (),
                event::Event::Paste(_) => (),
                event::Event::Resize(_, _) => (),
            }
        }
        Ok(())
    }
}

impl Console {
    fn setup(&self) -> io::Result<()> {
        terminal::enable_raw_mode()?;

        let mut stdout = io::stdout();
        stdout.queue(cursor::Hide {})?;
        Ok(())
    }

    // TODO don't allow user to move off screen
    // TODO print a message if user moves off screen
    fn render(&self) -> io::Result<()> {
        let mut stdout = io::stdout();

        let ploc = self.game.player_loc();
        let dt = self.game.default();
        let dc = self.compose_tile(dt);
        let (width, height) = terminal::size().unwrap();
        for v in 0..height {
            for h in 0..width {
                let dx = h as i32 - (width / 2) as i32;
                let dy = v as i32 - (height / 2) as i32;
                let loc = Point::new(ploc.x + dx, ploc.y + dy);
                if let Some(tile) = &self.game.tile(loc) {
                    let composed = self.compose_tile(tile);
                    self.render_tile(h, v, composed)?;
                } else {
                    self.render_tile(h, v, dc)?;
                }
            }
        }
        stdout.flush()?;
        Ok(())
    }

    fn render_tile(&self, h: u16, v: u16, composed: (char, Color, Color)) -> io::Result<()> {
        let mut stdout = io::stdout();
        stdout.queue(cursor::MoveTo(h, v))?;
        stdout.queue(style::PrintStyledContent(
            composed.0.with(to_crossterm(composed.1)).on(to_crossterm(composed.2)),
        ))?;
        Ok(())
    }

    fn compose_tile(&self, tile: &Tile) -> (char, Color, Color) {
        if tile.character.is_some() {
            ('@', Color::Yellow, Color::Black)
        } else {
            match tile.terrain {
                Terrain::Dirt => (' ', Color::White, Color::Black),
                Terrain::RockWall => ('#', Color::RosyBrown, Color::Black),
            }
        }
    }

    // TODO can special case keypad using https://sw.kovidgoyal.net/kitty/keyboard-protocol/#progressive-enhancement
    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            event::KeyCode::Left | event::KeyCode::Char('4') => self.move_player(-1, 0),
            event::KeyCode::Right | event::KeyCode::Char('6') => self.move_player(1, 0),
            event::KeyCode::Up | event::KeyCode::Char('8') => self.move_player(0, -1),
            event::KeyCode::Down | event::KeyCode::Char('2') => self.move_player(0, 1),
            event::KeyCode::Char('q') => self.running = false,
            _ => (), // TODO beep
        }
    }

    fn move_player(&mut self, dx: i32, dy: i32) {
        // let (width, height) = terminal::size().unwrap();
        let delta = Point::new(dx, dy);
        self.game.execute(Command::Move(delta));
    }
}

// This restores the terminal state when the process exits but unfortunately it's called
// after panic! backtraces are printed so they are completely mis-formatted. TODO: there
// is a solution in https://werat.dev/blog/pretty-rust-backtraces-in-raw-terminal-mode
// but it's rather messy.
impl Drop for Console {
    fn drop(&mut self) {
        let mut stdout = io::stdout();
        let _ = stdout.queue(cursor::Show {});

        let _ = terminal::disable_raw_mode();
    }
}
