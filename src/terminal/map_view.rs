use crate::backend::{Color, Game, Point, Size, Tile};
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
            game.player().x - self.size.width / 2,
            game.player().y - self.size.height / 2,
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
                    Tile::NotVisible => (Color::Black, Color::Black, ' '),
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
                if symbol == '#' {
                    // TODO: is this how we want to handle unicode?
                    // TODO: at least should have constants for these
                    let _ = write!(stdout, "\u{25FC}\u{FE0E}"); // BLACK MEDIUM SQUARE
                } else {
                    let _ = write!(stdout, "{}", symbol);
                }
                if focused {
                    let _ = write!(stdout, "{}", style::Reset);
                }
            }
        }
    }
}
