// use super::Terminal;
// use crate::backend::{Command, Point};
// use fnv::FnvHashMap;
// use termion::event::Key;

// pub enum Action {
//     Command(Command),
//     Examine,
//     TargetNext,
//     TargetPrev,
//     ExitMode,
//     ToggleReplay,
//     StepReplay,
//     SpeedUpReplay,
//     SlowDownReplay,
//     SkipReplay,
//     ShowEvents,
//     Quit,
// }

// pub type KeyHandler = fn(&Terminal) -> Action;
// pub type CommandTable = FnvHashMap<Key, Box<KeyHandler>>;

// fn examine(terminal: &Terminal, dx: i32, dy: i32) -> Action {
//     let old = terminal.examined.unwrap();
//     Action::Command(Command::Examine(Point::new(old.x + dx, old.y + dy)))
// }

// pub fn replay_mode() -> CommandTable {
//     let mut mode: CommandTable = FnvHashMap::default();

//     mode.insert(Key::Char(' '), Box::new(|_terminal| Action::ToggleReplay));
//     mode.insert(Key::Char('s'), Box::new(|_terminal| Action::StepReplay));
//     mode.insert(Key::Char('+'), Box::new(|_terminal| Action::SpeedUpReplay));
//     mode.insert(Key::Char('-'), Box::new(|_terminal| Action::SlowDownReplay));
//     mode.insert(Key::Esc, Box::new(|_terminal| Action::SkipReplay));

//     mode
// }

// pub fn text_mode() -> CommandTable {
//     let mut mode: CommandTable = FnvHashMap::default();

//     mode.insert(Key::Esc, Box::new(|_terminal| Action::ExitMode));

//     mode
// }
