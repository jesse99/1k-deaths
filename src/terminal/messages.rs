use super::color;
use super::View;
use crate::backend::{Color, Game, Topic};
use std::io::Write;

/// Responsible for drawing the last few messages.
pub fn render(stdout: &mut Box<dyn Write>, view: &View, game: &Game) {
    let h = (view.origin.x + 1) as u16; // termion is 1-based
    let mut v = (view.origin.y + 1) as u16;
    for message in game.recent_messages(view.size.height as usize) {
        let fg = to_fore_color(message.topic);
        let bg = Color::White;

        // Pad the string out to the full terminal width so that the back
        // color of the line is correct.
        let mut text = message.text.clone();
        text.push_str(&String::from(' ').repeat(view.size.width as usize - text.len()));
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

fn to_fore_color(topic: Topic) -> Color {
    match topic {
        Topic::NonGamePlay => Color::Blue,
    }
}
