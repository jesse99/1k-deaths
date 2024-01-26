use super::*;
use fnv::FnvHashMap;
// use std::fs::File;
// use std::io::{Error, Write};
// use std::path::Path;
use termion::event::Key;

const NUM_MESSAGES: i32 = 5;

type KeyHandler = fn(&mut MainMode, &IPC) -> InputAction;
type CommandTable = FnvHashMap<Key, Box<KeyHandler>>;

pub struct MainMode {
    map: MapView,
    // details: DetailsView,
    messages: MessagesView,
    commands: CommandTable,
    // screen_size: Size,
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
        // commands.insert(Key::Char('5'), Box::new(|s, game| s.do_rest(game)));
        // commands.insert(Key::Char('s'), Box::new(|s, game| s.do_rest(game)));
        commands.insert(Key::Char('6'), Box::new(|s, game| s.do_move(game, 1, 0)));
        commands.insert(Key::Char('7'), Box::new(|s, game| s.do_move(game, -1, -1)));
        commands.insert(Key::Char('8'), Box::new(|s, game| s.do_move(game, 0, -1)));
        commands.insert(Key::Char('9'), Box::new(|s, game| s.do_move(game, 1, -1)));
        // commands.insert(Key::Char('i'), Box::new(|s, game| s.do_inventory(game)));
        // commands.insert(Key::Char('x'), Box::new(|s, game| s.do_examine(game)));
        // if super::wizard_mode() {
        //     commands.insert(Key::Ctrl('d'), Box::new(|s, game| s.do_save_state(game)));
        // }

        // We don't receive ctrl-m so we use ctrl-p because that's what Crawl does.
        // commands.insert(Key::Ctrl('p'), Box::new(|s, game| s.do_show_messages(game)));
        commands.insert(Key::Char('?'), Box::new(|s, game| s.do_help(game)));
        commands.insert(Key::Char('q'), Box::new(|s, game| s.do_quit(game)));

        // let details_width = 20;
        let details_width = 0;
        Box::new(MainMode {
            map: MapView {
                origin: Point::new(0, 0),
                size: Size::new(width - details_width, height - NUM_MESSAGES),
            },
            // details: DetailsView {
            //     origin: Point::new(width - details_width, 0),
            //     size: Size::new(details_width, height - NUM_MESSAGES),
            // },
            messages: MessagesView {
                origin: Point::new(0, height - NUM_MESSAGES),
                size: Size::new(width, NUM_MESSAGES),
            },
            commands,
            // screen_size: Size::new(width, height),
        })
    }
}

impl Mode for MainMode {
    fn render(&self, context: &mut RenderContext) -> bool {
        // self.details.render(context.stdout, context.game); // TODO: views should probably take context
        self.map.render(context.stdout, context.ipc, context.examined); // TODO: details can write into the next line so this will fix up (which may cause flashing)
        self.messages.render(context.stdout, context.ipc);
        true
    }

    fn handle_input(&mut self, ipc: &IPC, key: Key) -> InputAction {
        match self.commands.get(&key).cloned() {
            Some(handler) => handler(self, ipc),
            None => InputAction::NotHandled,
        }
    }

    fn input_timeout_ms(&self) -> Option<i32> {
        None
    }
}

impl MainMode {
    // fn do_examine(&mut self, ipc: &IPC) -> InputAction {
    //     let loc = ipc.player_loc();
    //     let window = super::examine_mode::ExamineMode::create(loc);
    //     InputAction::Push(window)
    // }

    // TODO: help commands to be supported
    // [[5]] or [[s]] rest for one turn.
    // [[i]] manage inventory items.
    // [[x]] examine visible cells.
    // [[control-p]] show recent messages.
    fn do_help(&mut self, _ipc: &IPC) -> InputAction {
        let help = r#"Help for the main game. Note that help is context sensitive,
    e.g. examine mode has its own set of commands and its own help screen.

    Movement is done using the numeric keypad or arrow keys:
    [[7]] [[8]] [[9]]                  [[up-arrow]]
    [[4]]   [[6]]           [[left-arrow]]   [[right-arrow]]
    [[1]] [[2]] [[3]]                 [[down-arrow]]

    [[?]] show help for help.
    [[q]] save and quit
    "#
        .to_string();
        //         if super::wizard_mode() {
        //             help += r#"

        // Wizard mode commands:
        // [[control-d]] dump game state to state-xxx.txt.
        // "#;
        //         }
        validate_help("main", &help, self.commands.keys());

        let lines = format_help(&help, self.commands.keys());
        InputAction::Push(TextMode::at_top().create(lines))
    }

    // fn do_inventory(&mut self, ipc: &IPC) -> InputAction {
    //     let window = super::inventory_mode::InventoryMode::create(ipc, self.screen_size);
    //     InputAction::Push(window)
    // }

    fn do_move(&mut self, ipc: &IPC, dx: i32, dy: i32) -> InputAction {
        let mut loc = ipc.get_player_loc();
        loc.x += dx;
        loc.y += dy;
        ipc.send_mutate(StateMutators::Bump(PLAYER_ID, loc));
        InputAction::UpdatedGame
    }

    fn do_quit(&mut self, _ipc: &IPC) -> InputAction {
        InputAction::Quit
    }

    // fn do_rest(&mut self, ipc: &IPC) -> InputAction {
    //     ipc.player_acted(Action::Rest);
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

    // fn save_state<W: Write>(&self, path: &str, writer: &mut W, ipc: &IPC) -> Result<(), Error> {
    //     ipc.dump_state(writer)?;
    //     ipc.add_message(Message {
    //         kind: MessageKind::Important,
    //         text: format!("Saved state to {path}"),
    //     });
    //     Ok(())
    // }

    // // Dumps game state into a human readable file.
    // fn do_save_state(&mut self, ipc: &IPC) -> InputAction {
    //     let path = self.state_path("state");
    //     info!("dumped game to {path}");
    //     if let Err(err) = File::create(&path).and_then(|mut file| self.save_state(&path, &mut file, ipc)) {
    //         ipc.add_message(Message {
    //             kind: MessageKind::Error,
    //             text: format!("Couldn't save state to {path}: {err}"),
    //         })
    //     }
    //     InputAction::UpdatedGame
    // }

    // fn do_show_messages(&mut self, ipc: &IPC) -> InputAction {
    //     fn get_lines(ipc: &IPC) -> Vec<Line> {
    //         let mut lines = Vec::new();
    //         for message in ipc.recent_messages(usize::MAX) {
    //             let fg = messages_view::to_fore_color(message.topic);
    //             let line = vec![TextRun::Color(fg), TextRun::Text(message.text.clone())];
    //             lines.push(line);
    //         }
    //         lines
    //     }

    //     let lines = get_lines(ipc);
    //     InputAction::Push(TextMode::at_bottom().with_bg(Color::White).create(lines))
    // }
}
