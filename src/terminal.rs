//! Rendering and UI using termion terminal module.
mod color;
mod map;
mod messages;
mod modes;

use super::backend::{Command, Event, Game, Point, Size};
use map::MapView;
use messages::MessagesView;
use modes::{Action, CommandTable};
use std::io::{stdin, stdout, Write};
use std::panic;
use std::process;
use termion::input::TermRead; // for keys trait
use termion::raw::IntoRawMode;

const NUM_MESSAGES: i32 = 5;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum GameState {
    Running,
    Exiting,
}

pub struct Terminal {
    game: Game,
    stdout: Box<dyn Write>,

    map: MapView,
    messages: MessagesView,
    replay: Vec<Event>,
    mode: CommandTable,
    examined: Option<Point>,
}

impl Terminal {
    pub fn new(game: Game, replay: Vec<Event>) -> Terminal {
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
            game,
            stdout: Box::new(stdout),
            map: MapView {
                origin: Point::new(0, 0),
                size: Size::new(width, height - NUM_MESSAGES),
            },
            messages: MessagesView {
                origin: Point::new(0, height - NUM_MESSAGES),
                size: Size::new(width, NUM_MESSAGES),
            },
            replay,
            mode: modes::move_mode(),
            examined: None,
        }
    }

    pub fn run(&mut self) {
        let stdin = stdin();
        let stdin = stdin.lock();
        let mut key_iter = stdin.keys();
        let mut state = GameState::Running;

        if let Some(i) = self.replay.iter().position(|e| matches!(e, Event::EndConstructLevel)) {
            // We don't want to replay setting the level up.
            let tail = self.replay.split_off(i + 1);
            let head = core::mem::replace(&mut self.replay, tail);
            self.game.post(head, true);
        }
        while state != GameState::Exiting {
            if !self.replay.is_empty() {
                let e = self.replay.remove(0);
                self.game.post(vec![e], true);
                self.render(); // TODO: probably should skip render while inside BeginConstructLevel
                std::thread::sleep(std::time::Duration::from_millis(25));
            } else {
                self.render();
                if let Some(c) = key_iter.next() {
                    let c = c.unwrap();
                    // debug!("input key {:?}", c);
                    state = self.handle_input(c);
                } else {
                    panic!("Couldn't read the next key");
                }
            }
        }
    }

    fn render(&mut self) {
        self.map.render(&mut self.stdout, &mut self.game, self.examined);
        self.messages.render(&mut self.stdout, &self.game);
        self.stdout.flush().unwrap();
    }

    fn handle_input(&mut self, key: termion::event::Key) -> GameState {
        match self.mode.get(&key) {
            Some(handler) => {
                let action = handler(self);
                match action {
                    Action::Command(command) => {
                        if let Command::Examine(loc) = command {
                            self.examined = Some(loc);
                        }
                        let mut events = Vec::new();
                        self.game.command(command, &mut events);
                        self.game.post(events, false);
                        GameState::Running
                    }
                    Action::Examine => {
                        self.mode = modes::examine_mode();
                        self.examined = Some(self.game.player());
                        GameState::Running
                    }
                    Action::TargetNext => {
                        self.tab_target(1);
                        GameState::Running
                    }
                    Action::TargetPrev => {
                        self.tab_target(-1);
                        GameState::Running
                    }
                    Action::ExitMode => {
                        self.mode = modes::move_mode();
                        self.examined = None;
                        GameState::Running
                    }
                    Action::Quit => GameState::Exiting,
                }
            }
            None => {
                debug!("player pressed {key:?}"); // TODO: beep?
                GameState::Running
            }
        }
    }

    fn tab_target(&mut self, delta: i32) {
        let old_loc = self.examined.unwrap();
        if let Some(loc) = self.game.target_next(&old_loc, delta) {
            self.examined = Some(loc);

            let mut events = Vec::new();
            self.game.command(Command::Examine(loc), &mut events);
            self.game.post(events, false);
        }
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
