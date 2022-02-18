use crate::backend::{Color, Game, Point, Size, Symbol, Tile};
use std::io::Write;
use termion::{color, cursor, style};

/// Responsible for drawing the level, i.e. the terrain, characters, items, etc.
pub struct MapView {
    pub origin: Point,
    pub size: Size,
}

impl MapView {
    pub fn render(&self, stdout: &mut Box<dyn Write>, game: &mut Game, examined: Option<Point>) {
        let start_loc = Point::new(
            game.player_loc().x - self.size.width / 2,
            game.player_loc().y - self.size.height / 2,
        );
        for y in 0..self.size.height {
            for x in 0..self.size.width {
                let pt = Point::new(start_loc.x + x, start_loc.y + y);
                let h = (self.origin.x + x + 1) as u16; // termion is 1-based
                let v = (self.origin.y + y + 1) as u16;
                let tile = game.tile(&pt);
                let (bg, fg, symbol) = match tile {
                    Tile::Visible {
                        bg: b,
                        fg: f,
                        symbol: s,
                    } => (b, f, s), // TODO: use black if there is a character or item?
                    Tile::Stale(s) => (Color::LightGrey, Color::DarkGray, s),
                    Tile::NotVisible => (Color::Black, Color::Black, Symbol::Unseen),
                };
                let _ = write!(
                    stdout,
                    "{}{}{}",
                    cursor::Goto(h, v),
                    color::Bg(super::color::to_termion(bg)),
                    color::Fg(super::color::to_termion(fg))
                );
                let focused = examined.map_or(false, |loc| loc == pt);
                if focused {
                    let _ = write!(stdout, "{}", style::Invert);
                }
                self.render_symbol(stdout, symbol);
                if focused {
                    let _ = write!(stdout, "{}", style::Reset);
                }
            }
        }
    }

    // let _ = write!(stdout, "\u{25FC}\u{FE0E}"); // BLACK MEDIUM SQUARE
    fn render_symbol(&self, stdout: &mut Box<dyn Write>, symbol: Symbol) {
        use Symbol::*;
        match symbol {
            ClosedDoor => {
                let _ = write!(stdout, "+");
            }
            DeepLiquid => {
                let _ = write!(stdout, "\u{224B}"); // TRIPLE TILDE
            }
            Dirt => {
                let _ = write!(stdout, ".");
            }
            Npc(ch) => {
                let _ = write!(stdout, "{}", ch);
            }
            OpenDoor => {
                let _ = write!(stdout, ":");
            }
            PickAxe => {
                let _ = write!(stdout, "\u{26CF}"); // pick
            }
            Player => {
                let _ = write!(stdout, "\u{265D}"); // BLACK CHESS BISHOP
            }
            Rubble => {
                let _ = write!(stdout, "\u{2237}"); // PROPORTION
            }
            ShallowLiquid => {
                let _ = write!(stdout, "~");
            }
            Sign => {
                let _ = write!(stdout, "\u{261E}"); // WHITE RIGHT POINTING INDEX
            }
            StrongSword => {
                let _ = write!(stdout, "\u{2694}\u{FE0F}"); // crossed swords
            }
            Tree => {
                let _ = write!(stdout, "\u{2B06}\u{FE0E}"); // UPWARDS BLACK ARROW
            }
            Unseen => {
                let _ = write!(stdout, " ");
            }
            Wall => {
                let _ = write!(stdout, "\u{25FC}\u{FE0E}"); // BLACK MEDIUM SQUARE
            }
            WeakSword => {
                let _ = write!(stdout, "\u{1F5E1}"); // dagger
            }
        }
    }
}
