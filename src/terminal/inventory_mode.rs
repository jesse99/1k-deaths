use super::help::{format_help, validate_help};
use super::inventory_view::{InventoryView, SelectedItem};
use super::mode::{InputAction, Mode, RenderContext};
use super::text_mode::TextMode;
use fnv::FnvHashMap;
use one_thousand_deaths::{Game, InvItems, Point, Size};
use termion::event::Key;

type KeyHandler = fn(&mut InventoryMode, &mut Game) -> InputAction;
type CommandTable = FnvHashMap<Key, Box<KeyHandler>>;

pub struct InventoryMode {
    commands: CommandTable,
    view: InventoryView,
    selected: SelectedItem,
}

impl InventoryMode {
    pub fn create(game: &Game, size: Size) -> Box<dyn Mode> {
        let mut commands: CommandTable = FnvHashMap::default();
        commands.insert(Key::Left, Box::new(|s, game| s.do_select(game, -1, 0)));
        commands.insert(Key::Right, Box::new(|s, game| s.do_select(game, 1, 0)));
        commands.insert(Key::Up, Box::new(|s, game| s.do_select(game, 0, -1)));
        commands.insert(Key::Down, Box::new(|s, game| s.do_select(game, 0, 1)));
        commands.insert(Key::Char('1'), Box::new(|s, game| s.do_select(game, -1, 1)));
        commands.insert(Key::Char('2'), Box::new(|s, game| s.do_select(game, 0, 1)));
        commands.insert(Key::Char('3'), Box::new(|s, game| s.do_select(game, 1, 1)));
        commands.insert(Key::Char('4'), Box::new(|s, game| s.do_select(game, -1, 0)));
        commands.insert(Key::Char('6'), Box::new(|s, game| s.do_select(game, 1, 0)));
        commands.insert(Key::Char('7'), Box::new(|s, game| s.do_select(game, -1, -1)));
        commands.insert(Key::Char('8'), Box::new(|s, game| s.do_select(game, 0, -1)));
        commands.insert(Key::Char('9'), Box::new(|s, game| s.do_select(game, 1, -1)));
        commands.insert(Key::Char('?'), Box::new(|s, game| s.do_help(game)));
        commands.insert(Key::Esc, Box::new(|s, game| s.do_pop(game)));

        let origin = Point::new(1, 1);
        let view = InventoryView { origin, size };
        let selected = SelectedItem::None;
        let mut mode = InventoryMode {
            commands,
            view,
            selected,
        };
        mode.do_select(game, 0, 1);
        Box::new(mode)
    }
}

impl Mode for InventoryMode {
    fn render(&self, context: &mut RenderContext) -> bool {
        self.view.render(self.selected, context.stdout, context.game);
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

impl InventoryMode {
    fn do_help(&mut self, _game: &mut Game) -> InputAction {
        let help = r#"Used to manage the items you've picked up.

Selection can be moved using the numeric keypad or arrow keys:
[[7]] [[8]] [[9]]                  [[up-arrow]]
[[4]]   [[6]]           [[left-arrow]]   [[right-arrow]]
[[1]] [[2]] [[3]]                 [[down-arrow]]

[[?]] shows this help.
[[escape]] exits the inventory screen."#;
        validate_help("inventory", help, self.commands.keys());

        let lines = format_help(help, self.commands.keys());
        InputAction::Push(TextMode::at_top().create(lines))
    }

    fn do_pop(&mut self, _game: &mut Game) -> InputAction {
        InputAction::Pop
    }

    fn do_select(&mut self, game: &Game, dx: i32, dy: i32) -> InputAction {
        let inv = game.inventory();
        if dx == 1 {
            self.do_select_next_col(&inv);
        } else if dx == -1 {
            self.do_select_prev_col(&inv);
        }
        if dy == 1 {
            self.do_select_next_row(&inv);
        } else if dy == -1 {
            self.do_select_prev_row(&inv);
        }
        InputAction::UpdatedGame
    }

    fn do_select_next_col(&mut self, items: &InvItems) {
        match self.selected {
            SelectedItem::None => {
                if !items.other.is_empty() {
                    self.selected = SelectedItem::Other(0);
                } else if !items.weapons.is_empty() {
                    self.selected = SelectedItem::Weapon(0);
                } else if !items.armor.is_empty() {
                    self.selected = SelectedItem::Armor(0);
                }
            }
            SelectedItem::Armor(i) | SelectedItem::Weapon(i) => {
                let count = items.other.len();
                if count > 0 {
                    if i < count {
                        self.selected = SelectedItem::Other(i);
                    } else {
                        self.selected = SelectedItem::Other(count - 1);
                    }
                }
            }
            SelectedItem::Other(_) => {
                self.do_select_prev_col(items); // wrap around
            }
        }
    }

    fn do_select_prev_col(&mut self, items: &InvItems) {
        match self.selected {
            SelectedItem::None => {
                if !items.weapons.is_empty() {
                    self.selected = SelectedItem::Weapon(0);
                } else if !items.armor.is_empty() {
                    self.selected = SelectedItem::Armor(0);
                } else if !items.other.is_empty() {
                    self.selected = SelectedItem::Other(0);
                }
            }
            SelectedItem::Other(i) => {
                let count1 = items.weapons.len();
                let count2 = items.armor.len();
                if i < count1 {
                    self.selected = SelectedItem::Weapon(i);
                } else if count2 == 0 {
                    if count1 > 0 {
                        self.selected = SelectedItem::Weapon(count1 - 1);
                    }
                } else if i < count1 + count2 {
                    self.selected = SelectedItem::Armor(i - count1);
                } else if count2 > 0 {
                    self.selected = SelectedItem::Armor(count2 - 1);
                }
            }
            SelectedItem::Weapon(_) | SelectedItem::Armor(_) => {
                self.do_select_next_col(items); // wrap around
            }
        }
    }

    fn do_select_next_row(&mut self, items: &InvItems) {
        match self.selected {
            SelectedItem::None => {
                if !items.weapons.is_empty() {
                    self.selected = SelectedItem::Weapon(0);
                } else if !items.armor.is_empty() {
                    self.selected = SelectedItem::Armor(0);
                } else if !items.other.is_empty() {
                    self.selected = SelectedItem::Other(0);
                }
            }
            SelectedItem::Weapon(i) => {
                let count1 = items.weapons.len();
                let count2 = items.armor.len();
                if i + 1 < count1 {
                    self.selected = SelectedItem::Weapon(i + 1);
                } else if count2 > 0 {
                    self.selected = SelectedItem::Armor(0);
                } else {
                    self.selected = SelectedItem::Weapon(0);
                }
            }
            SelectedItem::Armor(i) => {
                let count1 = items.weapons.len();
                let count2 = items.armor.len();
                if i + 1 < count2 {
                    self.selected = SelectedItem::Armor(i + 1);
                } else if count1 > 0 {
                    self.selected = SelectedItem::Weapon(0);
                } else {
                    self.selected = SelectedItem::Armor(0);
                }
            }
            SelectedItem::Other(i) => {
                let count = items.other.len();
                if i + 1 < count {
                    self.selected = SelectedItem::Other(i + 1);
                } else {
                    self.selected = SelectedItem::Other(0);
                }
            }
        }
    }

    fn do_select_prev_row(&mut self, items: &InvItems) {
        match self.selected {
            SelectedItem::None => {
                if !items.weapons.is_empty() {
                    self.selected = SelectedItem::Weapon(items.weapons.len() - 1);
                } else if !items.armor.is_empty() {
                    self.selected = SelectedItem::Armor(items.armor.len() - 1);
                } else if !items.other.is_empty() {
                    self.selected = SelectedItem::Other(items.armor.len() - 1);
                }
            }
            SelectedItem::Weapon(i) => {
                let count1 = items.weapons.len();
                let count2 = items.armor.len();
                if i > 0 {
                    self.selected = SelectedItem::Weapon(i - 1);
                } else if count2 > 0 {
                    self.selected = SelectedItem::Armor(count2 - 1);
                } else {
                    self.selected = SelectedItem::Weapon(count1 - 1);
                }
            }
            SelectedItem::Armor(i) => {
                let count1 = items.weapons.len();
                let count2 = items.armor.len();
                if i > 0 {
                    self.selected = SelectedItem::Armor(i - 1);
                } else if count1 > 0 {
                    self.selected = SelectedItem::Weapon(count1 - 1);
                } else {
                    self.selected = SelectedItem::Armor(count2 - 1);
                }
            }
            SelectedItem::Other(i) => {
                let count = items.other.len();
                if i > 0 {
                    self.selected = SelectedItem::Other(i - 1);
                } else {
                    self.selected = SelectedItem::Other(count - 1);
                }
            }
        }
    }
}
