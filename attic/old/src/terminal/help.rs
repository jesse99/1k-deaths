use super::{Color, Line, TextRun};
use fnv::FnvHashSet;
use termion::event::Key;

/// Asserts if the help text is missing a command.
pub fn validate_help<'a>(mode: &str, help: &str, keys: impl Iterator<Item = &'a Key>) {
    let mut errors = Vec::new();
    for key in keys {
        let label = key_to_label(*key);
        let pattern = format!("[[{label}]]");
        let count = help.matches(&pattern).count();
        if count == 0 {
            errors.push(format!("{label} doesn't appear in {mode}'s help"));
        } else if count > 1 {
            errors.push(format!("{label} appears {count} times in {mode}'s help"));
            // this is probably an error
        }
    }
    if !errors.is_empty() {
        panic!("{}", errors.join("\n"));
    }
}

/// Converts a help string into TextRun's. The keys iterator is used for validation.
pub fn format_help<'a>(help: &str, keys: impl Iterator<Item = &'a Key>) -> Vec<Line> {
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

enum HelpSection {
    Text(String),
    Key(String),
}

struct SectionIterator {
    text: Vec<char>,
    index: i32,
}

const EOT: char = '\x04';

fn key_to_label(key: Key) -> String {
    use Key::*;
    match key {
        Char(' ') => "space".to_string(),
        Char('\n') => "return".to_string(),
        Char('\t') => "tab".to_string(),
        Char('?') => "?".to_string(),
        Char(c) => c.to_string(),
        Ctrl(c) => format!("control-{c}"),
        BackTab => "shift-tab".to_string(),
        Esc => "escape".to_string(),
        Left => "left-arrow".to_string(),
        Right => "right-arrow".to_string(),
        Up => "up-arrow".to_string(),
        Down => "down-arrow".to_string(),
        _ => panic!("don't know how to format {key:?}"),
    }
}

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
