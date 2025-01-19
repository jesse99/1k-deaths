use super::*;
use fnv::FnvHashMap;
use std::time::Instant;
use termion::event::Key;

enum Replaying {
    Running,
    Blocking,
    SingleStep,
}

pub struct ReplayMode {
    replay: Vec<Command>,
    replaying: Replaying,
    timeout: i32, // ms
    commands: CommandTable,
    start_time: Instant,
}

const REPLAY_DELTA: i32 = 20;

impl ReplayMode {
    pub fn create(replay: Vec<Command>) -> Box<dyn Mode> {
        let mut commands: CommandTable = FnvHashMap::default();
        commands.insert(Key::Char(' '), Command::ToggleReplay);
        commands.insert(Key::Char('s'), Command::StepReplay);
        commands.insert(Key::Char('+'), Command::SpeedUpReplay);
        commands.insert(Key::Char('-'), Command::SlowDownReplay);
        commands.insert(Key::Char('?'), Command::Help);
        commands.insert(Key::Esc, Command::SkipReplay);
        commands.insert(Key::Char('q'), Command::Quit);

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

    // TODO: if window is replaying then it needs to direct input here
    // TODO: but commands still need to be directed at the top mode
    fn handle_input(&self, key: Key) -> Option<Command> {
        if self.replay.is_empty() {
            let elapsed = self.start_time.elapsed();
            info!("done replaying after {elapsed:.1?} secs");
            Some(Command::Quit)
        } else if key == Key::Null {
            Some(self.replay.remove(0))
        } else {
            self.commands.get(&key).copied()
        }
    }

    fn handle_command(&mut self, ipc: &IPC, command: Command) -> CommandResult {
        match command {
            Command::Help => self.do_help(),
            Command::Quit => CommandResult::Quit,
            Command::SkipReplay => self.do_scroll_by(ipc, sign),
            Command::SlowDownReplay => self.do_scroll_by(ipc, sign),
            Command::SpeedUpReplay => self.do_scroll_by(ipc, sign),
            Command::StepReplay => self.do_scroll(ipc, delta),
            Command::ToggleReplay => self.do_page(ipc, sign),
            _ => panic!("didn't handle {command:?}"),
        }
    }

    fn input_timeout_ms(&self) -> Option<i32> {
        match self.replaying {
            Replaying::Running => Some(self.timeout),
            Replaying::Blocking => None,
            Replaying::SingleStep => None,
        }
    }
}

impl ReplayMode {
    fn do_help(&mut self) -> CommandResult {
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
        CommandResult::Push(TextMode::at_top().create(lines))
    }

    // TODO: handle this by having a CommandResult with a list of commands?
    fn do_skip(&mut self, ipc: &IPC) -> CommandResult {
        // This will skip UI updates so the player can start playing ASAP.
        // TODO: It would also be nice to have something like AbortReplay
        // so that the user can use only part of the saved events. However
        // this is tricky to do because we'd need to somehow truncate the
        // saved file. The way to do this is probably to write the replayed
        // events to a temp file and swap the two files if the user aborts.
        let commands = std::mem::take(&mut self.replay);
        for command in commands.into_iter() {
            game.replay_action(command);
        }
        let elapsed = self.start_time.elapsed();
        info!("done replaying after {elapsed:.1?} secs");
        CommandResult::Pop
    }

    fn do_slow_down(&mut self) -> CommandResult {
        self.timeout += REPLAY_DELTA;
        CommandResult::UpdatedGame
    }

    fn do_speed_up(&mut self) -> CommandResult {
        if self.timeout > REPLAY_DELTA {
            self.timeout -= REPLAY_DELTA;
        } else if self.timeout > 0 {
            self.timeout = 0;
        } else {
            // This is not working (nor does it work when the raw stdout is used directly).
            // let _ = io::stdout().write(b"\x07");
        }
        CommandResult::UpdatedGame
    }

    fn do_step(&mut self, ipc: &IPC) -> CommandResult {
        self.replaying = Replaying::SingleStep;
        let command = self.replay.remove(0);
        game.replay_action(command);
        CommandResult::UpdatedGame
    }

    fn do_toggle(&mut self) -> CommandResult {
        if let Replaying::Running = self.replaying {
            self.replaying = Replaying::Blocking;
        } else {
            self.replaying = Replaying::Running;
        }
        CommandResult::UpdatedGame
    }
}
