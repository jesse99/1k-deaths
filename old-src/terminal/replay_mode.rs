use super::help::{format_help, validate_help};
use super::mode::{InputAction, Mode, RenderContext};
use super::text_mode::TextMode;
use fnv::FnvHashMap;
use one_thousand_deaths::{Action, Game};
use std::time::Instant;
use termion::event::Key;

type KeyHandler = fn(&mut ReplayMode, &mut Game) -> InputAction;
type CommandTable = FnvHashMap<Key, Box<KeyHandler>>;

enum Replaying {
    Running,
    Blocking,
    SingleStep,
}

pub struct ReplayMode {
    replay: Vec<Action>,
    replaying: Replaying,
    timeout: i32, // ms
    commands: CommandTable,
    start_time: Instant,
}

const REPLAY_DELTA: i32 = 20;

impl ReplayMode {
    pub fn create(replay: Vec<Action>) -> Box<dyn Mode> {
        let mut commands: CommandTable = FnvHashMap::default();
        commands.insert(Key::Char(' '), Box::new(|s, game| s.do_toggle(game)));
        commands.insert(Key::Char('s'), Box::new(|s, game| s.do_step(game)));
        commands.insert(Key::Char('+'), Box::new(|s, game| s.do_speed_up(game)));
        commands.insert(Key::Char('-'), Box::new(|s, game| s.do_slow_down(game)));
        commands.insert(Key::Char('?'), Box::new(|s, game| s.do_help(game)));
        commands.insert(Key::Esc, Box::new(|s, game| s.do_skip(game)));
        commands.insert(Key::Char('q'), Box::new(|s, game| s.do_quit(game)));

        Box::new(ReplayMode {
            replay,
            replaying: Replaying::Running,
            timeout: 10,
            commands,
            start_time: Instant::now(),
        })
    }
}

impl Mode for ReplayMode {
    fn replaying(&self) -> bool {
        true
    }

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

    fn handle_input(&mut self, game: &mut Game, key: Key) -> InputAction {
        if self.replay.is_empty() {
            let elapsed = self.start_time.elapsed();
            info!("done replaying after {elapsed:.1?} secs");
            InputAction::Pop
        } else if key == Key::Null {
            let action = self.replay.remove(0);
            game.replay_action(action);
            InputAction::UpdatedGame
        } else {
            match self.commands.get(&key).cloned() {
                Some(handler) => handler(self, game),
                None => InputAction::NotHandled,
            }
        }
    }
}

impl ReplayMode {
    fn do_help(&mut self, _game: &mut Game) -> InputAction {
        let help = r#"Replaying a saved game.

[[space]] toggles replay on and off.
[[s]] single step replay.
[[+]] speed up replay.
[[-]] slow down replay.
[[?]] show this help.
[[q]] save and quit.
[[escape]] exits replay mode."#;
        validate_help("replay", help, self.commands.keys());

        let lines = format_help(help, self.commands.keys());
        InputAction::Push(TextMode::at_top().create(lines))
    }

    fn do_quit(&mut self, _game: &mut Game) -> InputAction {
        InputAction::Quit
    }

    fn do_skip(&mut self, game: &mut Game) -> InputAction {
        // This will skip UI updates so the player can start playing ASAP.
        // TODO: It would also be nice to have something like AbortReplay
        // so that the user can use only part of the saved events. However
        // this is tricky to do because we'd need to somehow truncate the
        // saved file. The way to do this is probably to write the replayed
        // events to a temp file and swap the two files if the user aborts.
        let actions = std::mem::take(&mut self.replay);
        for action in actions.into_iter() {
            game.replay_action(action);
        }
        let elapsed = self.start_time.elapsed();
        info!("done replaying after {elapsed:.1?} secs");
        InputAction::Pop
    }

    fn do_slow_down(&mut self, _game: &mut Game) -> InputAction {
        self.timeout += REPLAY_DELTA;
        InputAction::UpdatedGame
    }

    fn do_speed_up(&mut self, _game: &mut Game) -> InputAction {
        if self.timeout > REPLAY_DELTA {
            self.timeout -= REPLAY_DELTA;
        } else if self.timeout > 0 {
            self.timeout = 0;
        } else {
            // This is not working (nor does it work when the raw stdout is used directly).
            // let _ = io::stdout().write(b"\x07");
        }
        InputAction::UpdatedGame
    }

    fn do_step(&mut self, game: &mut Game) -> InputAction {
        self.replaying = Replaying::SingleStep;
        let action = self.replay.remove(0);
        game.replay_action(action);
        InputAction::UpdatedGame
    }

    fn do_toggle(&mut self, _game: &mut Game) -> InputAction {
        if let Replaying::Running = self.replaying {
            self.replaying = Replaying::Blocking;
        } else {
            self.replaying = Replaying::Running;
        }
        InputAction::UpdatedGame
    }
}
