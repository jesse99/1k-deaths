#[macro_use]
extern crate slog; // for info!, debug!, etc

use slog::Logger;
use sloggers::Build; // for build trait
use std::cmp::min;
use std::collections::HashMap;
use std::fmt::{self, Formatter};
use std::io::{self, stdin, stdout, Write};
use std::panic;
use std::path::Path;
use std::process;
use std::str::FromStr; // for from_str trait
use termion::input::TermRead; // for keys trait
use termion::raw::IntoRawMode;

#[derive(Clone, Copy, Eq, PartialEq)]
enum GameState {
    Running,
    Exiting,
}

enum Terrain {
    DeepWater,
    Ground,
    ShallowWater,
    Wall,
}

/// Location within a level.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    /// top-left
    pub fn origin() -> Point {
        Point { x: 0, y: 0 }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

pub struct Level {
    pub width: i32,
    pub height: i32,
    pub player: Point,
    terrain: HashMap<Point, Terrain>, // TODO: use FnvHashMap?
}

impl Level {
    fn new() -> Level {
        let width = 100;
        let height = 30;
        let player = Point::new(20, 10);
        let mut terrain = HashMap::new();

        // Terrain defaults to ground
        for y in 0..height {
            for x in 0..width {
                terrain.insert(Point::new(x, y), Terrain::Ground);
            }
        }

        // Walls along the edges
        for y in 0..height {
            terrain.insert(Point::new(0, y), Terrain::Wall);
            terrain.insert(Point::new(width - 1, y), Terrain::Wall);
        }
        for x in 0..width {
            terrain.insert(Point::new(x, 0), Terrain::Wall);
            terrain.insert(Point::new(x, height - 1), Terrain::Wall);
        }

        // Small lake
        terrain.insert(Point::new(29, 20), Terrain::DeepWater);
        terrain.insert(Point::new(30, 20), Terrain::DeepWater); // lake center
        terrain.insert(Point::new(31, 20), Terrain::DeepWater);
        terrain.insert(Point::new(30, 19), Terrain::DeepWater);
        terrain.insert(Point::new(30, 21), Terrain::DeepWater);

        terrain.insert(Point::new(29, 19), Terrain::ShallowWater);
        terrain.insert(Point::new(31, 19), Terrain::ShallowWater);
        terrain.insert(Point::new(29, 21), Terrain::ShallowWater);
        terrain.insert(Point::new(31, 21), Terrain::ShallowWater);

        terrain.insert(Point::new(28, 20), Terrain::ShallowWater);
        terrain.insert(Point::new(32, 20), Terrain::ShallowWater);
        terrain.insert(Point::new(30, 18), Terrain::ShallowWater);
        terrain.insert(Point::new(30, 22), Terrain::ShallowWater);

        Level {
            width,
            height,
            player,
            terrain,
        }
    }
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

fn move_player(level: &mut Level, dx: i32, dy: i32) {
    level.player = Point::new(level.player.x + dx, level.player.y + dy);
}

fn can_move(level: &Level, dx: i32, dy: i32) -> bool {
    let new_loc = Point::new(level.player.x + dx, level.player.y + dy);
    match level.terrain.get(&new_loc).unwrap() {
        Terrain::DeepWater => false,
        Terrain::ShallowWater => true,
        Terrain::Wall => false,
        Terrain::Ground => true,
    }
}

fn handle_move_player(level: &mut Level, dx: i32, dy: i32) {
    if can_move(level, dx, dy) {
        move_player(level, dx, dy)
    }
}

fn handle_input(key: termion::event::Key, level: &mut Level) -> GameState {
    match key {
        termion::event::Key::Left => handle_move_player(level, -1, 0),
        termion::event::Key::Right => handle_move_player(level, 1, 0),
        termion::event::Key::Up => handle_move_player(level, 0, -1),
        termion::event::Key::Down => handle_move_player(level, 0, 1),
        termion::event::Key::Char('q') => return GameState::Exiting,
        _ => (),
    };
    GameState::Running
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

// Note that logging is async which should be OK as long as we continue to unwind
// on panics (though note that aborting would shrink binary size).
fn make_logger() -> Logger {
    // let severity = match sloggers::types::Severity::from_str(&options.log_level) {
    let severity = match sloggers::types::Severity::from_str("debug") {
        Ok(l) => l,
        Err(_) => {
            eprintln!("--log-level should be critical, error, warning, info, debug, or trace");
            process::exit(1);
        }
    };

    // "event" => event			uses slog::Value trait (so that output is structured)
    // "event" => %event		uses Display trait
    // "event" => ?event		uses Debug trait
    let path = Path::new("1k-deaths.log");
    let mut builder = sloggers::file::FileLoggerBuilder::new(path);
    builder.format(sloggers::types::Format::Compact);
    builder.overflow_strategy(sloggers::types::OverflowStrategy::Block); // TODO: logging is async which is kinda lame
    builder.source_location(sloggers::types::SourceLocation::None);
    builder.level(severity);
    builder.truncate();
    builder.build().unwrap()
}

fn main() {
    let root_logger = make_logger();
    let local = chrono::Local::now();
    info!(root_logger, "started up"; "on" => local.to_rfc2822(), "version" => env!("CARGO_PKG_VERSION"));
    //	info!(root_logger, "started up"; "seed" => options.seed, "on" => local.to_rfc2822());

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

    let mut level = Level::new();
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
