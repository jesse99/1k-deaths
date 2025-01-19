use super::*;
// use std::convert::From;
use std::io::Write;
use termion::{color, cursor};
// use termion::{style};
use fnv::FnvHashMap;
use std::cell::RefCell;

/// Responsible for drawing the level, i.e. the terrain, characters, items, etc.
pub struct MapView {
    pub origin: Point,
    pub size: Size,
    pub string_cache: RefCell<FnvHashMap<(char, usize), String>>,
}

impl MapView {
    pub fn render(&self, stdout: &mut Box<dyn Write>, ipc: &IPC, _examined: Option<Point>) {
        let player_loc = ipc.get_player_loc();
        let start_loc = Point::new(player_loc.x - self.size.width / 2, player_loc.y - self.size.height / 2);
        for y in 0..self.size.height {
            let v = (self.origin.y + y + 1) as u16;
            let _ = write!(stdout, "{}", cursor::Goto(1, v));

            // TODO: need to invert examined cell
            let row = ipc.get_terminal_row(Point::new(start_loc.x, start_loc.y + y), self.size.width);
            for (cell, len) in row.row {
                self.render_run(stdout, cell, len as usize);
            }
        }
    }

    fn render_run(&self, stdout: &mut Box<dyn Write>, cell: TerminalCell, count: usize) {
        let symbol = match cell {
            TerminalCell::Seen {
                symbol,
                color: _,
                back_color: _,
            } => symbol,
            TerminalCell::Stale { symbol, back_color: _ } => symbol,
            TerminalCell::Unseen => ' ',
        };
        let mut cache = self.string_cache.borrow_mut(); // not clear how much this helps
        let text = cache
            .entry((symbol, count))
            .or_insert_with(|| symbol.to_string().repeat(count));

        let _ = match cell {
            TerminalCell::Seen {
                symbol: _,
                color,
                back_color,
            } => write!(
                stdout,
                "{}{}{}",
                color::Bg(to_termion(back_color)),
                color::Fg(to_termion(color)),
                text
            ),
            TerminalCell::Stale { symbol: _, back_color } => write!(
                stdout,
                "{}{}{}",
                color::Bg(to_termion(back_color)),
                color::Fg(to_termion(Color::Gray)),
                text
            ),
            TerminalCell::Unseen => write!(
                stdout,
                "{}{}{}",
                color::Bg(to_termion(Color::Black)),
                color::Fg(to_termion(Color::Black)),
                text
            ),
        };
    }
}
