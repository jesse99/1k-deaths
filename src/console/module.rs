use crate::shared::UI;
use crossterm::{
    cursor,
    style::{self, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};
use std::io::{self, Write};

struct Console {}

pub fn new() -> Box<dyn UI> {
    Box::new(Console {})
}

impl UI for Console {
    fn run(&self) -> io::Result<()> {
        let mut stdout = io::stdout();

        // TODO think we want raw mode, see https://docs.rs/crossterm/latest/crossterm/terminal/index.html#raw-mode
        stdout.execute(terminal::Clear(terminal::ClearType::All))?;

        let prompt = "hello world";
        let (width, height) = terminal::size().unwrap();
        let x = width / 2 - (prompt.bytes().len() / 2) as u16;
        let y = height / 2;

        stdout
            .queue(cursor::MoveTo(x, y))?
            .queue(style::PrintStyledContent(prompt.red()))?;
        stdout.flush()?;
        Ok(())
    }
}
