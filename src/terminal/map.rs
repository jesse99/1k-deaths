use super::color;
use super::View;
use crate::backend::{Color, Game, Point, Terrain, Tile};
use std::io::Write;

/// Responsible for drawing the level, i.e. the terrain, characters, items, etc.
pub fn render(stdout: &mut Box<dyn Write>, view: &View, game: &mut Game) {
    let start_loc = Point::new(
        game.player().x - view.size.width / 2,
        game.player().y - view.size.height / 2,
    );
    for y in 0..view.size.height {
        for x in 0..view.size.width {
            let pt = Point::new(start_loc.x + x, start_loc.y + y);
            let h = (view.origin.x + x + 1) as u16; // termion is 1-based
            let v = (view.origin.y + y + 1) as u16;
            if pt == game.player() {
                let fg = Color::Blue;
                let bg = Color::Black;
                let _ = write!(
                    stdout,
                    "{}{}{}@",
                    termion::cursor::Goto(h, v),
                    termion::color::Bg(color::to_termion(bg)),
                    termion::color::Fg(color::to_termion(fg))
                );
            } else {
                let tile = game.tile(&pt);
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
                // TODO: use a function to return a tuple with sumbol, fg, and bg
                let _ = write!(
                    stdout,
                    "{}{}{}{}",
                    termion::cursor::Goto(h, v),
                    termion::color::Bg(color::to_termion(bg)),
                    termion::color::Fg(color::to_termion(fg)),
                    symbol
                );
            }
        }
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
        Terrain::ClosedDoor => Color::Green,
        Terrain::DeepWater => Color::Blue,
        Terrain::ShallowWater => Color::Blue,
        Terrain::Wall => Color::Chocolate,
        Terrain::Ground => Color::LightSlateGray,
    }
}
