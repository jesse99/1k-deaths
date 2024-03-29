use super::color;
use one_thousand_deaths::{Color, Game, InvItem, ItemKind, Point, Size, Slot};
use std::borrow::Cow;
use std::io::Write;

const WIDTH: u16 = 30;
/// Shows info about the player and nearby NPCs.
pub struct InventoryView {
    pub origin: Point,
    pub size: Size,
}

impl InventoryView {
    pub fn render(&self, sindex: Option<usize>, stdout: &mut Box<dyn Write>, game: &Game, desc: Vec<String>) {
        let h = (self.origin.x + 1) as u16; // termion is 1-based
        let mut v = 1;
        self.render_background(stdout);

        let inv = game.inventory();
        let desc_height = if desc.is_empty() { 0 } else { desc.len() as u16 + 1 };
        self.render_weapons(&inv, sindex, h, &mut v, stdout, desc_height);

        v += 1;
        self.render_armor(&inv, sindex, h, &mut v, stdout, desc_height);

        v = 1;
        self.render_other(&inv, sindex, h + WIDTH + 1, &mut v, stdout, desc_height);

        self.render_desc(desc, stdout);
    }

    fn render_desc(&self, desc: Vec<String>, stdout: &mut Box<dyn Write>) {
        let first = self.size.height as u16 - desc.len() as u16;
        for (i, s) in desc.iter().enumerate() {
            let _ = write!(
                stdout,
                "{}{}{}{}",
                termion::cursor::Goto(2, first + i as u16 + 1),
                termion::color::Bg(color::to_termion(Color::Black)),
                termion::color::Fg(color::to_termion(Color::White)),
                s,
            );
        }
    }

    fn render_weapons(
        &self,
        inv: &Vec<InvItem>,
        sindex: Option<usize>,
        h: u16,
        v: &mut u16,
        stdout: &mut Box<dyn Write>,
        desc_height: u16,
    ) {
        let _ = write!(
            stdout,
            "{}{}{}Weapons:",
            termion::cursor::Goto(h, *v),
            termion::color::Bg(color::to_termion(Color::Black)),
            termion::color::Fg(color::to_termion(Color::Yellow)),
        );
        *v += 1;

        for (i, item) in inv.iter().enumerate() {
            if matches!(item.kind, ItemKind::TwoHandWeapon) {
                let selected = Some(i) == sindex;
                self.render_item(item, selected, "both hands", h, *v, stdout, WIDTH);
                *v += 1;
            } else if matches!(item.kind, ItemKind::OneHandWeapon) {
                let selected = Some(i) == sindex;
                if item.equipped == Some(Slot::MainHand) {
                    self.render_item(item, selected, "main hand", h, *v, stdout, WIDTH);
                } else {
                    self.render_item(item, selected, "off hand", h, *v, stdout, WIDTH);
                }
                *v += 1;
            }
            if *v >= self.size.height as u16 - desc_height {
                break;
            }
        }
    }

    fn render_armor(
        &self,
        inv: &Vec<InvItem>,
        sindex: Option<usize>,
        h: u16,
        v: &mut u16,
        stdout: &mut Box<dyn Write>,
        desc_height: u16,
    ) {
        let _ = write!(
            stdout,
            "{}{}{}Armor:",
            termion::cursor::Goto(h, *v),
            termion::color::Bg(color::to_termion(Color::Black)),
            termion::color::Fg(color::to_termion(Color::Yellow)),
        );
        *v += 1;

        for (i, item) in inv.iter().enumerate() {
            if matches!(item.kind, ItemKind::Armor) {
                let selected = Some(i) == sindex;
                self.render_item(item, selected, "worn", h, *v, stdout, WIDTH);
                *v += 1;

                if *v >= self.size.height as u16 - desc_height {
                    break;
                }
            }
        }
    }

    fn render_other(
        &self,
        inv: &Vec<InvItem>,
        sindex: Option<usize>,
        h: u16,
        v: &mut u16,
        stdout: &mut Box<dyn Write>,
        desc_height: u16,
    ) {
        let _ = write!(
            stdout,
            "{}{}{}Other:",
            termion::cursor::Goto(h, *v),
            termion::color::Bg(color::to_termion(Color::Black)),
            termion::color::Fg(color::to_termion(Color::Yellow)),
        );
        *v += 1;

        let max_width = (self.size.width as u16) - WIDTH - h;
        for (i, item) in inv.iter().enumerate() {
            if matches!(item.kind, ItemKind::Other) {
                let selected = Some(i) == sindex;
                self.render_item(item, selected, "worn", h, *v, stdout, max_width);
                *v += 1;

                if *v >= self.size.height as u16 - desc_height {
                    break;
                }
            }
        }
    }

    fn render_item(
        &self,
        item: &InvItem,
        selected: bool,
        etext: &str,
        h: u16,
        v: u16,
        stdout: &mut Box<dyn Write>,
        max_width: u16,
    ) {
        let text = if item.equipped.is_some() {
            format!("{} ({etext})", item.name)
        } else {
            item.name.to_string()
        };
        let text = truncate_middle(&text, max_width as usize);
        let fg = if selected { Color::SkyBlue } else { Color::White };
        let _ = write!(
            stdout,
            "{}{}{}{}",
            termion::cursor::Goto(h, v),
            termion::color::Bg(color::to_termion(Color::Black)),
            termion::color::Fg(color::to_termion(fg)),
            text,
        );
    }

    fn render_background(&self, stdout: &mut Box<dyn Write>) {
        for v in 1..=self.size.height {
            let text = " ".repeat(self.size.width as usize);
            let _ = write!(
                stdout,
                "{}{}{}{}",
                termion::cursor::Goto(1, v as u16),
                termion::color::Bg(color::to_termion(Color::Black)),
                termion::color::Fg(color::to_termion(Color::White)),
                text,
            );
        }
    }
}

pub fn truncate_middle(text: &str, max_width: usize) -> Cow<str> {
    if text.len() <= max_width {
        text.into()
    } else {
        let middle = max_width / 2;
        let count = text.len() - max_width + 1; // +1 to account for the ellipsis
        let mut str = text.to_string();
        str.replace_range(middle..(middle + count), "…");
        str.into()
    }
}
