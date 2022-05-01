use super::main_mode::MainMode;
use super::mode::{InputAction, Mode, RenderContext};
// use super::replay_mode::ReplayMode;
use super::GameLoop;
use one_thousand_deaths::State;
use std::io::{self, Write};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use termion::event::Key;
use termion::input::TermRead; // for keys trait

pub struct UI {
    modes: Vec<Box<dyn Mode>>,
    recv: Receiver<Key>,
}

impl UI {
    pub fn new(width: i32, height: i32) -> UI {
        let (send, recv) = mpsc::channel();
        let _ = thread::spawn(move || {
            let stdin = io::stdin();
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

        let modes = vec![MainMode::create(width, height)];
        UI { modes, recv }
    }

    pub fn render(&self, stdout: &mut Box<dyn Write>, state: &mut State) {
        let mut context = RenderContext {
            stdout,
            state,
            examined: None,
        };
        for mode in self.modes.iter().rev() {
            if mode.render(&mut context) {
                return;
            }
        }
        panic!("No modes rendered!")
    }

    fn get_key(&self) -> Key {
        if let Some(ms) = self.modes.last().unwrap().input_timeout_ms() {
            let duration = std::time::Duration::from_millis(ms as u64);
            match self.recv.recv_timeout(duration) {
                Ok(key) => key,
                Err(_) => Key::Null, // bit of a hack
            }
        } else {
            self.recv.recv().unwrap()
        }
    }

    fn clear(&self, stdout: &mut Box<dyn Write>) {
        write!(stdout, "{}", termion::clear::All).unwrap();
    }

    pub(super) fn handle_input(&mut self, stdout: &mut Box<dyn Write>, state: &mut State) -> GameLoop {
        use InputAction::*;
        let key = self.get_key();
        let mode = self.modes.last_mut().unwrap();
        match mode.handle_input(state, key) {
            UpdatedGame => (),
            Quit => return GameLoop::Quitting,
            Push(mode) => {
                self.modes.push(mode);
                self.clear(stdout);
            }
            Pop => {
                let _ = self.modes.pop();
                assert!(!self.modes.is_empty());
                self.clear(stdout);
            }
            NotHandled => {
                debug!("player pressed {key:?}"); // TODO: beep?
            }
        }
        GameLoop::Running
    }
}
