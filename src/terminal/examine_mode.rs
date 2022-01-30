use super::mode::{InputAction, Mode, RenderContext};
use super::text_mode::TextMode;
use super::text_view::{Line, TextRun};
use crate::backend::Color;
use crate::backend::{Command, Game, Point};
use fnv::{FnvHashMap, FnvHashSet};
use termion::event::Key;

type KeyHandler = fn(&mut ExamineMode, &mut Game) -> InputAction;
type CommandTable = FnvHashMap<Key, Box<KeyHandler>>;

pub struct ExamineMode {
    examined: Point,
    commands: CommandTable,
}

fn key_to_label(key: Key) -> String {
    match key {
        Key::Char('\t') => "tab".to_string(),
        Key::Char('?') => "?".to_string(),
        Key::Char(c) => c.to_string(),
        Key::BackTab => "shift-tab".to_string(),
        Key::Esc => "escape".to_string(),
        Key::Left => "left-arrow".to_string(),
        Key::Right => "right-arrow".to_string(),
        Key::Up => "up-arrow".to_string(),
        Key::Down => "down-arrow".to_string(),
        _ => panic!("don't know how to format {key:?}"),
    }
}

/// Asserts if the help text is missing a command.
fn validate_help<'a>(help: &str, keys: impl Iterator<Item = &'a Key>) {
    let mut errors = Vec::new();
    for key in keys {
        let label = key_to_label(*key);
        let pattern = format!("[[{label}]]");
        let count = help.matches(&pattern).count();
        if count == 0 {
            errors.push(format!("{label} doesn't appear in examine's help"));
        } else if count > 1 {
            errors.push(format!("{label} appears {count} times in examine's help"));
            // this is probably an error
        }
    }
    if !errors.is_empty() {
        panic!("{}", errors.join("\n"));
    }
}

enum HelpSection {
    Text(String),
    Key(String),
}

struct SectionIterator {
    text: Vec<char>,
    index: i32,
}

const EOT: char = '\x04';

impl SectionIterator {
    fn new(text: &str) -> SectionIterator {
        SectionIterator {
            text: text.chars().collect(),
            index: 0,
        }
    }

    fn at(&self, delta: i32) -> char {
        let i = (self.index + delta) as usize;
        if i < self.text.len() {
            self.text[i]
        } else {
            EOT
        }
    }

    fn collect_text(&mut self) -> String {
        let mut text = String::new();

        while self.at(0) != EOT && !(self.at(0) == '[' && self.at(1) == '[') {
            text.push(self.at(0));
            self.index += 1;
        }

        text
    }

    fn collect_key(&mut self) -> String {
        let mut text = String::new();

        while !(self.at(0) == ']' && self.at(1) == ']') {
            assert!(self.at(0) != EOT, "missing ]]");
            text.push(self.at(0));
            self.index += 1;
        }
        self.index += 2;

        assert!(!text.is_empty(), "no key in double brackets");
        text
    }
}

impl Iterator for SectionIterator {
    type Item = HelpSection;

    fn next(&mut self) -> Option<Self::Item> {
        if self.at(0) == '[' && self.at(1) == '[' {
            self.index += 2;
            Some(HelpSection::Key(self.collect_key()))
        } else if self.at(0) != EOT {
            Some(HelpSection::Text(self.collect_text()))
        } else {
            None
        }
    }
}

fn format_help<'a>(help: &str, keys: impl Iterator<Item = &'a Key>) -> Vec<Line> {
    let mut lines = Vec::new();

    let mut errors = Vec::new();
    let keys: FnvHashSet<String> = keys.map(|k| key_to_label(*k)).collect();
    for line in help.lines() {
        let mut text_runs = Vec::new();

        for section in SectionIterator::new(line) {
            match section {
                HelpSection::Text(s) => {
                    text_runs.push(TextRun::Color(Color::White));
                    text_runs.push(TextRun::Text(s));
                }
                HelpSection::Key(s) => {
                    if keys.contains(&s) {
                        text_runs.push(TextRun::Color(Color::Yellow));
                        text_runs.push(TextRun::Text(s));
                    } else {
                        errors.push(format!("Help uses [[{s}]] which isn't in the command table"));
                    }
                }
            }
        }
        lines.push(text_runs);
    }
    if !errors.is_empty() {
        panic!("{}", errors.join("\n"));
    }

    lines
}

impl ExamineMode {
    pub fn create(examined: Point) -> Box<dyn Mode> {
        let mut commands: CommandTable = FnvHashMap::default();
        commands.insert(Key::Left, Box::new(|s, game| s.do_examine(game, -1, 0)));
        commands.insert(Key::Right, Box::new(|s, game| s.do_examine(game, 1, 0)));
        commands.insert(Key::Up, Box::new(|s, game| s.do_examine(game, 0, -1)));
        commands.insert(Key::Down, Box::new(|s, game| s.do_examine(game, 0, 1)));
        commands.insert(Key::Char('1'), Box::new(|s, game| s.do_examine(game, -1, 1)));
        commands.insert(Key::Char('2'), Box::new(|s, game| s.do_examine(game, 0, 1)));
        commands.insert(Key::Char('3'), Box::new(|s, game| s.do_examine(game, 1, 1)));
        commands.insert(Key::Char('4'), Box::new(|s, game| s.do_examine(game, -1, 0)));
        commands.insert(Key::Char('6'), Box::new(|s, game| s.do_examine(game, 1, 0)));
        commands.insert(Key::Char('7'), Box::new(|s, game| s.do_examine(game, -1, -1)));
        commands.insert(Key::Char('8'), Box::new(|s, game| s.do_examine(game, 0, -1)));
        commands.insert(Key::Char('9'), Box::new(|s, game| s.do_examine(game, 1, -1)));
        commands.insert(Key::Char('q'), Box::new(|s, game| s.do_quit(game)));
        commands.insert(Key::Char('?'), Box::new(|s, game| s.do_help(game)));
        commands.insert(Key::Char('\t'), Box::new(|s, game| s.do_tab_target(game, 1)));
        commands.insert(Key::BackTab, Box::new(|s, game| s.do_tab_target(game, -1)));
        commands.insert(Key::Esc, Box::new(|s, game| s.do_pop(game)));

        Box::new(ExamineMode { examined, commands })
    }
}

impl Mode for ExamineMode {
    fn render(&self, context: &mut RenderContext) -> bool {
        context.examined = Some(self.examined);
        false
    }

    fn input_timeout_ms(&self) -> Option<i32> {
        None
    }

    fn handle_input(&mut self, game: &mut Game, key: Key) -> InputAction {
        match self.commands.get(&key).cloned() {
            Some(handler) => handler(self, game),
            None => InputAction::NotHandled,
        }
    }
}

impl ExamineMode {
    fn do_examine(&mut self, game: &mut Game, dx: i32, dy: i32) -> InputAction {
        let mut events = Vec::new();
        self.examined = Point::new(self.examined.x + dx, self.examined.y + dy);
        game.command(Command::Examine(self.examined), &mut events);
        game.post(events, false);
        InputAction::UpdatedGame
    }

    fn do_help(&mut self, _game: &mut Game) -> InputAction {
        let help = r#"Move the focus to examine the contents of a cell.
The focus is drawn with reversed colors and can be
moved using the normal movement keys:

[[left-arrow]] or [[4]]
move west

[[right-arrow]] or [[6]]
move east

[[up-arrow]] or [[8]]
move north

[[down-arrow]] or [[2]]
move south

[[7]]
move north-west

[[9]]
move north-east

[[1]]
move south-west

[[3]]
move south-east

[[tab]] can be used to select the next character.
[[shift-tab]] can be used to select the previous character.
[[?]] show this help.
[[q]] will quit the command.
[[escape]] exits examine mode."#;
        validate_help(help, self.commands.keys());

        let lines = format_help(help, self.commands.keys());
        InputAction::Push(TextMode::create_at_top(lines))
    }

    fn do_pop(&mut self, _game: &mut Game) -> InputAction {
        InputAction::Pop
    }

    fn do_quit(&mut self, _game: &mut Game) -> InputAction {
        InputAction::Quit
    }

    fn do_tab_target(&mut self, game: &mut Game, delta: i32) -> InputAction {
        if let Some(loc) = game.target_next(&self.examined, delta) {
            self.examined = loc;

            let mut events = Vec::new();
            game.command(Command::Examine(loc), &mut events);
            game.post(events, false);
        }
        InputAction::UpdatedGame
    }
}
