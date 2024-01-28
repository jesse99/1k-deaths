use super::*;
// use std::cell::RefCell;
use std::io::{self, Write};
use std::process;
use std::time::Instant;
use termion::raw::IntoRawMode;

// thread_local!(pub static WIZARD_MODE: RefCell<bool> = RefCell::new(false));

// pub fn wizard_mode() -> bool {
//     WIZARD_MODE.with(|w| *w.borrow())
// }

pub struct Terminal {
    ui: UI,
    ipc: IPC,
    stdout: Box<dyn Write>,
}

impl Terminal {
    // pub fn new(ipc: IPC, replay: Vec<Action>) -> Terminal {
    pub fn new(ipc: IPC) -> Terminal {
        let stdout = io::stdout();
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
            // ui: UI::new(width, height, replay),
            ui: UI::new(width, height),
            ipc,
            stdout: Box::new(stdout),
        }
    }

    pub fn run(&mut self) {
        loop {
            self.render();
            if self.ui.handle_input(&mut self.stdout, &mut self.ipc) != LifeCycle::Running {
                break;
            }
        }
    }

    // debug:   60s
    // release: 45s
    pub fn benchmark(&mut self) {
        // TODO: It'd be better to just replay a saved game.
        // TODO: Might be better to use something like the criterion crate (or cargo
        // benchmark if it ever stabilizes).
        let locs = vec![
            Point::new(14, 14),
            Point::new(15, 14),
            Point::new(16, 14),
            Point::new(17, 14),
            Point::new(18, 14),
            Point::new(19, 14),
            Point::new(20, 14),
            Point::new(21, 14),
            Point::new(22, 14),
            Point::new(23, 14),
            Point::new(24, 14),
            Point::new(25, 14),
            Point::new(26, 14),
            Point::new(27, 14),
            Point::new(28, 14),
            Point::new(29, 14),
            Point::new(30, 14),
            Point::new(31, 14),
            Point::new(32, 14),
            Point::new(33, 14),
            Point::new(34, 14),
            Point::new(35, 14),
            Point::new(36, 14),
            Point::new(37, 13),
            Point::new(38, 12),
            Point::new(39, 11),
            Point::new(40, 10),
            Point::new(41, 9),
            Point::new(42, 8),
            Point::new(43, 7),
            Point::new(44, 6),
            Point::new(45, 5),
            Point::new(46, 4),
            Point::new(47, 3),
            Point::new(48, 3),
            Point::new(49, 3),
            Point::new(50, 3),
            Point::new(51, 3),
            Point::new(52, 3),
            Point::new(53, 4),
            Point::new(54, 5),
            Point::new(55, 6),
            Point::new(56, 7),
            Point::new(57, 8),
            Point::new(58, 9),
            Point::new(59, 10),
            Point::new(60, 10),
            Point::new(61, 10),
            Point::new(62, 10),
            Point::new(63, 10),
            Point::new(64, 10),
            Point::new(65, 10),
            Point::new(66, 10),
            Point::new(67, 10),
            Point::new(68, 11),
            Point::new(69, 12),
            Point::new(70, 13),
            Point::new(71, 14),
            Point::new(72, 15),
            Point::new(73, 16),
            Point::new(74, 17),
            Point::new(75, 18),
            Point::new(76, 19),
            Point::new(77, 20),
            Point::new(78, 21),
            Point::new(79, 22),
            Point::new(80, 23),
            Point::new(81, 24),
            Point::new(82, 25),
            Point::new(83, 26),
            Point::new(84, 27),
            Point::new(85, 28),
            Point::new(86, 29),
            Point::new(87, 30),
            Point::new(88, 31),
            Point::new(89, 32),
            Point::new(90, 32),
            Point::new(91, 32),
            Point::new(92, 32),
            Point::new(93, 32),
            Point::new(94, 32),
            Point::new(95, 32),
            Point::new(96, 32),
            Point::new(97, 32),
            Point::new(98, 32),
            Point::new(99, 32),
            Point::new(100, 32),
            Point::new(101, 32),
            Point::new(102, 32),
            Point::new(103, 32),
            Point::new(104, 32),
            Point::new(105, 32),
            Point::new(106, 32),
            Point::new(107, 32),
            Point::new(108, 32),
            Point::new(109, 32),
            Point::new(110, 32),
            Point::new(111, 32),
            Point::new(112, 32),
            Point::new(113, 32),
            Point::new(112, 32),
            Point::new(111, 32),
            Point::new(110, 32),
            Point::new(109, 32),
            Point::new(108, 32),
            Point::new(107, 32),
            Point::new(106, 32),
            Point::new(105, 32),
            Point::new(104, 32),
            Point::new(103, 32),
            Point::new(102, 31),
            Point::new(102, 30),
            Point::new(102, 29),
            Point::new(102, 28),
            Point::new(103, 27),
            Point::new(104, 26),
            Point::new(105, 25),
            Point::new(106, 24),
            Point::new(107, 23),
            Point::new(108, 23),
            Point::new(109, 23),
            Point::new(110, 23),
            Point::new(111, 23),
            Point::new(112, 23),
            Point::new(113, 23),
            Point::new(114, 22),
            Point::new(113, 22),
            Point::new(112, 22),
            Point::new(111, 22),
            Point::new(110, 22),
            Point::new(109, 22),
            Point::new(108, 22),
            Point::new(107, 22),
            Point::new(106, 21),
            Point::new(105, 21),
            Point::new(104, 21),
            Point::new(103, 20),
            Point::new(103, 19),
            Point::new(103, 18),
            Point::new(103, 17),
            Point::new(103, 16),
            Point::new(103, 15),
            Point::new(104, 14),
            Point::new(105, 13),
            Point::new(106, 12),
            Point::new(107, 11),
            Point::new(108, 11),
            Point::new(109, 11),
            Point::new(110, 11),
            Point::new(111, 11),
            Point::new(112, 11),
            Point::new(113, 11),
            Point::new(114, 11),
            Point::new(115, 11),
            Point::new(116, 11),
            Point::new(117, 11),
            Point::new(118, 11),
            Point::new(118, 12),
            Point::new(118, 13),
            Point::new(118, 14),
            Point::new(118, 13),
            Point::new(118, 12),
            Point::new(118, 11),
            Point::new(118, 10),
            Point::new(117, 9),
            Point::new(116, 8),
            Point::new(116, 7),
            Point::new(116, 6),
            Point::new(116, 5),
            Point::new(116, 4),
            Point::new(116, 3),
            Point::new(116, 2),
            Point::new(115, 2),
            Point::new(114, 2),
            Point::new(113, 2),
            Point::new(112, 2),
            Point::new(111, 2),
            Point::new(110, 2),
            Point::new(109, 2),
            Point::new(108, 2),
            Point::new(107, 2),
            Point::new(106, 2),
            Point::new(105, 2),
            Point::new(104, 2),
            Point::new(103, 2),
            Point::new(102, 2),
            Point::new(101, 2),
            Point::new(100, 2),
            Point::new(99, 2),
            Point::new(98, 2),
            Point::new(97, 2),
            Point::new(96, 2),
            Point::new(95, 2),
            Point::new(94, 2),
            Point::new(93, 2),
            Point::new(92, 2),
            Point::new(91, 2),
            Point::new(90, 2),
            Point::new(89, 2),
            Point::new(88, 3),
            Point::new(87, 4),
            Point::new(86, 5),
            Point::new(86, 6),
            Point::new(86, 7),
            Point::new(86, 8),
            Point::new(85, 9),
            Point::new(84, 10),
            Point::new(83, 11),
            Point::new(82, 12),
            Point::new(81, 13),
            Point::new(80, 14),
            Point::new(79, 15),
            Point::new(78, 16),
            Point::new(77, 17),
            Point::new(76, 18),
            Point::new(75, 19),
            Point::new(74, 20),
            Point::new(73, 21),
            Point::new(72, 22),
            Point::new(71, 23),
            Point::new(70, 24),
            Point::new(69, 25),
            Point::new(68, 26),
            Point::new(67, 27),
            Point::new(66, 28),
            Point::new(65, 29),
            Point::new(64, 30),
            Point::new(63, 31),
            Point::new(62, 32),
            Point::new(61, 33),
            Point::new(60, 33),
            Point::new(59, 33),
            Point::new(58, 33),
            Point::new(57, 33),
            Point::new(56, 33),
            Point::new(55, 33),
            Point::new(54, 33),
            Point::new(53, 33),
            Point::new(52, 33),
            Point::new(51, 33),
            Point::new(50, 33),
            Point::new(49, 33),
            Point::new(48, 33),
            Point::new(47, 33),
            Point::new(46, 33),
            Point::new(45, 33),
            Point::new(44, 33),
            Point::new(43, 33),
            Point::new(42, 33),
            Point::new(41, 33),
            Point::new(40, 33),
            Point::new(39, 33),
            Point::new(38, 33),
            Point::new(39, 33),
            Point::new(40, 33),
            Point::new(41, 33),
            Point::new(42, 33),
            Point::new(43, 33),
            Point::new(44, 33),
            Point::new(45, 33),
            Point::new(46, 33),
            Point::new(47, 33),
            Point::new(48, 33),
            Point::new(49, 33),
            Point::new(50, 33),
            Point::new(51, 33),
            Point::new(52, 33),
            Point::new(53, 33),
            Point::new(54, 33),
            Point::new(55, 33),
            Point::new(56, 33),
            Point::new(57, 33),
            Point::new(58, 33),
            Point::new(59, 34),
            Point::new(60, 35),
            Point::new(61, 36),
            Point::new(62, 36),
            Point::new(63, 36),
            Point::new(64, 36),
            Point::new(65, 36),
            Point::new(66, 36),
            Point::new(67, 36),
            Point::new(68, 36),
            Point::new(69, 36),
            Point::new(70, 36),
            Point::new(71, 36),
        ];

        let start = Instant::now();
        for loc in locs {
            self.render();
            self.ipc.send_mutate(StateMutators::Bump(PLAYER_OID, loc));
        }
        let elapsed = start.elapsed();
        info!("benchmark took {:?} secs", elapsed.as_secs());
    }

    fn render(&mut self) {
        self.ui.render(&mut self.stdout, &mut self.ipc);
        self.stdout.flush().unwrap();
    }
}

// This restores the terminal state when the process exits but unfortunately it's called
// after panic! backtraces are printed so they are completely mis-formatted. TODO: there
// is a solution in https://werat.dev/blog/pretty-rust-backtraces-in-raw-terminal-mode
// but it's rather messy.
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
