//! Rendering and UI using termion terminal module.
mod color;
mod examine_mode;
mod main_mode;
mod map_view;
mod messages_view;
mod mode;
mod modes;
mod text;
mod ui;

// TODO: I think we can do better with these. Some are here only for sub-modules (which can
// pull them in with somewhat longer paths). May also be able to leverage the pub modifiers.
// See https://doc.rust-lang.org/stable/rust-by-example/mod/visibility.html
// Backend could do similar things.
use super::backend::{Command, Event, Game, Point, Size};
use main_mode::MainMode;
use map_view::MapView;
use messages_view::MessagesView;
use mode::{InputAction, Mode, RenderContext};
use std::io::{stdin, stdout, Write};
use std::panic;
use std::process;
use std::sync::mpsc::channel;
use std::thread;
use termion::input::TermRead; // for keys trait
use termion::raw::IntoRawMode;
use ui::UI;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum GameState {
    // TODO: do we really need this?
    Running,
    Exiting,
}

// enum Replaying {
//     Replay,
//     Suspend,
//     SingleStep,
// }

pub struct Terminal {
    ui: UI,
    game: Game,
    stdout: Box<dyn Write>,
    // replay: Vec<Event>,
    // replaying: Replaying,
    // replay_delay: u64, // ms
}

impl Terminal {
    pub fn new(game: Game, _replay: Vec<Event>) -> Terminal {
        let stdout = stdout();
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
            ui: UI::new(width, height),
            game,
            stdout: Box::new(stdout),
            // replay,
            // replay_delay: 30,
            // replaying: Replaying::Replay,
        }
    }

    pub fn run(&mut self) {
        let mut state = GameState::Running;

        let (send, recv) = channel();
        thread::spawn(move || {
            let stdin = stdin();
            let stdin = stdin.lock();
            let mut key_iter = stdin.keys();

            loop {
                if let Some(c) = key_iter.next() {
                    let c = c.unwrap();
                    // debug!("input key {:?}", c);
                    send.send(c).unwrap();
                } else {
                    panic!("Couldn't read the next key");
                }
            }
        });

        // if let Some(i) = self.replay.iter().position(|e| matches!(e, Event::EndConstructLevel)) {
        //     // We don't want to replay setting the level up.
        //     let tail = self.replay.split_off(i + 1);
        //     let head = core::mem::replace(&mut self.replay, tail);
        //     self.game.post(head, true);
        //     self.mode = modes::replay_mode();
        // }
        while state != GameState::Exiting {
            // if !self.replay.is_empty() {
            //     if let Replaying::Replay = self.replaying {
            //         let e = self.replay.remove(0);
            //         self.game.post(vec![e], true);
            //         if self.replay.is_empty() {
            //             self.mode = modes::main_mode();
            //         }
            //     }
            // }

            self.render();
            // if let Replaying::Replay = self.replaying {
            //     let duration = std::time::Duration::from_millis(self.replay_delay);
            //     if let Ok(c) = recv.recv_timeout(duration) {
            //         state = self.handle_input(c);
            //     }
            // } else {
            let c = recv.recv().unwrap();
            state = self.handle_input(c);
            // }
        }
    }

    fn render(&mut self) {
        self.ui.render(&mut self.stdout, &mut self.game);
        self.stdout.flush().unwrap();
    }

    fn handle_input(&mut self, key: termion::event::Key) -> GameState {
        self.ui.handle_input(&mut self.game, key)
    }
}

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
