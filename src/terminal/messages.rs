use super::color;
use crate::backend::{Color, Game, Point, Size, Topic};
use std::io::Write;

/// Responsible for drawing the last few messages.
pub struct MessagesView {
    pub origin: Point,
    pub size: Size,
}

// TODO: Should have a command to show a screen full of messages. Ideally
// with support for scrolling and perhaps even searching.
impl MessagesView {
    pub fn render(&self, stdout: &mut Box<dyn Write>, game: &Game) {
        let h = (self.origin.x + 1) as u16; // termion is 1-based
        let mut v = (self.origin.y + 1) as u16;
        for message in game.recent_messages(self.size.height as usize) {
            let fg = to_fore_color(message.topic);
            let bg = Color::White;

            // Pad the string out to the full terminal width so that the back
            // color of the line is correct.
            let mut text = message.text.clone();
            text.push_str(&String::from(' ').repeat(self.size.width as usize - text.len()));
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
    }
}

fn to_fore_color(topic: Topic) -> Color {
    match topic {
        Topic::Error => Color::Red,
        Topic::Normal => Color::Black,
        Topic::Important => Color::Blue,
        Topic::NPCSpeaks => Color::Coral,
    }
}
