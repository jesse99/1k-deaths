use crate::backend::Game;
use std::io::{self, Write};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use termion::cursor;
use termion::event::Key;
use termion::input::TermRead; // for keys trait

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifeCycle {
    Running,
    Quit,
}

pub struct UI {
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

        UI { recv }
    }

    pub fn render(&self, stdout: &mut Box<dyn Write>, game: &mut Game) {
        write!(
            stdout,
            "{}{}loc: {}",
            termion::clear::All,
            cursor::Goto(1, 1),
            game.player_loc()
        )
        .unwrap();
    }

    fn get_key(&self) -> Key {
        self.recv.recv().unwrap()
    }

    fn clear(&self, stdout: &mut Box<dyn Write>) {
        write!(stdout, "{}", termion::clear::All).unwrap();
    }

    pub(super) fn handle_input(&mut self, stdout: &mut Box<dyn Write>, game: &mut Game) -> LifeCycle {
        let key = self.get_key();
        match key {
            Key::Left => self.do_move(game, -1, 0),
            Key::Right => self.do_move(game, 1, 0),
            Key::Up => self.do_move(game, 0, -1),
            Key::Down => self.do_move(game, 0, 1),
            Key::Char('1') => self.do_move(game, -1, 1),
            Key::Char('2') => self.do_move(game, 0, 1),
            Key::Char('3') => self.do_move(game, 1, 1),
            Key::Char('4') => self.do_move(game, -1, 0),
            Key::Char('6') => self.do_move(game, 1, 0),
            Key::Char('7') => self.do_move(game, -1, -1),
            Key::Char('8') => self.do_move(game, 0, -1),
            Key::Char('9') => self.do_move(game, 1, -1),
            Key::Char('q') => LifeCycle::Quit,
            _ => {
                debug!("player pressed {key:?}"); // TODO: beep?
                LifeCycle::Running
            }
        }
    }

    fn do_move(&mut self, game: &mut Game, dx: i32, dy: i32) -> LifeCycle {
        game.move_player(dx, dy);
        LifeCycle::Running
    }
}
