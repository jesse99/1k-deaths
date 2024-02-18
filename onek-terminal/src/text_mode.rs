use super::*;
use termion::event::Key;

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
    // pub fn with_bg(mut self, color: Color) -> TextModeBuilder {
    //     self.bg = color;
    //     self
    // }

    pub fn create(self, lines: Vec<Line>) -> Box<dyn Mode> {
        Box::new(TextMode::create(lines, self.at_top, self.bg))
    }
}

impl TextMode {
    fn create(lines: Vec<Line>, at_top: bool, bg: Color) -> TextMode {
        let mut commands = CommandTable::default();

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

        // Commands are a subset of those in less, see https://man7.org/linux/man-pages/man1/less.1.html.
        // commands.insert(Key::PageDown, Box::new(|s, game| s.do_page(game, 1))); // TODO: we're not catching these
        commands.insert(Key::Char(' '), Command::Page(1));
        commands.insert(Key::Char('f'), Command::Page(1));
        commands.insert(Key::Ctrl('f'), Command::Page(1));
        commands.insert(Key::Ctrl('v'), Command::Page(1));

        // commands.insert(Key::PageUp, Command::Page(-1));
        commands.insert(Key::Char('b'), Command::Page(-1));
        commands.insert(Key::Ctrl('b'), Command::Page(-1));

        commands.insert(Key::Down, Command::Scroll(1));
        commands.insert(Key::Char('\n'), Command::Scroll(1));
        commands.insert(Key::Char('e'), Command::Scroll(1));
        commands.insert(Key::Char('j'), Command::Scroll(1));
        commands.insert(Key::Ctrl('e'), Command::Scroll(1));
        commands.insert(Key::Ctrl('j'), Command::Scroll(1));
        commands.insert(Key::Ctrl('n'), Command::Scroll(1));

        commands.insert(Key::Up, Command::Scroll(-1));
        commands.insert(Key::Char('k'), Command::Scroll(-1));
        commands.insert(Key::Char('p'), Command::Scroll(-1));
        commands.insert(Key::Char('y'), Command::Scroll(-1));
        commands.insert(Key::Ctrl('k'), Command::Scroll(-1));
        commands.insert(Key::Ctrl('p'), Command::Scroll(-1));
        commands.insert(Key::Ctrl('y'), Command::Scroll(-1));

        commands.insert(Key::Char('d'), Command::ScrollBy(1));
        commands.insert(Key::Ctrl('d'), Command::ScrollBy(1));

        commands.insert(Key::Char('u'), Command::ScrollBy(-1));
        commands.insert(Key::Ctrl('u'), Command::ScrollBy(-1));

        // commands.insert(Key::Home, Command::ScrollToStart);
        // commands.insert(Key::End, Command::ScrollToEnd);
        commands.insert(Key::Char('?'), Command::Help);
        commands.insert(Key::Char('q'), Command::Quit);
        commands.insert(Key::Esc, Command::Quit);

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

    // pub fn at_bottom() -> TextModeBuilder {
    //     TextModeBuilder {
    //         at_top: false,
    //         bg: Color::Black,
    //     }
    // }
}

impl Mode for TextMode {
    fn render(&self, context: &mut RenderContext) -> bool {
        self.text.render(context.stdout);
        true
    }

    fn input_timeout_ms(&self) -> Option<i32> {
        None
    }

    fn handle_input(&self, key: Key) -> Option<Command> {
        self.commands.get(&key).copied()
    }

    fn handle_command(&mut self, ipc: &IPC, command: Command) -> CommandResult {
        match command {
            Command::Help => self.do_help(ipc),
            Command::Page(sign) => self.do_page(ipc, sign),
            Command::Scroll(delta) => self.do_scroll(ipc, delta),
            Command::ScrollBy(sign) => self.do_scroll_by(ipc, sign),
            Command::Quit => self.do_pop(ipc),
            _ => panic!("didn't handle {command:?}"),
        }
    }
}

impl TextMode {
    fn do_help(&mut self, _ipc: &IPC) -> CommandResult {
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
        CommandResult::Push(TextMode::at_top().create(lines))
    }

    fn do_page(&mut self, _ipc: &IPC, sign: i32) -> CommandResult {
        self.text.scroll(sign * self.text.size().height);
        CommandResult::UpdatedGame
    }

    fn do_pop(&mut self, _ipc: &IPC) -> CommandResult {
        CommandResult::Pop
    }

    fn do_scroll(&mut self, _ipc: &IPC, delta: i32) -> CommandResult {
        self.text.scroll(delta);
        CommandResult::UpdatedGame
    }

    fn do_scroll_by(&mut self, _ipc: &IPC, sign: i32) -> CommandResult {
        self.text.scroll(sign * self.scroll_by);
        CommandResult::UpdatedGame
    }
}
