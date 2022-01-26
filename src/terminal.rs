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

use std::sync::mpsc::channel;
// use std::sync::mpsc::TryRecvError;
use std::thread;
// use std::time::{Duration, Instant};

const NUM_MESSAGES: i32 = 5;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum GameState {
    Running,
    Exiting,
}

enum Replaying {
    Replay,
    Suspend,
    SingleStep,
}

pub struct Terminal {
    game: Game,
    stdout: Box<dyn Write>,

    map: MapView,
    messages: MessagesView,
    replay: Vec<Event>,
    replaying: Replaying,
    replay_delay: u64, // ms
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
            replay_delay: 30,
            replaying: Replaying::Replay,
            mode: modes::move_mode(),
            examined: None,
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

        if let Some(i) = self.replay.iter().position(|e| matches!(e, Event::EndConstructLevel)) {
            // We don't want to replay setting the level up.
            let tail = self.replay.split_off(i + 1);
            let head = core::mem::replace(&mut self.replay, tail);
            self.game.post(head, true);
            self.mode = modes::replay_mode();
        }
        while state != GameState::Exiting {
            if !self.replay.is_empty() {
                if let Replaying::Replay = self.replaying {
                    let e = self.replay.remove(0);
                    self.game.post(vec![e], true);
                    if self.replay.is_empty() {
                        self.mode = modes::move_mode();
                    }
                }
            }

            self.render();
            if let Replaying::Replay = self.replaying {
                let duration = std::time::Duration::from_millis(self.replay_delay);
                if let Ok(c) = recv.recv_timeout(duration) {
                    state = self.handle_input(c);
                }
            } else {
                let c = recv.recv().unwrap();
                state = self.handle_input(c);
            }
        }
    }

    fn render(&mut self) {
        self.map.render(&mut self.stdout, &mut self.game, self.examined);
        self.messages.render(&mut self.stdout, &self.game);
        self.stdout.flush().unwrap();
    }

    fn handle_input(&mut self, key: termion::event::Key) -> GameState {
        const REPLAY_DELTA: u64 = 20;

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
                    Action::ToggleReplay => {
                        if let Replaying::Replay = self.replaying {
                            self.replaying = Replaying::Suspend;
                        } else {
                            self.replaying = Replaying::Replay;
                        }
                        GameState::Running
                    }
                    Action::StepReplay => {
                        self.replaying = Replaying::SingleStep;
                        let e = self.replay.remove(0);
                        self.game.post(vec![e], true);
                        GameState::Running
                    }
                    Action::SpeedUpReplay => {
                        if self.replay_delay > REPLAY_DELTA {
                            self.replay_delay -= REPLAY_DELTA;
                        } else {
                            self.replay_delay = 0;
                        }
                        GameState::Running
                    }
                    Action::SlowDownReplay => {
                        self.replay_delay += REPLAY_DELTA;
                        GameState::Running
                    }
                    Action::SkipReplay => {
                        // This will skip UI updates so the player can start playing ASAP.
                        // TODO: It would also be nice to have something like AbortReplay
                        // so that the user can use only part of the saved events. However
                        // this is tricky to do because we'd need to somehow truncate the
                        // saved file. The way to do this is probably to write the replayed
                        // events to a temp file and swap the two files if the user aborts.
                        let events = std::mem::take(&mut self.replay);
                        self.game.post(events, true);
                        self.mode = modes::move_mode();
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
