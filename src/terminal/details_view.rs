use super::color;
use crate::backend::{Color, Game, Point, Size};
use std::io::Write;

/// Shows info about the player and nearby NPCs.
pub struct DetailsView {
    pub origin: Point,
    pub size: Size,
}

impl DetailsView {
    pub fn render(&self, stdout: &mut Box<dyn Write>, game: &Game) {
        let h = (self.origin.x + 1) as u16; // termion is 1-based
        let mut v = 1;

        self.render_player(h, &mut v, stdout, game);
        self.render_trailer(h, v, stdout);
    }

    fn render_player(&self, h: u16, v: &mut u16, stdout: &mut Box<dyn Write>, game: &Game) {
        let (current, max) = game.player_hps();
        let percent = (current as f64) / (max as f64);

        let fg = self.player_color(percent);
        let bg = Color::White;

        let n = (10.0 * percent).round() as usize;
        let text = format!(" {}", "*".repeat(n));
        let _ = write!(
            stdout,
            "{}{}{}{}",
            termion::cursor::Goto(h, *v),
            termion::color::Bg(color::to_termion(bg)),
            termion::color::Fg(color::to_termion(fg)),
            text
        );

        let text2 = "*".repeat(10 - n);
        let _ = write!(
            stdout,
            "{}{}{}",
            termion::color::Bg(color::to_termion(bg)),
            termion::color::Fg(color::to_termion(Color::LightGrey)),
            text2
        );

        let text3 = format!(" {current}/{max}");
        let _ = write!(
            stdout,
            "{}{}{}",
            termion::color::Bg(color::to_termion(bg)),
            termion::color::Fg(color::to_termion(fg)),
            text3
        );

        // Pad the string out to the full terminal width so that the back
        // color of the line is correct.
        let count = text.len() + text2.len() + text3.len();
        if self.size.width as usize > count {
            let padding = " ".repeat(self.size.width as usize - count);
            let _ = write!(
                stdout,
                "{}{}{}",
                termion::color::Bg(color::to_termion(bg)),
                termion::color::Fg(color::to_termion(fg)),
                padding
            );
        }
        *v += 1;
    }

    fn render_trailer(&self, h: u16, in_v: u16, stdout: &mut Box<dyn Write>) {
        let fg = Color::Black;
        let bg = Color::White;
        let _ = write!(
            stdout,
            "{}{}",
            termion::color::Bg(color::to_termion(bg)),
            termion::color::Fg(color::to_termion(fg)),
        );

        let text = " ".repeat(self.size.width as usize);
        for v in in_v..=(self.size.height as u16) {
            let _ = write!(stdout, "{}{}", termion::cursor::Goto(h, v), text);
        }
    }

    fn player_color(&self, percent: f64) -> Color {
        if percent < 0.2 {
            Color::Red
        } else if percent < 0.4 {
            Color::Orange
        } else if percent < 0.6 {
            Color::Gold
        } else if percent < 0.8 {
            Color::Blue
        } else {
            Color::Green
        }
    }
}
