use super::color;
use one_thousand_deaths::{render, Color, Point, Size, State, Topic};
use std::io::Write;

/// Responsible for drawing the last few messages.
pub struct MessagesView {
    pub origin: Point,
    pub size: Size,
}

impl MessagesView {
    pub fn render(&self, stdout: &mut Box<dyn Write>, state: &State) {
        let h = (self.origin.x + 1) as u16; // termion is 1-based
        let mut v = (self.origin.y + 1) as u16;
        let bg = Color::White;
        for message in render::recent_messages(state, self.size.height as usize) {
            let fg = to_fore_color(message.topic);

            // Pad the string out to the full terminal width so that the back
            // color of the line is correct.
            let mut text = message.text.clone();
            if self.size.width as usize > text.len() {
                text.push_str(&String::from(' ').repeat(self.size.width as usize - text.len()));
            }
            let _ = write!(
                stdout,
                "{}{}{}{}",
                termion::cursor::Goto(h, v),
                termion::color::Bg(color::to_termion(bg)),
                termion::color::Fg(color::to_termion(fg)),
                text // TODO: will need to wrap long lines, possibly with some sort of indication that it has wrapped
            );
            v += 1;
        }

        let text = " ".repeat(self.size.width as usize);
        while (v as i32) - self.origin.y <= self.size.height {
            let _ = write!(
                stdout,
                "{}{}{}{}",
                termion::cursor::Goto(h, v),
                termion::color::Bg(color::to_termion(bg)),
                termion::color::Fg(color::to_termion(Color::Black)),
                text
            );
            v += 1;
        }
    }
}

pub fn to_fore_color(topic: Topic) -> Color {
    use Topic::*;
    match topic {
        // Error => Color::Red,
        // Normal => Color::Black,
        Failed => Color::Red,
        Important => Color::Blue,
        // NpcIsDamaged => Color::LightSkyBlue,
        // NpcIsNotDamaged => Color::Black,
        // NPCSpeaks => Color::Coral,
        // PlayerDidDamage => Color::Goldenrod,
        // PlayerDidNoDamage => Color::Khaki,
        // PlayerIsDamaged => Color::Crimson,
        // PlayerIsNotDamaged => Color::Pink,
        // Warning => Color::Orange,
    }
}
