use crate::shared::UI;
use crossterm::{
    cursor,
    event::{self, KeyEvent},
    style::{self, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};
use std::io::{self, Write};

struct Console {
    player_x: u16,
    player_y: u16,
    running: bool,
}

pub fn new() -> Box<dyn UI> {
    Box::new(Console {
        player_x: 10,
        player_y: 10,
        running: true,
    })
}

// Note that there is support for non-blocking reads
impl UI for Console {
    fn run(&mut self) -> io::Result<()> {
        self.setup()?;
        while self.running {
            self.render()?;

            let event = event::read()?;
            match event {
                event::Event::FocusGained => (),
                event::Event::FocusLost => (),
                event::Event::Key(e) => self.handle_key(e),
                event::Event::Mouse(_) => (),
                event::Event::Paste(_) => (),
                event::Event::Resize(_, _) => (),
            }
        }
        Ok(())
    }
}

impl Console {
    fn setup(&self) -> io::Result<()> {
        terminal::enable_raw_mode()?;

        let mut stdout = io::stdout();
        stdout.queue(cursor::Hide {})?;
        Ok(())
    }

    // TODO don't allow user to move off screen
    // TODO print a message if user moves off screen
    fn render(&self) -> io::Result<()> {
        // let (width, height) = terminal::size().unwrap();
        // let x = width / 2 - (prompt.bytes().len() / 2) as u16;
        // let y = height / 2;

        let mut stdout = io::stdout();
        stdout.execute(terminal::Clear(terminal::ClearType::All))?;

        stdout
            .queue(cursor::MoveTo(self.player_x, self.player_y))?
            .queue(style::PrintStyledContent("@".red()))?;
        stdout.flush()?;
        Ok(())
    }

    // TODO can special case keypad using https://sw.kovidgoyal.net/kitty/keyboard-protocol/#progressive-enhancement
    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            event::KeyCode::Left | event::KeyCode::Char('4') => self.player_x -= 1,
            event::KeyCode::Right | event::KeyCode::Char('6') => self.player_x += 1,
            event::KeyCode::Up | event::KeyCode::Char('8') => self.player_y -= 1,
            event::KeyCode::Down | event::KeyCode::Char('2') => self.player_y += 1,
            event::KeyCode::Char('q') => self.running = false,
            _ => (), // TODO beep
        }
    }
}

// This restores the terminal state when the process exits but unfortunately it's called
// after panic! backtraces are printed so they are completely mis-formatted. TODO: there
// is a solution in https://werat.dev/blog/pretty-rust-backtraces-in-raw-terminal-mode
// but it's rather messy.
impl Drop for Console {
    fn drop(&mut self) {
        let mut stdout = io::stdout();
        let _ = stdout.queue(cursor::Show {});

        let _ = terminal::disable_raw_mode();
    }
}
