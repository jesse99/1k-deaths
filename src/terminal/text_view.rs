use super::color;
use crate::backend::{Color, Point, Size};
use std::io::Write;

#[derive(Debug)]
pub enum TextRun {
    Text(String),
    Color(Color),
}

pub type Line = Vec<TextRun>;

/// Takes over the window and renders a scrollable number of lines.
pub struct TextView {
    origin: Point,
    size: Size,
    lines: Vec<Line>,
    start: usize,
    bg: Color,
}

// TODO: less shows a percentage...
impl TextView {
    pub fn new(lines: Vec<Line>) -> TextView {
        let num_lines = lines.len();
        let (width, height) = termion::terminal_size().expect("couldn't get terminal size");
        let start = if num_lines >= (height as usize) {
            num_lines - (height as usize)
        } else {
            0
        };
        TextView {
            origin: Point::origin(),
            size: Size::new(width as i32, height as i32),
            lines,
            start,
            bg: Color::Black,
        }
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn render(&self, stdout: &mut Box<dyn Write>) {
        let mut v = (self.origin.y + 1) as u16;
        let width = self.size.width as usize;
        let height = self.size.height as usize;
        let height = if self.start + height <= self.lines.len() {
            height
        } else {
            self.lines.len() - self.start
        };
        let _ = write!(stdout, "{}", termion::color::Bg(color::to_termion(self.bg)),);
        for index in self.start..self.start + height {
            let mut h = (self.origin.x + 1) as u16; // termion is 1-based
            for run in self.lines[index].iter() {
                // Note that we continue rendering even when truncating to ensure that
                // styling is set correctly.
                h = self.render_run(stdout, run, h, v);
            }
            if width > h as usize {
                let padding = " ".repeat(width - (h as usize));
                let _ = write!(stdout, "{}{}", termion::cursor::Goto(h, v), padding);
            }
            v += 1;
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        let height = self.size.height as usize;
        if self.lines.len() >= height {
            self.start = self.lines.len() - height;
        } else {
            self.start = 0;
        }
    }

    pub fn scroll(&mut self, delta: i32) {
        let height = self.size.height as usize;
        if delta > 0 {
            let delta = delta as usize;
            if self.start + delta + height <= self.lines.len() {
                self.start += delta;
            } else {
                self.scroll_to_bottom();
            }
        } else {
            let delta = -delta as usize;
            if self.start >= delta {
                self.start -= delta;
            } else {
                self.start = 0;
            }
        }
    }

    fn render_run(&self, stdout: &mut Box<dyn Write>, run: &TextRun, h: u16, v: u16) -> u16 {
        match run {
            TextRun::Text(str) => {
                if h + str.len() as u16 <= self.size.width as u16 {
                    // TODO: will need to wrap long lines, possibly with some sort of indication that it has wrapped
                    let _ = write!(stdout, "{}{}", termion::cursor::Goto(h, v), str);
                    return h + (str.len() as u16);
                }
            }
            TextRun::Color(color) => {
                let _ = write!(stdout, "{}", termion::color::Fg(color::to_termion(*color)));
            }
        }
        h
    }
}
