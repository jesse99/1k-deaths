//! Rendering and UI using termion terminal module.
use super::backend::{self, GameState, Level, Point, Terrain};
use slog::Logger;
use std::cmp::min;
use std::io::{self, stdin, stdout, Write};
use std::panic;
use std::process;
use termion::input::TermRead; // for keys trait
use termion::raw::IntoRawMode;

pub fn run(root_logger: Logger, mut level: Level) {
    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();
    write!(
        stdout,
        "{}{}{}",
        termion::style::Reset,
        termion::cursor::Hide,
        termion::clear::All
    )
    .unwrap();

    let old_hook = panic::take_hook();
    panic::set_hook(Box::new(move |arg| {
        restore_terminal();
        old_hook(arg);
    }));

    let stdin = stdin();
    let stdin = stdin.lock();
    let mut key_iter = stdin.keys();

    let mut state = GameState::Running;
    while state != GameState::Exiting {
        render(&mut stdout, &level);

        if let Some(c) = key_iter.next() {
            let c = c.unwrap();
            debug!(root_logger, "input"; "key" => ?c);
            state = handle_input(c, &mut level);
        } else {
            panic!("Couldn't read the next key");
        }
    }
    restore_terminal();
}

fn restore_terminal() {
    let mut stdout = io::stdout();
    let _ = write!(
        stdout,
        "{}{}{}{}",
        termion::style::Reset,
        termion::cursor::Restore,
        termion::cursor::Show,
        termion::cursor::Goto(1, 1)
    );
    let _ = write!(stdout, "{}", termion::clear::All);
    stdout.flush().unwrap();

    let _ = process::Command::new("reset").output(); // new line mode isn't reset w/o this
}

fn render(stdout: &mut dyn Write, level: &Level) {
    let (width, height) = termion::terminal_size().expect("couldn't get terminal size");
    for y in 0..min(level.height as u16, height) {
        for x in 0..min(level.width as u16, width) {
            let pt = Point::new(x as i32, y as i32);
            if pt == level.player {
                let color = termion::color::AnsiValue::rgb(0, 0, 4);
                let _ = write!(
                    stdout,
                    "{}{}@",
                    termion::cursor::Goto(x + 1, y + 1), // termion is 1-based
                    // termion::color::Bg(view.bg),
                    termion::color::Fg(color)
                );
            } else {
                let color = termion::color::AnsiValue::grayscale(0);
                let symbol = match level.terrain.get(&pt).unwrap() {
                    Terrain::DeepWater => "W",
                    Terrain::ShallowWater => "w",
                    Terrain::Wall => "#",
                    Terrain::Ground => ".",
                };
                let _ = write!(
                    stdout,
                    "{}{}{}",
                    termion::cursor::Goto(x + 1, y + 1), // termion is 1-based
                    // termion::color::Bg(view.bg),
                    termion::color::Fg(color),
                    symbol
                );
            }
        }
    }
    stdout.flush().unwrap();
}

fn handle_input(key: termion::event::Key, level: &mut Level) -> GameState {
    match key {
        termion::event::Key::Left => backend::handle_move_player(level, -1, 0),
        termion::event::Key::Right => backend::handle_move_player(level, 1, 0),
        termion::event::Key::Up => backend::handle_move_player(level, 0, -1),
        termion::event::Key::Down => backend::handle_move_player(level, 0, 1),
        termion::event::Key::Char('q') => return GameState::Exiting,
        _ => (),
    };
    GameState::Running
}
