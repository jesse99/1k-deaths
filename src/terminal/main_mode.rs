use super::details_view::DetailsView;
use super::help::{format_help, validate_help};
use super::map_view::MapView;
use super::messages_view::{self, MessagesView};
use super::mode::{InputAction, Mode, RenderContext};
use super::text_mode::TextMode;
use super::text_view::{Line, TextRun};
use fnv::FnvHashMap;
use one_thousand_deaths::{Action, Color, Game, Message, Point, Size, Topic};
use std::fs::File;
use std::io::{Error, Write};
use std::path::Path;
use termion::event::Key;

const NUM_MESSAGES: i32 = 5;

type KeyHandler = fn(&mut MainMode, &mut Game) -> InputAction;
type CommandTable = FnvHashMap<Key, Box<KeyHandler>>;

pub struct MainMode {
    map: MapView,
    details: DetailsView,
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
        commands.insert(Key::Char('5'), Box::new(|s, game| s.do_rest(game)));
        commands.insert(Key::Char('6'), Box::new(|s, game| s.do_move(game, 1, 0)));
        commands.insert(Key::Char('7'), Box::new(|s, game| s.do_move(game, -1, -1)));
        commands.insert(Key::Char('8'), Box::new(|s, game| s.do_move(game, 0, -1)));
        commands.insert(Key::Char('9'), Box::new(|s, game| s.do_move(game, 1, -1)));
        commands.insert(Key::Char('x'), Box::new(|s, game| s.do_examine(game)));
        if super::wizard_mode() {
            commands.insert(Key::Ctrl('d'), Box::new(|s, game| s.do_save_state(game)));
        }

        // We don't receive ctrl-m. We're using ctrl-p because that's what Crawl does.
        commands.insert(Key::Ctrl('p'), Box::new(|s, game| s.do_show_messages(game)));
        commands.insert(Key::Char('?'), Box::new(|s, game| s.do_help(game)));
        commands.insert(Key::Char('q'), Box::new(|s, game| s.do_quit(game)));

        let details_width = 20;
        Box::new(MainMode {
            map: MapView {
                origin: Point::new(0, 0),
                size: Size::new(width - details_width, height - NUM_MESSAGES),
            },
            details: DetailsView {
                origin: Point::new(width - details_width, 0),
                size: Size::new(details_width, height - NUM_MESSAGES),
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
        self.details.render(context.stdout, context.game); // TODO: views should probably take context
        self.map.render(context.stdout, context.game, context.examined); // TODO: details can write into the next line so this will fix up (which may cause flashing)
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
        let loc = game.player_loc();
        let window = super::examine_mode::ExamineMode::create(loc);
        InputAction::Push(window)
    }

    fn do_help(&mut self, _game: &mut Game) -> InputAction {
        let mut help = r#"Help for the main game. Note that help is context sensitive,
e.g. examine mode has its own set of commands and its own help screen.

Movement is done using the numeric keypad or arrow keys:
[[7]] [[8]] [[9]]                  [[up-arrow]]
[[4]]   [[6]]           [[left-arrow]]   [[right-arrow]]
[[1]] [[2]] [[3]]                 [[down-arrow]]

[[5]] rest for one turn.
[[x]] examine visible cells.
[[control-p]] show recent messages.
[[?]] show this help.
[[q]] save and quit
"#
        .to_string();
        if super::wizard_mode() {
            help += r#"

Wizard mode commands:
[[control-d]] dump game state to state-xxx.txt.
"#;
        }
        validate_help("main", &help, self.commands.keys());

        let lines = format_help(&help, self.commands.keys());
        InputAction::Push(TextMode::at_top().create(lines))
    }

    fn do_move(&mut self, game: &mut Game, dx: i32, dy: i32) -> InputAction {
        game.player_acted(Action::Move { dx, dy });
        InputAction::UpdatedGame
    }

    fn do_quit(&mut self, _game: &mut Game) -> InputAction {
        InputAction::Quit
    }

    fn do_rest(&mut self, game: &mut Game) -> InputAction {
        game.player_acted(Action::Rest);
        InputAction::UpdatedGame
    }

    fn state_path(&self, base: &str) -> String {
        for i in 1..1000 {
            let candidate = format!("{base}-{:0>3}.txt", i);
            if !Path::new(&candidate).is_file() {
                return candidate;
            }
        }
        panic!("Couldn't find a file to dump state in 1K tries!");
    }

    fn save_state<W: Write>(&self, path: &str, writer: &mut W, game: &mut Game) -> Result<(), Error> {
        game.dump_state(writer)?;
        game.add_mesg(Message {
            topic: Topic::Important,
            text: format!("Saved state to {path}"),
        });
        Ok(())
    }

    fn do_save_state(&mut self, game: &mut Game) -> InputAction {
        let path = self.state_path("state");
        if let Err(err) = File::create(&path).and_then(|mut file| self.save_state(&path, &mut file, game)) {
            game.add_mesg(Message {
                topic: Topic::Error,
                text: format!("Couldn't save state to {path}: {err}"),
            })
        }
        InputAction::UpdatedGame
    }

    fn do_show_messages(&mut self, game: &mut Game) -> InputAction {
        fn get_lines(game: &mut Game) -> Vec<Line> {
            let mut lines = Vec::new();
            for message in game.recent_messages(usize::MAX) {
                let fg = messages_view::to_fore_color(message.topic);
                let line = vec![TextRun::Color(fg), TextRun::Text(message.text.clone())];
                lines.push(line);
            }
            lines
        }

        let lines = get_lines(game);
        InputAction::Push(TextMode::at_bottom().with_bg(Color::White).create(lines))
    }
}
