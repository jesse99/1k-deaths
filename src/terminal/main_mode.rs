use super::map_view::MapView;
use super::messages_view::MessagesView;
use super::mode::{InputAction, Mode, RenderContext};
use super::text_mode::TextMode;
use super::text_view::{Line, TextRun};
use crate::backend::{Color, Command, Game, Point, Size};
use fnv::FnvHashMap;
use termion::event::Key;

const NUM_MESSAGES: i32 = 5;

type KeyHandler = fn(&mut MainMode, &mut Game) -> InputAction;
type CommandTable = FnvHashMap<Key, Box<KeyHandler>>;

pub struct MainMode {
    map: MapView,
    messages: MessagesView,
    commands: CommandTable,
}

impl MainMode {
    pub fn create(width: i32, height: i32) -> Box<dyn Mode> {
        let mut commands: CommandTable = FnvHashMap::default();
        commands.insert(Key::Left, Box::new(|s, game| s.do_move(game, -1, 0)));
        commands.insert(Key::Right, Box::new(|s, game| s.do_move(game, 1, 0)));
        commands.insert(Key::Up, Box::new(|s, game| s.do_move(game, 0, -1)));
        commands.insert(Key::Down, Box::new(|s, game| s.do_move(game, 0, 1)));
        commands.insert(Key::Char('1'), Box::new(|s, game| s.do_move(game, -1, 1)));
        commands.insert(Key::Char('2'), Box::new(|s, game| s.do_move(game, 0, 1)));
        commands.insert(Key::Char('3'), Box::new(|s, game| s.do_move(game, 1, 1)));
        commands.insert(Key::Char('4'), Box::new(|s, game| s.do_move(game, -1, 0)));
        commands.insert(Key::Char('6'), Box::new(|s, game| s.do_move(game, 1, 0)));
        commands.insert(Key::Char('7'), Box::new(|s, game| s.do_move(game, -1, -1)));
        commands.insert(Key::Char('8'), Box::new(|s, game| s.do_move(game, 0, -1)));
        commands.insert(Key::Char('9'), Box::new(|s, game| s.do_move(game, 1, -1)));
        commands.insert(Key::Char('x'), Box::new(|s, game| s.do_examine(game)));
        commands.insert(Key::Ctrl('e'), Box::new(|s, game| s.do_show_events(game)));
        commands.insert(Key::Char('q'), Box::new(|s, game| s.do_quit(game)));

        Box::new(MainMode {
            map: MapView {
                origin: Point::new(0, 0),
                size: Size::new(width, height - NUM_MESSAGES),
            },
            messages: MessagesView {
                origin: Point::new(0, height - NUM_MESSAGES),
                size: Size::new(width, NUM_MESSAGES),
            },
            commands,
        })
    }
}

impl Mode for MainMode {
    fn render(&self, context: &mut RenderContext) -> bool {
        self.map.render(context.stdout, context.game, context.examined); // TODO: views should probably take context
        self.messages.render(context.stdout, context.game);
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

impl MainMode {
    fn do_examine(&mut self, game: &mut Game) -> InputAction {
        let loc = game.player();
        let window = super::examine_mode::ExamineMode::create(loc);
        InputAction::Push(window)
    }

    fn do_move(&mut self, game: &mut Game, dx: i32, dy: i32) -> InputAction {
        let mut events = Vec::new();
        game.command(Command::Move { dx, dy }, &mut events);
        game.post(events, false);
        InputAction::UpdatedGame
    }

    fn do_quit(&mut self, _game: &mut Game) -> InputAction {
        InputAction::Quit
    }

    fn do_show_events(&mut self, game: &mut Game) -> InputAction {
        // TODO: Should we load saved events? Do we even want this mode?
        let lines = self.get_events(game);
        InputAction::Push(TextMode::create_at_bottom(lines))
    }

    fn get_events(&mut self, game: &mut Game) -> Vec<Line> {
        let mut lines = Vec::new();
        for e in game.events() {
            let line = vec![TextRun::Color(Color::White), TextRun::Text(e)];
            lines.push(line);
        }
        lines
    }
}
