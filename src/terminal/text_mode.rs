use super::{format_help, validate_help, Color, InputAction, Line, Mode, RenderContext, TextView};
use crate::backend::Game;
use fnv::FnvHashMap;
use termion::event::Key;

type KeyHandler = fn(&mut TextMode, &mut Game) -> InputAction;
type CommandTable = FnvHashMap<Key, Box<KeyHandler>>;

pub struct TextMode {
    text: TextView,
    commands: CommandTable,
    scroll_by: i32, // for u and d commands
}

pub struct TextModeBuilder {
    at_top: bool,
    bg: Color,
}

impl TextModeBuilder {
    pub fn with_bg(mut self, color: Color) -> TextModeBuilder {
        self.bg = color;
        self
    }

    pub fn create(self, lines: Vec<Line>) -> Box<dyn Mode> {
        Box::new(TextMode::create(lines, self.at_top, self.bg))
    }
}

impl TextMode {
    fn create(lines: Vec<Line>, at_top: bool, bg: Color) -> TextMode {
        let mut commands: CommandTable = FnvHashMap::default();

        // Commands are a subset of those in less, see https://man7.org/linux/man-pages/man1/less.1.html.
        // commands.insert(Key::PageDown, Box::new(|s, game| s.do_page(game, 1))); // TODO: we're not catching these
        commands.insert(Key::Char(' '), Box::new(|s, game| s.do_page(game, 1)));
        commands.insert(Key::Char('f'), Box::new(|s, game| s.do_page(game, 1)));
        commands.insert(Key::Ctrl('f'), Box::new(|s, game| s.do_page(game, 1)));
        commands.insert(Key::Ctrl('v'), Box::new(|s, game| s.do_page(game, 1)));

        // commands.insert(Key::PageUp, Box::new(|s, game| s.do_page(game, -1)));
        commands.insert(Key::Char('b'), Box::new(|s, game| s.do_page(game, -1)));
        commands.insert(Key::Ctrl('b'), Box::new(|s, game| s.do_page(game, -1)));

        commands.insert(Key::Down, Box::new(|s, game| s.do_scroll(game, 1)));
        commands.insert(Key::Char('\n'), Box::new(|s, game| s.do_scroll(game, 1)));
        commands.insert(Key::Char('e'), Box::new(|s, game| s.do_scroll(game, 1)));
        commands.insert(Key::Char('j'), Box::new(|s, game| s.do_scroll(game, 1)));
        commands.insert(Key::Ctrl('e'), Box::new(|s, game| s.do_scroll(game, 1)));
        commands.insert(Key::Ctrl('j'), Box::new(|s, game| s.do_scroll(game, 1)));
        commands.insert(Key::Ctrl('n'), Box::new(|s, game| s.do_scroll(game, 1)));

        commands.insert(Key::Up, Box::new(|s, game| s.do_scroll(game, -1)));
        commands.insert(Key::Char('k'), Box::new(|s, game| s.do_scroll(game, -1)));
        commands.insert(Key::Char('p'), Box::new(|s, game| s.do_scroll(game, -1)));
        commands.insert(Key::Char('y'), Box::new(|s, game| s.do_scroll(game, -1)));
        commands.insert(Key::Ctrl('k'), Box::new(|s, game| s.do_scroll(game, -1)));
        commands.insert(Key::Ctrl('p'), Box::new(|s, game| s.do_scroll(game, -1)));
        commands.insert(Key::Ctrl('y'), Box::new(|s, game| s.do_scroll(game, -1)));

        commands.insert(Key::Char('d'), Box::new(|s, game| s.do_scroll_by(game, 1)));
        commands.insert(Key::Ctrl('d'), Box::new(|s, game| s.do_scroll_by(game, 1)));

        commands.insert(Key::Char('u'), Box::new(|s, game| s.do_scroll_by(game, -1)));
        commands.insert(Key::Ctrl('u'), Box::new(|s, game| s.do_scroll_by(game, -1)));

        // commands.insert(Key::Home, Box::new(|s, game| s.do_scroll_to_start(game)));
        // commands.insert(Key::End, Box::new(|s, game| s.do_scroll_to_end(game)));
        commands.insert(Key::Char('?'), Box::new(|s, game| s.do_help(game)));
        commands.insert(Key::Char('q'), Box::new(|s, game| s.do_pop(game)));
        commands.insert(Key::Esc, Box::new(|s, game| s.do_pop(game)));

        // less supports other good stuff, most of which requires additional user input.
        // Not clear how to do that atm and we want to transition to a web UI so there's
        // not much point in doing too much here. TODO: But the some of the neat things are:
        // d and u allow the user to specify the number of lines (and it becomes the new default)
        // g and G to goto a line
        // /pattern to search for an re
        // ?pattern to search backwards for an re
        // &pattern show only lines that match an re
        // v invoke an editor
        // s save the file to a path

        let mut view = TextView::new(lines, bg);
        if !at_top {
            view.scroll_to_bottom();
        }

        TextMode {
            text: view,
            commands,
            scroll_by: 1,
        }
    }

    pub fn at_top() -> TextModeBuilder {
        TextModeBuilder {
            at_top: true,
            bg: Color::Black,
        }
    }

    pub fn at_bottom() -> TextModeBuilder {
        TextModeBuilder {
            at_top: false,
            bg: Color::Black,
        }
    }
}

impl Mode for TextMode {
    fn render(&self, context: &mut RenderContext) -> bool {
        self.text.render(context.stdout);
        true
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

impl TextMode {
    fn do_help(&mut self, _game: &mut Game) -> InputAction {
        let help = r#"There are a number of keys that allow the screen to be scrolled.

Scroll down by one full screen:
[[space]] or [[f]] or [[control-f]] or [[control-v]]

[[b]] or [[control-b]] scroll up by one full screen.

Scroll down by one line:
[[down-arrow]] or [[return]] or [[d]] or [[e]] or [[j]]
[[control-d]] or [[control-e]] or [[control-j]] or [[control-n]]

Scroll up by one line:
[[up-arrow]] or [[u]] or [[k]] or [[p]] or [[y]]
[[control-u]] or [[control-k]] or [[control-p]] or [[control-y]]

[[?]] show this help.
[[escape]] and [[q]] exit this mode."#;
        validate_help("text", help, self.commands.keys());

        let lines = format_help(help, self.commands.keys());
        InputAction::Push(TextMode::at_top().create(lines))
    }

    fn do_page(&mut self, _game: &mut Game, sign: i32) -> InputAction {
        self.text.scroll(sign * self.text.size().height);
        InputAction::UpdatedGame
    }

    fn do_pop(&mut self, _game: &mut Game) -> InputAction {
        InputAction::Pop
    }

    fn do_scroll(&mut self, _game: &mut Game, delta: i32) -> InputAction {
        self.text.scroll(delta);
        InputAction::UpdatedGame
    }

    fn do_scroll_by(&mut self, _game: &mut Game, sign: i32) -> InputAction {
        self.text.scroll(sign * self.scroll_by);
        InputAction::UpdatedGame
    }
}
