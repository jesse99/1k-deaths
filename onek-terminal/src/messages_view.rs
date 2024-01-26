use super::*;
use std::io::Write;

/// Responsible for drawing the last few messages.
pub struct MessagesView {
    pub origin: Point,
    pub size: Size,
}

impl MessagesView {
    pub fn render(&self, stdout: &mut Box<dyn Write>, ipc: &IPC) {
        let h = (self.origin.x + 1) as u16; // termion is 1-based
        let mut v = (self.origin.y + 1) as u16;
        let bg = Color::White;
        for note in ipc.get_notes(self.size.height as usize) {
            let fg = to_fore_color(note.kind);

            // Pad the string out to the full terminal width so that the back
            // color of the line is correct.
            let mut text = note.text.clone();
            if self.size.width as usize > text.len() {
                text.push_str(&String::from(' ').repeat(self.size.width as usize - text.len()));
            }
            let _ = write!(
                stdout,
                "{}{}{}{}",
                termion::cursor::Goto(h, v),
                termion::color::Bg(to_termion(bg)),
                termion::color::Fg(to_termion(fg)),
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
                termion::color::Bg(to_termion(bg)),
                termion::color::Fg(to_termion(Color::Black)),
                text
            );
            v += 1;
        }
    }
}

pub fn to_fore_color(kind: NoteKind) -> Color {
    use NoteKind::*;
    match kind {
        Error => Color::Red,
        Environmental => Color::Blue,
        Info => Color::Black,
    }
}
