use super::*;
// use std::cell::RefCell;
use std::io::{self, Write};
use std::process;
use termion::raw::IntoRawMode;

// thread_local!(pub static WIZARD_MODE: RefCell<bool> = RefCell::new(false));

// pub fn wizard_mode() -> bool {
//     WIZARD_MODE.with(|w| *w.borrow())
// }

pub struct Terminal {
    ui: UI,
    ipc: IPC,
    stdout: Box<dyn Write>,
}

impl Terminal {
    // pub fn new(ipc: IPC, replay: Vec<Action>) -> Terminal {
    pub fn new(ipc: IPC) -> Terminal {
        let stdout = io::stdout();
        let mut stdout = stdout.into_raw_mode().unwrap();
        write!(
            stdout,
            "{}{}{}",
            termion::style::Reset,
            termion::cursor::Hide,
            termion::clear::All
        )
        .unwrap();

        let (width, height) = termion::terminal_size().expect("couldn't get terminal size");
        let width = width as i32;
        let height = height as i32;
        info!("terminal size is {} x {}", width, height);

        Terminal {
            // ui: UI::new(width, height, replay),
            ui: UI::new(width, height),
            ipc,
            stdout: Box::new(stdout),
        }
    }

    pub fn run(&mut self) {
        loop {
            self.render();
            if self.ui.handle_input(&mut self.stdout, &mut self.ipc) != LifeCycle::Running {
                break;
            }
        }
    }

    fn render(&mut self) {
        self.ui.render(&mut self.stdout, &mut self.ipc);
        self.stdout.flush().unwrap();
    }
}

// This restores the terminal state when the process exits but unfortunately it's called
// after panic! backtraces are printed so they are completely mis-formatted. TODO: there
// is a solution in https://werat.dev/blog/pretty-rust-backtraces-in-raw-terminal-mode
// but it's rather messy.
impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = write!(
            self.stdout,
            "{}{}{}{}",
            termion::style::Reset,
            termion::cursor::Restore,
            termion::cursor::Show,
            termion::cursor::Goto(1, 1)
        );
        let _ = write!(self.stdout, "{}", termion::clear::All);
        self.stdout.flush().unwrap();
        let _ = process::Command::new("reset").output(); // new line mode isn't reset w/o this
    }
}
