use super::Color;
use crate::backend::{Character, Game, Point, Size, Terrain, Tile};
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

fn terrain_to_fg(terrain: Terrain) -> Color {
    match terrain {
        Terrain::ClosedDoor => Color::Yellow,
        Terrain::DeepWater => Color::Blue,
        Terrain::Dirt => Color::LightSlateGray,
        Terrain::OpenDoor => Color::Yellow,
        Terrain::Rubble => Color::Chocolate,
        Terrain::ShallowWater => Color::Blue,
        Terrain::Tree => Color::ForestGreen,
        Terrain::Vitr => Color::Gold,
        Terrain::Wall => Color::Chocolate,
    }
}

fn terrain_to_bg(terrain: Terrain) -> Color {
    match terrain {
        Terrain::ClosedDoor => Color::Black,
        Terrain::DeepWater => Color::LightBlue,
        Terrain::Dirt => Color::Black,
        Terrain::OpenDoor => Color::Black,
        Terrain::Rubble => Color::Black,
        Terrain::ShallowWater => Color::LightBlue,
        Terrain::Tree => Color::Black,
        Terrain::Vitr => Color::Black,
        Terrain::Wall => Color::Black,
    }
}

fn terrain_to_symbol(terrain: Terrain) -> char {
    match terrain {
        Terrain::ClosedDoor => '+',
        Terrain::DeepWater => 'W',
        Terrain::Dirt => '.',
        Terrain::OpenDoor => '-',
        Terrain::Rubble => '…',
        Terrain::ShallowWater => '~',
        Terrain::Tree => 'T',
        Terrain::Vitr => 'V',
        Terrain::Wall => '#',
    }
}

fn character_to_fg(character: Character) -> Color {
    match character {
        Character::Guard => Color::SandyBrown,
        Character::Player => Color::Linen,
    }
}

fn character_to_symbol(character: Character) -> char {
    match character {
        Character::Guard => 'G',
        Character::Player => '@',
    }
}

impl From<Tile> for RunTile {
    fn from(tile: Tile) -> Self {
        match tile {
            Tile::Visible(content) => {
                let bg = terrain_to_bg(content.terrain);
                let (fg, symbol) = if let Some(character) = content.character {
                    (character_to_fg(character), character_to_symbol(character))
                } else {
                    (terrain_to_fg(content.terrain), terrain_to_symbol(content.terrain))
                };
                RunTile::Visible { bg, fg, symbol }
            }
            Tile::Stale(content) => {
                let bg = Color::LightGrey;
                let (fg, symbol) = if let Some(character) = content.character {
                    (character_to_fg(character), character_to_symbol(character))
                } else {
                    (terrain_to_fg(content.terrain), terrain_to_symbol(content.terrain))
                };
                RunTile::Visible { bg, fg, symbol }
            }
            Tile::NotVisible => {
                let bg = Color::Black;
                let fg = Color::Black;
                let symbol = ' ';
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
    pub fn render(&self, stdout: &mut Box<dyn Write>, game: &mut Game, examined: Option<Point>) {
        let start_loc = Point::new(
            game.player_loc().x - self.size.width / 2,
            game.player_loc().y - self.size.height / 2,
        );
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
                let candidate = Run {
                    tile: RunTile::from(game.tile(loc)),
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
                color::Bg(super::color::to_termion(bg)),
                color::Fg(super::color::to_termion(fg)),
                style::Invert,
                text,
                style::Reset
            );
        } else {
            let _ = write!(
                stdout,
                "{}{}{}",
                color::Bg(super::color::to_termion(bg)),
                color::Fg(super::color::to_termion(fg)),
                text
            );
        }
    }
}
