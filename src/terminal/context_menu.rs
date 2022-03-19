use super::color;
use one_thousand_deaths::{Color, Point, Size};
use std::fmt::Display;
use std::io::Write;
use termion::event::Key;

pub enum ContextResult<T: Copy + Display> {
    Selected(T),
    Pop,
    Updated,
    NotHandled,
}

/// Modal menu rendered on top of a parent view.
pub struct ContextMenu<T: Copy + Display> {
    pub parent_origin: Point,
    pub parent_size: Size,
    pub items: Vec<T>,
    pub suffix: String,
    pub selected: usize,
}

impl<T: Copy + Display> ContextMenu<T> {
    pub fn render(&self, stdout: &mut Box<dyn Write>) {
        assert!(!self.items.is_empty());

        let item_height = self.items.len() as i32;
        let max_item_width = self
            .items
            .iter()
            .enumerate()
            .map(|(i, tag)| self.to_name(i, *tag).len())
            .max()
            .unwrap_or(0) as i32;

        let h = self.parent_origin.x + (self.parent_size.width - (max_item_width + 4)) / 2;
        let v = self.parent_origin.y + (self.parent_size.height - (item_height + 2)) / 2;

        let h = h as u16;
        let mut v = v as u16;
        let max_item_width = max_item_width as usize;

        let stars = "*".repeat(max_item_width + 4);
        let _ = write!(
            stdout,
            "{}{}{}{}",
            termion::cursor::Goto(h, v),
            termion::color::Bg(color::to_termion(Color::Black)),
            termion::color::Fg(color::to_termion(Color::Salmon)),
            stars,
        );
        v += 1;

        for (i, tag) in self.items.iter().enumerate() {
            let _ = write!(
                stdout,
                "{}{}* ",
                termion::cursor::Goto(h, v),
                termion::color::Fg(color::to_termion(Color::Salmon)),
            );
            let fg = if i == self.selected {
                Color::SkyBlue
            } else {
                Color::White
            };
            let name = self.to_name(i, *tag);
            let _ = write!(stdout, "{}{}", termion::color::Fg(color::to_termion(fg)), name);
            if name.len() < max_item_width {
                let _ = write!(stdout, "{}", " ".repeat(max_item_width - name.len()),);
            }
            let _ = write!(stdout, "{} *", termion::color::Fg(color::to_termion(Color::Salmon)),);
            v += 1;
        }

        let _ = write!(
            stdout,
            "{}{}{}",
            termion::cursor::Goto(h, v),
            termion::color::Fg(color::to_termion(Color::Salmon)),
            stars,
        );
    }

    pub fn handle_input(&mut self, key: Key) -> ContextResult<T> {
        match key {
            Key::Down | Key::Char('2') => {
                if self.selected + 1 < self.items.len() {
                    self.selected += 1;
                } else {
                    self.selected = 0;
                }
                ContextResult::Updated
            }
            Key::Up | Key::Char('8') => {
                if self.selected > 0 {
                    self.selected -= 1;
                } else {
                    self.selected = self.items.len() - 1;
                }
                ContextResult::Updated
            }
            Key::Char('\n') => {
                let value = self.items[self.selected];
                ContextResult::Selected(value)
            }
            Key::Esc => ContextResult::Pop,
            _ => ContextResult::NotHandled,
        }
    }

    fn to_name(&self, i: usize, tag: T) -> String {
        if i == 0 {
            format!("{tag} {}", self.suffix)
        } else {
            format!("{tag}")
        }
    }
}
