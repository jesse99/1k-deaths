// use super::color;
// use crate::backend::{Color, Point, Size};
// use std::io::Write;

// pub enum TextRun {
//     Text(String),
//     Color(Color),
// }

// pub type Line = Vec<TextRun>;

// /// Takes over the window and renders a scrollable number of lines.
// pub struct TextView {
//     pub origin: Point,
//     pub size: Size,
//     pub lines: Vec<Line>,
//     pub start: usize,
//     pub bg: Color,
// }

// impl TextView {
//     pub fn render(&self, stdout: &mut Box<dyn Write>) {
//         let mut v = (self.origin.y + 1) as u16;
//         for index in self.start..self.start + (self.size.height as usize) {
//             let mut h = (self.origin.x + 1) as u16; // termion is 1-based
//             for run in self.lines[index].iter() {
//                 // Note that we continue rendering even when truncating to ensure that
//                 // styling is set correctly.
//                 h = self.render_run(stdout, run, h, v);
//             }
//             v += 1;
//         }
//     }

//     pub fn scroll_to_bottom(&mut self) {
//         let height = self.size.height as usize;
//         if self.lines.len() >= height {
//             self.start = self.lines.len() - height;
//         } else {
//             self.start = 0;
//         }
//     }

//     fn render_run(&self, stdout: &mut Box<dyn Write>, run: &TextRun, h: u16, v: u16) -> u16 {
//         match run {
//             TextRun::Text(str) => {
//                 if h + str.len() as u16 <= self.size.width as u16 {
//                     // TODO: will need to wrap long lines, possibly with some sort of indication that it has wrapped
//                     let _ = write!(
//                         stdout,
//                         "{}{}{}",
//                         termion::cursor::Goto(h, v),
//                         termion::color::Bg(color::to_termion(self.bg)),
//                         str
//                     );
//                     return h + (str.len() as u16);
//                 }
//             }
//             TextRun::Color(color) => {
//                 let _ = write!(stdout, "{}", termion::color::Fg(color::to_termion(*color)));
//             }
//         }
//         h
//     }
// }
