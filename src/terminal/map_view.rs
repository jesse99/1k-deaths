use one_thousand_deaths::{render, Color, Point, Size, State};
use std::io::Write;
use termion::{color, cursor};

/// Responsible for drawing the level, i.e. the terrain, characters, items, etc.
pub struct MapView {
    pub origin: Point,
    pub size: Size,
}

// MapView::render is a major bottle-neck so we go to some effort to ensure that it's as
// efficient as possible.
#[derive(Eq, PartialEq)]
pub struct Run {
    // tile: Tile,
    focused: bool,
}

impl MapView {
    pub fn render(&self, stdout: &mut Box<dyn Write>, state: &mut State, _examined: Option<Point>) {
        let ploc = render::player_loc(state);
        let start_loc = Point::new(ploc.x - self.size.width / 2, ploc.y - self.size.height / 2);
        for y in 0..self.size.height {
            let v = (self.origin.y + y + 1) as u16;
            let _ = write!(stdout, "{}", cursor::Goto(1, v),);
            for x in 0..self.size.width {
                let pt = Point::new(start_loc.x + x, start_loc.y + y);
                let _ = if pt == ploc {
                    write!(
                        stdout,
                        "{}{}@",
                        color::Bg(super::color::to_termion(Color::Black)),
                        color::Fg(super::color::to_termion(Color::Blue)),
                    )
                } else if pt.y == 0 || pt.x == 0 {
                    // so we can see where the player is moving
                    write!(
                        stdout,
                        "{}{}.",
                        color::Bg(super::color::to_termion(Color::Black)),
                        color::Fg(super::color::to_termion(Color::Red)),
                    )
                } else {
                    write!(
                        stdout,
                        "{}{} ",
                        color::Bg(super::color::to_termion(Color::Black)),
                        color::Fg(super::color::to_termion(Color::Blue)),
                    )
                };
            }

            // let mut run = Run {
            //     tile: Tile::NotVisible,
            //     focused: false,
            // };
            // let mut count = 0;
            // for x in 0..self.size.width {
            //     let pt = Point::new(start_loc.x + x, start_loc.y + y);
            //     let candidate = Run {
            //         tile: state.tile(&pt),
            //         focused: examined.map_or(false, |loc| loc == pt),
            //     };
            //     if candidate == run {
            //         count += 1;
            //     } else {
            //         self.render_run(stdout, &run, count);
            //         run = candidate;
            //         count = 1;
            //     }
            // }
            // if count > 0 {
            //     self.render_run(stdout, &run, count);
            // }
        }
    }

    // fn render_run(&self, stdout: &mut Box<dyn Write>, run: &Run, count: usize) {
    //     let (bg, fg, symbol) = match run.tile {
    //         Tile::Visible {
    //             bg: b,
    //             fg: f,
    //             symbol: s,
    //         } => (b, f, s), // TODO: use black if there is a character or item?
    //         Tile::Stale(s) => (Color::LightGrey, Color::DarkGray, s),
    //         Tile::NotVisible => (Color::Black, Color::Black, Symbol::Unseen),
    //     };
    //     let text = self.symbols(symbol, count);
    //     if run.focused {
    //         let _ = write!(
    //             stdout,
    //             "{}{}{}{}{}",
    //             color::Bg(super::color::to_termion(bg)),
    //             color::Fg(super::color::to_termion(fg)),
    //             style::Invert,
    //             text,
    //             style::Reset
    //         );
    //     } else {
    //         let _ = write!(
    //             stdout,
    //             "{}{}{}",
    //             color::Bg(super::color::to_termion(bg)),
    //             color::Fg(super::color::to_termion(fg)),
    //             text
    //         );
    //     }
    // }

    // fn symbols(&self, symbol: Symbol, count: usize) -> String {
    //     use Symbol::*;
    //     match symbol {
    //         ClosedDoor => "+".repeat(count),
    //         DeepLiquid => "\u{224B}".repeat(count), // TRIPLE TILDE
    //         Dirt => ".".repeat(count),
    //         Npc(ch) => format!("{}", ch).repeat(count),
    //         OpenDoor => ":".repeat(count),
    //         PickAxe => "\u{26CF}".repeat(count), // pick
    //         Player => "\u{265D}".repeat(count),  // BLACK CHESS BISHOP
    //         Rubble => "\u{2237}".repeat(count),  // PROPORTION
    //         ShallowLiquid => "~".repeat(count),
    //         Armor => "\u{2720}".repeat(count),               // MALTESE CROSS
    //         Sign => "\u{261E}".repeat(count),                // WHITE RIGHT POINTING INDEX
    //         StrongSword => "\u{2694}\u{FE0F}".repeat(count), // crossed swords
    //         Tree => "\u{2B06}\u{FE0E}".repeat(count),        // UPWARDS BLACK ARROW
    //         Unseen => " ".repeat(count),
    //         Wall => "\u{25FC}\u{FE0E}".repeat(count), // BLACK MEDIUM SQUARE
    //         WeakSword => "\u{1F5E1}".repeat(count),   // dagger
    //     }
    // }
}
