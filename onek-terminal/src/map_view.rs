use super::*;
use std::convert::From;
use std::io::Write;
use termion::{color, cursor, style};

/// Responsible for drawing the level, i.e. the terrain, characters, items, etc.
pub struct MapView {
    pub origin: Point,
    pub size: Size,
}

#[derive(Eq, PartialEq)]
pub enum RunTile {
    Visible { bg: Color, fg: Color, symbol: char },
    Stale { bg: Color, fg: Color, symbol: char },
    NotVisible,
}

// fn render_obj(obj: Object) -> (Color, Color, char) {
//     (
//         obj.get("back_color").unwrap().to_color(),
//         obj.get("color").unwrap().to_color(),
//         obj.get("symbol").unwrap().to_char(),
//     )
// }

fn render_cell(cell: Cell) -> (Color, Color, char) {
    let bg = cell[0].get("back_color").unwrap().to_color();
    let fg = cell[0].get("color").unwrap().to_color();
    let symbol = cell[0].get("symbol").unwrap().to_char();
    if cell.len() == 1 {
        (bg, fg, symbol)
    } else {
        // TODO: would be nice to do something extra when there are multiple objects
        let fg = cell.last().unwrap().get("color").unwrap().to_color();
        let symbol = cell.last().unwrap().get("symbol").unwrap().to_char();
        (bg, fg, symbol)
    }
}

impl From<Cell> for RunTile {
    fn from(cell: Cell) -> Self {
        match cell[0].get("id").unwrap().to_id().0.as_str() {
            "stale" => {
                let (bg, fg, symbol) = render_cell(cell);
                RunTile::Stale { bg, fg, symbol }
            }
            "unseen" => RunTile::NotVisible,
            _ => {
                let (bg, fg, symbol) = render_cell(cell);
                RunTile::Visible { bg, fg, symbol }
            }
        }
    }
}

// MapView::render is a major bottleneck so we go to some effort to ensure that it's as
// efficient as possible.
#[derive(Eq, PartialEq)]
pub struct Run {
    tile: RunTile,
    focused: bool,
}

impl MapView {
    pub fn render(&self, stdout: &mut Box<dyn Write>, ipc: &IPC, examined: Option<Point>) {
        let player_loc = ipc.get_player_loc();
        let start_loc = Point::new(player_loc.x - self.size.width / 2, player_loc.y - self.size.height / 2);
        for y in 0..self.size.height {
            let v = (self.origin.y + y + 1) as u16;
            let _ = write!(stdout, "{}", cursor::Goto(1, v),);

            let mut run = Run {
                tile: RunTile::NotVisible,
                focused: false,
            };
            let mut count = 0;
            for x in 0..self.size.width {
                let loc = Point::new(start_loc.x + x, start_loc.y + y);
                let cell = ipc.get_cell_at(loc);
                let candidate = Run {
                    tile: RunTile::from(cell),
                    focused: examined.map_or(false, |pt| loc == pt),
                };
                if candidate == run {
                    count += 1;
                } else {
                    self.render_run(stdout, &run, count);
                    run = candidate;
                    count = 1;
                }
            }
            if count > 0 {
                self.render_run(stdout, &run, count);
            }
        }
    }

    fn render_run(&self, stdout: &mut Box<dyn Write>, run: &Run, count: usize) {
        let (bg, fg, symbol) = match run.tile {
            RunTile::Visible {
                bg: b,
                fg: f,
                symbol: s,
            } => (b, f, s), // TODO: use black if there is a character or item?
            RunTile::Stale {
                bg: b,
                fg: f,
                symbol: s,
            } => (b, f, s), // TODO: use black if there is a character or item?
            RunTile::NotVisible => (Color::Black, Color::Black, ' '),
        };
        let text = symbol.to_string().repeat(count);
        if run.focused {
            let _ = write!(
                stdout,
                "{}{}{}{}{}",
                color::Bg(to_termion(bg)),
                color::Fg(to_termion(fg)),
                style::Invert,
                text,
                style::Reset
            );
        } else {
            let _ = write!(
                stdout,
                "{}{}{}",
                color::Bg(to_termion(bg)),
                color::Fg(to_termion(fg)),
                text
            );
        }
    }
}
