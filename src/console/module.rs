use super::color::*;
use crate::shared::*;
use crossterm::{
    cursor,
    event::{self, KeyEvent},
    style::{self, Stylize},
    terminal, QueueableCommand,
};
use std::fs::OpenOptions;
use std::io::{self, Write};

const MAX_MESSAGES: u16 = 6;

struct Console {
    game: Box<dyn Game>,
    running: bool,
}

pub fn new(game: Box<dyn Game>) -> Box<dyn UI> {
    let running = true;
    Box::new(Console { game, running })
}

// Note that there is support for non-blocking reads.
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
                event::Event::Resize(_, _) => (), // TODO make sure resizing terminal works ok
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
        let (width, height) = terminal::size().unwrap();
        self.render_map(width, height - MAX_MESSAGES)?;
        self.render_messages(width, height - MAX_MESSAGES)?;

        let mut stdout = io::stdout();
        stdout.flush()?;
        Ok(())
    }

    fn render_messages(&self, width: u16, start_y: u16) -> io::Result<()> {
        let messages = self.game.messages();
        let count = messages.len().min(MAX_MESSAGES as usize);
        let n = if messages.len() >= count {
            messages.len() - count
        } else {
            0
        };
        let mut v = start_y;
        let mut stdout = io::stdout();
        for m in messages.iter().skip(n) {
            stdout.queue(cursor::MoveTo(0, v))?;
            if m.text.chars().count() <= width as usize {
                let color = message_to_color(m);
                stdout.queue(style::PrintStyledContent(m.text[..].with(color)))?;
                let padding = " ".repeat(width as usize - m.text.chars().count());
                stdout.queue(style::PrintStyledContent(padding.black()))?;
            } else {
                stdout.queue(style::PrintStyledContent(m.text[..width as usize].black()))?;
            }
            v += 1;
        }
        let padding = " ".repeat(width as usize);
        for _ in 0..(MAX_MESSAGES as usize - count) {
            stdout.queue(cursor::MoveTo(0, v))?;
            stdout.queue(style::PrintStyledContent(padding[..].black()))?;
            v += 1;
        }
        Ok(())
    }

    fn render_map(&self, width: u16, height: u16) -> io::Result<()> {
        let ploc = self.game.player_loc();
        let dt = self.game.default();
        let dc = self.compose_tile(dt);
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
                Terrain::DeepWater => ('w', Color::CornflowerBlue, Color::Black),
                Terrain::Dirt => (' ', Color::White, Color::Black),
                Terrain::RockWall => ('#', Color::RosyBrown, Color::Black),
                Terrain::ShallowWater => ('~', Color::SteelBlue1, Color::Black),
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
            event::KeyCode::Char('d') => self.dump_game(),
            event::KeyCode::Char('q') => self.running = false,
            _ => info!("bad key: {key:?}"), // TODO beep
        }
    }

    fn move_player(&mut self, dx: i32, dy: i32) {
        // let (width, height) = terminal::size().unwrap();
        let delta = Point::new(dx, dy);
        self.game.execute(Command::Move(delta));
    }

    fn dump(&self) -> io::Result<()> {
        let args = SnapshotArgs::new();
        let text = self.game.snapshot(args);

        let mut file = OpenOptions::new().append(true).create(true).open("state.txt")?;
        file.write(text.as_bytes())?;
        file.write("-".repeat(80).as_bytes())?;
        file.write("\n".as_bytes())?;
        Ok(())
    }

    fn dump_game(&self) {
        if let Err(err) = self.dump() {
            error!("failed to write state: {err}"); // TODO also write to messages?
        }
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

fn message_to_color(message: &Message) -> style::Color {
    match message.kind {
        MessageKind::PlayerFailed => to_crossterm(Color::Red3),
    }
}
