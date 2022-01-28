use super::{Command, Game, InputAction, Point, RenderContext, Window};
use fnv::FnvHashMap;
use termion::event::Key;

type KeyHandler = fn(&mut ExamineWindow, &mut Game) -> InputAction;
type CommandTable = FnvHashMap<Key, Box<KeyHandler>>;

pub struct ExamineWindow {
    examined: Point,
    commands: CommandTable,
}

impl ExamineWindow {
    pub fn window(examined: Point) -> Box<dyn Window> {
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
        commands.insert(Key::Char('\t'), Box::new(|s, game| s.do_tab_target(game, 1)));
        commands.insert(Key::BackTab, Box::new(|s, game| s.do_tab_target(game, -1)));
        commands.insert(Key::Esc, Box::new(|s, game| s.do_pop(game)));

        Box::new(ExamineWindow { examined, commands })
    }
}

impl Window for ExamineWindow {
    fn render(&self, context: &mut RenderContext) -> bool {
        context.examined = Some(self.examined);
        false
    }

    fn handle_input(&mut self, game: &mut Game, key: termion::event::Key) -> InputAction {
        match self.commands.get(&key).cloned() {
            Some(handler) => handler(self, game),
            None => InputAction::NotHandled,
        }
    }
}

impl ExamineWindow {
    fn do_examine(&mut self, game: &mut Game, dx: i32, dy: i32) -> InputAction {
        let mut events = Vec::new();
        self.examined = Point::new(self.examined.x + dx, self.examined.y + dy);
        game.command(Command::Examine(self.examined), &mut events);
        game.post(events, false);
        InputAction::UpdatedGame
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
