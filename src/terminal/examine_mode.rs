use super::help::{format_help, validate_help};
use super::mode::{InputAction, Mode, RenderContext};
use super::text_mode::TextMode;
use fnv::FnvHashMap;
use one_thousand_deaths::{Action, Game, Point};
use termion::event::Key;

type KeyHandler = fn(&mut ExamineMode, &mut Game) -> InputAction;
type CommandTable = FnvHashMap<Key, Box<KeyHandler>>;

pub struct ExamineMode {
    examined: Point,
    commands: CommandTable,
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
        self.examined = Point::new(self.examined.x + dx, self.examined.y + dy);
        game.player_acted(Action::Examine {
            loc: self.examined,
            wizard: super::wizard_mode(),
        });
        InputAction::UpdatedGame
    }

    fn do_help(&mut self, _game: &mut Game) -> InputAction {
        let help = r#"Move the focus to examine the contents of a cell.
The focus is drawn with reversed colors.

The focus can be moved with the usual keys:
[[7]] [[8]] [[9]]                  [[up-arrow]]
[[4]]   [[6]]           [[left-arrow]]   [[right-arrow]]
[[1]] [[2]] [[3]]                 [[down-arrow]]

[[tab]] can be used to select the next character.
[[shift-tab]] can be used to select the previous character.
[[?]] show this help.
[[q]] save and quit.
[[escape]] exits examine mode."#;
        validate_help("examine", help, self.commands.keys());

        let lines = format_help(help, self.commands.keys());
        InputAction::Push(TextMode::at_top().create(lines))
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
            game.player_acted(Action::Examine {
                loc,
                wizard: super::wizard_mode(),
            });
        }
        InputAction::UpdatedGame
    }
}
