use super::*;
use fnv::FnvHashMap;
// use std::fs::File;
// use std::io::{Error, Write};
// use std::path::Path;
use std::cell::RefCell;
use termion::event::Key;

const NUM_MESSAGES: i32 = 5;

pub struct MainMode {
    map: MapView,
    // details: DetailsView,
    messages: MessagesView,
    commands: CommandTable,
    // screen_size: Size,
}

impl MainMode {
    pub fn create(width: i32, height: i32) -> Box<dyn Mode> {
        let mut commands = FnvHashMap::default();

        commands.insert(Key::Left, Command::Bump(-1, 0));
        commands.insert(Key::Right, Command::Bump(1, 0));
        commands.insert(Key::Up, Command::Bump(0, -1));
        commands.insert(Key::Down, Command::Bump(0, 1));
        commands.insert(Key::Char('1'), Command::Bump(-1, 1));
        commands.insert(Key::Char('2'), Command::Bump(0, 1));
        commands.insert(Key::Char('3'), Command::Bump(1, 1));
        commands.insert(Key::Char('4'), Command::Bump(-1, 0));
        // commands.insert(Key::Char('5'), Command::Rest);
        // commands.insert(Key::Char('s'), Command::Rest);
        commands.insert(Key::Char('6'), Command::Bump(1, 0));
        commands.insert(Key::Char('7'), Command::Bump(-1, -1));
        commands.insert(Key::Char('8'), Command::Bump(0, -1));
        commands.insert(Key::Char('9'), Command::Bump(1, -1));
        // commands.insert(Key::Char('i'), Command::Inventory);
        // commands.insert(Key::Char('x'), Command::Examine);
        // if super::wizard_mode() {
        //     commands.insert(Key::Ctrl('d'), Command::SaveState);
        // }

        // We don't receive ctrl-m so we use ctrl-p because that's what Crawl does.
        // Key::Ctrl('p'), Command::ShowMessages);
        commands.insert(Key::Char('?'), Command::Help);
        commands.insert(Key::Char('q'), Command::Quit);

        // let details_width = 20;
        let details_width = 0;
        Box::new(MainMode {
            map: MapView {
                origin: Point::new(0, 0),
                size: Size::new(width - details_width, height - NUM_MESSAGES),
                string_cache: RefCell::new(FnvHashMap::default()),
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

    // TODO: use a commands table
    fn handle_input(&self, key: Key) -> Option<Command> {
        self.commands.get(&key).copied()
    }

    fn handle_command(&mut self, ipc: &IPC, command: Command) -> CommandResult {
        match command {
            Command::Bump(dx, dy) => self.do_bump(ipc, dx, dy),
            Command::Help => self.do_help(ipc),
            Command::Quit => self.do_quit(ipc),
            _ => panic!("didn't handle {command:?}"),
        }
    }

    fn input_timeout_ms(&self) -> Option<i32> {
        None
    }
}

impl MainMode {
    // fn do_examine(&mut self, ipc: &IPC) -> CommandResult {
    //     let loc = ipc.player_loc();
    //     let window = super::examine_mode::ExamineMode::create(loc);
    //     CommandResult::Push(window)
    // }

    // TODO: help commands to be supported
    // [[5]] or [[s]] rest for one turn.
    // [[i]] manage inventory items.
    // [[x]] examine visible cells.
    // [[control-p]] show recent messages.
    fn do_help(&mut self, _ipc: &IPC) -> CommandResult {
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
        CommandResult::Push(TextMode::at_top().create(lines))
    }

    // fn do_inventory(&mut self, ipc: &IPC) -> CommandResult {
    //     let window = super::inventory_mode::InventoryMode::create(ipc, self.screen_size);
    //     CommandResult::Push(window)
    // }

    fn do_bump(&mut self, ipc: &IPC, dx: i32, dy: i32) -> CommandResult {
        let mut loc = ipc.get_player_loc();
        loc.x += dx;
        loc.y += dy;
        // info!(
        //     "ipc.send_mutate(StateMutators::Bump(PLAYER_ID, Point::new({}, {})));",
        //     loc.x, loc.y
        // );
        ipc.send_mutate(StateMutators::Bump(loc));
        CommandResult::UpdatedGame
    }

    fn do_quit(&mut self, _ipc: &IPC) -> CommandResult {
        CommandResult::Quit
    }

    // fn do_rest(&mut self, ipc: &IPC) -> CommandResult {
    //     ipc.player_acted(Action::Rest);
    //     CommandResult::UpdatedGame
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
    // fn do_save_state(&mut self, ipc: &IPC) -> CommandResult {
    //     let path = self.state_path("state");
    //     info!("dumped game to {path}");
    //     if let Err(err) = File::create(&path).and_then(|mut file| self.save_state(&path, &mut file, ipc)) {
    //         ipc.add_message(Message {
    //             kind: MessageKind::Error,
    //             text: format!("Couldn't save state to {path}: {err}"),
    //         })
    //     }
    //     CommandResult::UpdatedGame
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
