use super::{Event, Game, InputAction, Mode, RenderContext};
use fnv::FnvHashMap;
use termion::event::Key;

type KeyHandler = fn(&mut ReplayMode, &mut Game) -> InputAction;
type CommandTable = FnvHashMap<Key, Box<KeyHandler>>;

enum Replaying {
    Running,
    Blocking,
    SingleStep,
}

pub struct ReplayMode {
    replay: Vec<Event>,
    replaying: Replaying,
    timeout: i32, // ms
    commands: CommandTable,
}

const REPLAY_DELTA: i32 = 20;

impl ReplayMode {
    pub fn create(game: &mut Game, mut replay: Vec<Event>) -> Box<dyn Mode> {
        if let Some(i) = replay.iter().position(|e| matches!(e, Event::EndConstructLevel)) {
            // We don't want to replay setting the level up.
            let tail = replay.split_off(i + 1);
            let head = core::mem::replace(&mut replay, tail);
            game.post(head, true);
        }

        let mut commands: CommandTable = FnvHashMap::default();
        commands.insert(Key::Char(' '), Box::new(|s, game| s.do_toggle(game)));
        commands.insert(Key::Char('s'), Box::new(|s, game| s.do_step(game)));
        commands.insert(Key::Char('+'), Box::new(|s, game| s.do_speed_up(game)));
        commands.insert(Key::Char('-'), Box::new(|s, game| s.do_slow_down(game)));
        commands.insert(Key::Esc, Box::new(|s, game| s.do_skip(game)));
        commands.insert(Key::Char('q'), Box::new(|s, game| s.do_quit(game)));

        Box::new(ReplayMode {
            replay,
            replaying: Replaying::Running,
            timeout: 30,
            commands,
        })
    }
}

impl Mode for ReplayMode {
    fn render(&self, _context: &mut RenderContext) -> bool {
        false
    }

    fn input_timeout_ms(&self) -> Option<i32> {
        match self.replaying {
            Replaying::Running => Some(self.timeout),
            Replaying::Blocking => None,
            Replaying::SingleStep => None,
        }
    }

    fn handle_input(&mut self, game: &mut Game, key: termion::event::Key) -> InputAction {
        if key == termion::event::Key::Null {
            let e = self.replay.remove(0);
            game.post(vec![e], true);
            if self.replay.is_empty() {
                InputAction::Pop
            } else {
                InputAction::UpdatedGame
            }
        } else {
            match self.commands.get(&key).cloned() {
                Some(handler) => handler(self, game),
                None => InputAction::NotHandled,
            }
        }
    }
}

impl ReplayMode {
    fn do_toggle(&mut self, _game: &mut Game) -> InputAction {
        if let Replaying::Running = self.replaying {
            self.replaying = Replaying::Blocking;
        } else {
            self.replaying = Replaying::Running;
        }
        InputAction::UpdatedGame
    }

    fn do_step(&mut self, game: &mut Game) -> InputAction {
        self.replaying = Replaying::SingleStep;
        let e = self.replay.remove(0);
        game.post(vec![e], true);
        InputAction::UpdatedGame
    }

    fn do_speed_up(&mut self, _game: &mut Game) -> InputAction {
        if self.timeout > REPLAY_DELTA {
            self.timeout -= REPLAY_DELTA;
        } else {
            self.timeout = 0;
        }
        InputAction::UpdatedGame
    }

    fn do_slow_down(&mut self, _game: &mut Game) -> InputAction {
        self.timeout += REPLAY_DELTA;
        InputAction::UpdatedGame
    }

    fn do_skip(&mut self, game: &mut Game) -> InputAction {
        // This will skip UI updates so the player can start playing ASAP.
        // TODO: It would also be nice to have something like AbortReplay
        // so that the user can use only part of the saved events. However
        // this is tricky to do because we'd need to somehow truncate the
        // saved file. The way to do this is probably to write the replayed
        // events to a temp file and swap the two files if the user aborts.
        let events = std::mem::take(&mut self.replay);
        game.post(events, true);
        InputAction::Pop
    }

    fn do_quit(&mut self, _game: &mut Game) -> InputAction {
        InputAction::Quit
    }
}
