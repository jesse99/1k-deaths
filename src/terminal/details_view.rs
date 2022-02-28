use super::color;
use crate::backend::{Color, Disposition, Game, Point, Size};
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
        self.render_npcs(h, &mut v, stdout, game);
        self.render_trailer(h, v, stdout);
    }

    fn render_player(&self, h: u16, v: &mut u16, stdout: &mut Box<dyn Write>, game: &Game) {
        let (current, max) = game.player_hps();
        let percent = (current as f64) / (max as f64);
        let fg = self.player_color(percent);
        let n = (10.0 * percent).round() as usize;

        let bar1 = format!(" {}", "*".repeat(n));
        let bar2 = "*".repeat(10 - n);
        let suffix = format!("{current}/{max}");
        self.render_char(h, *v, &bar1, &bar2, &suffix, fg, stdout);
        *v += 1;

        self.render_char(h, *v, "", "", "", fg, stdout);
        *v += 1;
    }

    // TODO: Should be an indication if the NPC is really dangerous, maybe use bold
    // or do something on the map.
    fn render_npcs(&self, h: u16, v: &mut u16, stdout: &mut Box<dyn Write>, game: &Game) {
        let npcs = game.npcs(super::wizard_mode());

        for npc in npcs.iter().take(5) {
            let current = npc.observed_hps.0 as usize;
            let max = npc.observed_hps.1 as usize;

            let (bar1, bar2) = if npc.is_sleeping {
                (" ".to_string(), "sleeping".to_string())
            } else {
                (format!(" {}", "*".repeat(current)), "*".repeat(max - current))
            };
            let fg = match npc.disposition {
                Disposition::Aggressive => Color::Red,
                Disposition::Neutral => Color::Blue,
                Disposition::Friendly => Color::Green,
            };
            self.render_char(h, *v, &bar1, &bar2, npc.name, fg, stdout);

            *v += 1;
        }
    }

    fn render_char(
        &self,
        h: u16,
        v: u16,
        bar1: &str,
        bar2: &str,
        suffix: &str,
        fg: Color,
        stdout: &mut Box<dyn Write>,
    ) {
        let bg = Color::White;

        let _ = write!(
            stdout,
            "{}{}{}{}",
            termion::cursor::Goto(h, v),
            termion::color::Bg(color::to_termion(bg)),
            termion::color::Fg(color::to_termion(fg)),
            bar1
        );

        let _ = write!(
            stdout,
            "{}{}{}",
            termion::color::Bg(color::to_termion(bg)),
            termion::color::Fg(color::to_termion(Color::LightGrey)),
            bar2
        );

        let _ = write!(
            stdout,
            " {}{}{}",
            termion::color::Bg(color::to_termion(bg)),
            termion::color::Fg(color::to_termion(fg)),
            suffix
        );

        // Pad the string out to the full terminal width so that the back
        // color of the line is correct.
        let count = bar1.len() + bar2.len() + suffix.len() + 1; // +1 because we inserted a space before the suffix
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
