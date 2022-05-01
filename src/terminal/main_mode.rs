// use super::details_view::DetailsView;
// use super::help::{format_help, validate_help};
use super::map_view::MapView;
// use super::messages_view::{self, MessagesView};
use super::mode::{InputAction, Mode, RenderContext};
// use super::text_mode::TextMode;
// use super::text_view::{Line, TextRun};
use fnv::FnvHashMap;
use one_thousand_deaths::player::{self, Direction};
use one_thousand_deaths::{Point, Size, State};
// use std::fs::File;
// use std::io::{Error, Write};
// use std::path::Path;
use termion::event::Key;

const NUM_MESSAGES: i32 = 5;

type KeyHandler = fn(&mut MainMode, &mut State) -> InputAction;
type CommandTable = FnvHashMap<Key, Box<KeyHandler>>;

pub struct MainMode {
    map: MapView,
    // details: DetailsView,
    // messages: MessagesView,
    commands: CommandTable,
    // screen_size: Size,
}

impl MainMode {
    pub fn create(width: i32, height: i32) -> Box<dyn Mode> {
        let mut commands: CommandTable = FnvHashMap::default();
        commands.insert(Key::Left, Box::new(|s, state| s.do_bump(state, Direction::West)));
        commands.insert(Key::Right, Box::new(|s, state| s.do_bump(state, Direction::East)));
        commands.insert(Key::Up, Box::new(|s, state| s.do_bump(state, Direction::North)));
        commands.insert(Key::Down, Box::new(|s, state| s.do_bump(state, Direction::South)));
        commands.insert(
            Key::Char('1'),
            Box::new(|s, state| s.do_bump(state, Direction::SouthWest)),
        );
        commands.insert(Key::Char('2'), Box::new(|s, state| s.do_bump(state, Direction::South)));
        commands.insert(
            Key::Char('3'),
            Box::new(|s, state| s.do_bump(state, Direction::SouthEast)),
        );
        commands.insert(Key::Char('4'), Box::new(|s, state| s.do_bump(state, Direction::West)));
        // commands.insert(Key::Char('5'), Box::new(|s, state| s.do_rest(state)));
        // commands.insert(Key::Char('s'), Box::new(|s, state| s.do_rest(state)));
        commands.insert(Key::Char('6'), Box::new(|s, state| s.do_bump(state, Direction::East)));
        commands.insert(
            Key::Char('7'),
            Box::new(|s, state| s.do_bump(state, Direction::NorthWest)),
        );
        commands.insert(Key::Char('8'), Box::new(|s, state| s.do_bump(state, Direction::North)));
        commands.insert(
            Key::Char('9'),
            Box::new(|s, state| s.do_bump(state, Direction::NorthEast)),
        );
        // commands.insert(Key::Char('i'), Box::new(|s, state| s.do_inventory(state)));
        // commands.insert(Key::Char('x'), Box::new(|s, state| s.do_examine(state)));
        // if super::wizard_mode() {
        //     commands.insert(Key::Ctrl('d'), Box::new(|s, state| s.do_save_state(state)));
        // }

        // We don't receive ctrl-m so we use ctrl-p because that's what Crawl does.
        // commands.insert(Key::Ctrl('p'), Box::new(|s, state| s.do_show_messages(state)));
        // commands.insert(Key::Char('?'), Box::new(|s, state| s.do_help(state)));
        commands.insert(Key::Char('q'), Box::new(|s, state| s.do_quit(state)));

        let details_width = 20;
        Box::new(MainMode {
            map: MapView {
                origin: Point::new(0, 0),
                size: Size::new(width - details_width, height - NUM_MESSAGES),
            },
            // details: DetailsView {
            //     origin: Point::new(width - details_width, 0),
            //     size: Size::new(details_width, height - NUM_MESSAGES),
            // },
            // messages: MessagesView {
            //     origin: Point::new(0, height - NUM_MESSAGES),
            //     size: Size::new(width, NUM_MESSAGES),
            // },
            commands,
            // screen_size: Size::new(width, height),
        })
    }
}

impl Mode for MainMode {
    fn render(&self, context: &mut RenderContext) -> bool {
        // self.details.render(context.stdout, context.state); // TODO: views should probably take context
        self.map.render(context.stdout, context.state, context.examined); // TODO: details can write into the next line so this will fix up (which may cause flashing)
                                                                          // self.messages.render(context.stdout, context.state);
        true
    }

    fn input_timeout_ms(&self) -> Option<i32> {
        None
    }

    fn handle_input(&mut self, state: &mut State, key: Key) -> InputAction {
        match self.commands.get(&key).cloned() {
            Some(handler) => handler(self, state),
            None => InputAction::NotHandled,
        }
    }
}

impl MainMode {
    // fn do_examine(&mut self, state: &mut State) -> InputAction {
    //     let loc = state.player_loc();
    //     let window = super::examine_mode::ExamineMode::create(loc);
    //     InputAction::Push(window)
    // }

    //     fn do_help(&mut self, _game: &mut State) -> InputAction {
    //         let mut help = r#"Help for the main game. Note that help is context sensitive,
    // e.g. examine mode has its own set of commands and its own help screen.

    // Movement is done using the numeric keypad or arrow keys:
    // [[7]] [[8]] [[9]]                  [[up-arrow]]
    // [[4]]   [[6]]           [[left-arrow]]   [[right-arrow]]
    // [[1]] [[2]] [[3]]                 [[down-arrow]]

    // [[5]] or [[s]] rest for one turn.
    // [[i]] manage inventory items.
    // [[x]] examine visible cells.
    // [[control-p]] show recent messages.
    // [[?]] show this help.
    // [[q]] save and quit
    // "#
    //         .to_string();
    //         if super::wizard_mode() {
    //             help += r#"

    // Wizard mode commands:
    // [[control-d]] dump game state to state-xxx.txt.
    // "#;
    //         }
    //         validate_help("main", &help, self.commands.keys());

    //         let lines = format_help(&help, self.commands.keys());
    //         InputAction::Push(TextMode::at_top().create(lines))
    //     }

    // fn do_inventory(&mut self, state: &mut State) -> InputAction {
    //     let window = super::inventory_mode::InventoryMode::create(state, self.screen_size);
    //     InputAction::Push(window)
    // }

    fn do_bump(&mut self, state: &mut State, dir: Direction) -> InputAction {
        player::bump(state, dir);
        InputAction::UpdatedGame
    }

    fn do_quit(&mut self, _game: &mut State) -> InputAction {
        InputAction::Quit
    }

    // fn do_rest(&mut self, state: &mut State) -> InputAction {
    //     state.player_acted(Action::Rest);
    //     InputAction::UpdatedGame
    // }

    // fn state_path(&self, base: &str) -> String {
    //     for i in 1..1000 {
    //         let candidate = format!("{base}-{:0>3}.txt", i);
    //         if !Path::new(&candidate).is_file() {
    //             return candidate;
    //         }
    //     }
    //     panic!("Couldn't find a file to dump state in 1K tries!");
    // }

    // fn save_state<W: Write>(&self, path: &str, writer: &mut W, state: &mut State) -> Result<(), Error> {
    //     state.dump_state(writer)?;
    //     state.add_mesg(Message {
    //         topic: Topic::Important,
    //         text: format!("Saved state to {path}"),
    //     });
    //     Ok(())
    // }

    // fn do_save_state(&mut self, state: &mut State) -> InputAction {
    //     let path = self.state_path("state");
    //     if let Err(err) = File::create(&path).and_then(|mut file| self.save_state(&path, &mut file, state)) {
    //         state.add_mesg(Message {
    //             topic: Topic::Error,
    //             text: format!("Couldn't save state to {path}: {err}"),
    //         })
    //     }
    //     InputAction::UpdatedGame
    // }

    // fn do_show_messages(&mut self, state: &mut State) -> InputAction {
    //     fn get_lines(state: &mut State) -> Vec<Line> {
    //         let mut lines = Vec::new();
    //         for message in state.recent_messages(usize::MAX) {
    //             let fg = messages_view::to_fore_color(message.topic);
    //             let line = vec![TextRun::Color(fg), TextRun::Text(message.text.clone())];
    //             lines.push(line);
    //         }
    //         lines
    //     }

    //     let lines = get_lines(state);
    //     InputAction::Push(TextMode::at_bottom().with_bg(Color::White).create(lines))
    // }
}
