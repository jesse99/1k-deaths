use super::Terminal;
use crate::backend::{Command, Point};
use fnv::FnvHashMap;
use termion::event::Key;

pub enum Action {
    Command(Command),
    Examine,
    ExitMode,
    Quit,
}

pub type KeyHandler = fn(&Terminal) -> Action;
pub type CommandTable = FnvHashMap<Key, Box<KeyHandler>>;

pub fn move_mode() -> CommandTable {
    let mut mode: CommandTable = FnvHashMap::default();

    mode.insert(
        Key::Left,
        Box::new(|_terminal| Action::Command(Command::Move { dx: -1, dy: 0 })),
    );
    mode.insert(
        Key::Right,
        Box::new(|_terminal| Action::Command(Command::Move { dx: 1, dy: 0 })),
    );
    mode.insert(
        Key::Up,
        Box::new(|_terminal| Action::Command(Command::Move { dx: 0, dy: -1 })),
    );
    mode.insert(
        Key::Down,
        Box::new(|_terminal| Action::Command(Command::Move { dx: 0, dy: 1 })),
    );
    mode.insert(
        Key::Char('1'),
        Box::new(|_terminal| Action::Command(Command::Move { dx: -1, dy: 1 })),
    );
    mode.insert(
        Key::Char('2'),
        Box::new(|_terminal| Action::Command(Command::Move { dx: 0, dy: 1 })),
    );
    mode.insert(
        Key::Char('3'),
        Box::new(|_terminal| Action::Command(Command::Move { dx: 1, dy: 1 })),
    );
    mode.insert(
        Key::Char('4'),
        Box::new(|_terminal| Action::Command(Command::Move { dx: -1, dy: 0 })),
    );
    mode.insert(
        Key::Char('6'),
        Box::new(|_terminal| Action::Command(Command::Move { dx: 1, dy: 0 })),
    );
    mode.insert(
        Key::Char('7'),
        Box::new(|_terminal| Action::Command(Command::Move { dx: -1, dy: -1 })),
    );
    mode.insert(
        Key::Char('8'),
        Box::new(|_terminal| Action::Command(Command::Move { dx: 0, dy: -1 })),
    );
    mode.insert(
        Key::Char('9'),
        Box::new(|_terminal| Action::Command(Command::Move { dx: 1, dy: -1 })),
    );
    mode.insert(Key::Char('x'), Box::new(|_terminal| Action::Examine));
    mode.insert(Key::Char('q'), Box::new(|_terminal| Action::Quit));

    mode
}

fn examine(terminal: &Terminal, dx: i32, dy: i32) -> Action {
    let old = terminal.examined.unwrap();
    Action::Command(Command::Examine(Point::new(old.x + dx, old.y + dy)))
}

pub fn examine_mode() -> CommandTable {
    let mut mode: CommandTable = FnvHashMap::default();

    mode.insert(Key::Left, Box::new(|terminal| examine(terminal, -1, 0)));
    mode.insert(Key::Right, Box::new(|terminal| examine(terminal, 1, 0)));
    mode.insert(Key::Up, Box::new(|terminal| examine(terminal, 0, -1)));
    mode.insert(Key::Down, Box::new(|terminal| examine(terminal, 0, 1)));
    mode.insert(
        Key::Char('1'),
        Box::new(|terminal| examine(terminal, -1, 1)),
    );
    mode.insert(Key::Char('2'), Box::new(|terminal| examine(terminal, 0, 1)));
    mode.insert(Key::Char('3'), Box::new(|terminal| examine(terminal, 1, 1)));
    mode.insert(
        Key::Char('4'),
        Box::new(|terminal| examine(terminal, -1, 0)),
    );
    mode.insert(Key::Char('6'), Box::new(|terminal| examine(terminal, 1, 0)));
    mode.insert(
        Key::Char('7'),
        Box::new(|terminal| examine(terminal, -1, -1)),
    );
    mode.insert(
        Key::Char('8'),
        Box::new(|terminal| examine(terminal, 0, -1)),
    );
    mode.insert(
        Key::Char('9'),
        Box::new(|terminal| examine(terminal, 1, -1)),
    );
    mode.insert(Key::Char('q'), Box::new(|_terminal| Action::Quit));
    mode.insert(Key::Esc, Box::new(|_terminal| Action::ExitMode));

    mode
}
