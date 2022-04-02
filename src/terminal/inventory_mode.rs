use super::context_menu::{ContextMenu, ContextResult};
use super::help::{format_help, validate_help};
use super::inventory_view::InventoryView;
use super::mode::{InputAction, Mode, RenderContext};
use super::text_mode::TextMode;
use fnv::FnvHashMap;
use one_thousand_deaths::{Action, Game, InvItem, ItemKind, Point, Size};
use std::fmt::{self, Formatter};
use termion::event::Key;

type KeyHandler = fn(&mut InventoryMode, &mut Game) -> InputAction;
type CommandTable = FnvHashMap<Key, Box<KeyHandler>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ContextItem {
    Drop,
    Remove,
    WieldBothHands,
    WieldMainHand,
    WieldOffHand,
}

pub struct InventoryMode {
    commands: CommandTable,
    view: InventoryView,
    selected: Option<usize>,
    menu: Option<ContextMenu<ContextItem>>,
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
        commands.insert(Key::Char('\n'), Box::new(|s, game| s.do_create_menu(game)));
        commands.insert(Key::Esc, Box::new(|s, game| s.do_pop(game)));

        let origin = Point::new(1, 1);
        let view = InventoryView { origin, size };
        let selected = None;
        let mut mode = InventoryMode {
            commands,
            view,
            selected,
            menu: None,
        };
        mode.do_select(game, 0, 1);
        Box::new(mode)
    }
}

impl Mode for InventoryMode {
    fn render(&self, context: &mut RenderContext) -> bool {
        let desc = self.describe_item(context.game);
        self.view.render(self.selected, context.stdout, context.game, desc);
        if let Some(menu) = self.menu.as_ref() {
            menu.render(context.stdout);
        }
        true
    }

    fn input_timeout_ms(&self) -> Option<i32> {
        None
    }

    fn handle_input(&mut self, game: &mut Game, key: Key) -> InputAction {
        if let Some(menu) = self.menu.as_mut() {
            match menu.handle_input(key) {
                ContextResult::Selected(ContextItem::Drop) => {
                    self.drop_item(game);
                    self.menu = None;
                }
                ContextResult::Selected(ContextItem::Remove) => {
                    self.remove_item(game);
                    self.menu = None;
                }
                ContextResult::Selected(ContextItem::WieldBothHands) => {
                    self.wield_main(game);
                    self.menu = None;
                }
                ContextResult::Selected(ContextItem::WieldMainHand) => {
                    self.wield_main(game);
                    self.menu = None;
                }
                ContextResult::Selected(ContextItem::WieldOffHand) => {
                    self.wield_off(game);
                    self.menu = None;
                }
                ContextResult::Pop => self.menu = None,
                ContextResult::Updated => (),
                ContextResult::NotHandled => (),
            }
            InputAction::UpdatedGame
        } else {
            match self.commands.get(&key).cloned() {
                Some(handler) => handler(self, game),
                None => InputAction::NotHandled,
            }
        }
    }
}

impl InventoryMode {
    fn describe_item(&self, game: &mut Game) -> Vec<String> {
        if let Some(index) = self.selected {
            let inv = game.inventory();
            game.describe_item(inv[index].oid)
        } else {
            Vec::new()
        }
    }

    fn drop_item(&self, game: &mut Game) {
        let inv = game.inventory();
        let index = self.selected.unwrap();
        game.player_acted(Action::Drop(inv[index].oid));
    }

    fn remove_item(&self, game: &mut Game) {
        let inv = game.inventory();
        let index = self.selected.unwrap();
        game.player_acted(Action::Remove(inv[index].oid));
    }

    fn wield_main(&self, game: &mut Game) {
        let inv = game.inventory();
        let index = self.selected.unwrap();
        game.player_acted(Action::WieldMainHand(inv[index].oid));
    }

    fn wield_off(&self, game: &mut Game) {
        let inv = game.inventory();
        let index = self.selected.unwrap();
        game.player_acted(Action::WieldOffHand(inv[index].oid));
    }

    fn do_create_menu(&mut self, game: &mut Game) -> InputAction {
        if self.selected.is_none() {
            return InputAction::NotHandled;
        }

        let inv = game.inventory();
        let index = self.selected.unwrap();
        let suffix = inv[index].name;

        let mut items = vec![ContextItem::Drop];
        if inv[index].equipped.is_some() {
            items.push(ContextItem::Remove);
        }
        match inv[index].kind {
            ItemKind::TwoHandWeapon => items.push(ContextItem::WieldBothHands),
            ItemKind::OneHandWeapon => {
                items.push(ContextItem::WieldMainHand);
                items.push(ContextItem::WieldOffHand);
            }
            ItemKind::Armor => (),
            ItemKind::Other => (),
        };

        assert!(self.menu.is_none(), "if there's a menu it should have handled return");
        self.menu = Some(ContextMenu {
            parent_origin: self.view.origin,
            parent_size: self.view.size,
            items,
            suffix: suffix.to_string(),
            selected: 0,
        });
        InputAction::UpdatedGame
    }

    fn do_help(&mut self, _game: &mut Game) -> InputAction {
        let help = r#"Used to manage the items you've picked up.

Selection can be moved using the numeric keypad or arrow keys:
[[7]] [[8]] [[9]]                  [[up-arrow]]
[[4]]   [[6]]           [[left-arrow]]   [[right-arrow]]
[[1]] [[2]] [[3]]                 [[down-arrow]]

[[return]] operates on the selection.
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
        let weapons = vec![ItemKind::OneHandWeapon, ItemKind::TwoHandWeapon];
        let armor = vec![ItemKind::Armor];
        let other = vec![ItemKind::Other];
        if let Some(start) = self.selected {
            let kind = inv[start].kind;
            if dx == 1 {
                // right
                match kind {
                    ItemKind::Other => {
                        let _ = self.do_select_first_item(&inv, &weapons) || self.do_select_first_item(&inv, &armor);
                    }
                    _ => {
                        self.do_select_first_item(&inv, &other);
                    }
                }
            } else if dx == -1 {
                // left
                match kind {
                    ItemKind::Other => {
                        let _ = self.do_select_last_item(&inv, &weapons) || self.do_select_last_item(&inv, &armor);
                    }
                    _ => {
                        self.do_select_last_item(&inv, &other);
                    }
                }
            }
            if dy == 1 {
                // down
                match kind {
                    ItemKind::OneHandWeapon | ItemKind::TwoHandWeapon => {
                        let _ = self.do_select_next_item(&inv, start)
                            || self.do_select_first_item(&inv, &armor)
                            || self.do_select_first_item(&inv, &other)
                            || self.do_select_first_item(&inv, &weapons);
                    }
                    ItemKind::Armor => {
                        let _ = self.do_select_next_item(&inv, start)
                            || self.do_select_first_item(&inv, &other)
                            || self.do_select_first_item(&inv, &weapons)
                            || self.do_select_first_item(&inv, &armor);
                    }
                    ItemKind::Other => {
                        let _ = self.do_select_next_item(&inv, start)
                            || self.do_select_first_item(&inv, &weapons)
                            || self.do_select_first_item(&inv, &armor)
                            || self.do_select_first_item(&inv, &other);
                    }
                }
            } else if dy == -1 {
                // up
                match kind {
                    ItemKind::OneHandWeapon | ItemKind::TwoHandWeapon => {
                        let _ = self.do_select_prev_item(&inv, start)
                            || self.do_select_last_item(&inv, &other)
                            || self.do_select_last_item(&inv, &armor)
                            || self.do_select_last_item(&inv, &weapons);
                    }
                    ItemKind::Armor => {
                        let _ = self.do_select_prev_item(&inv, start)
                            || self.do_select_last_item(&inv, &weapons)
                            || self.do_select_last_item(&inv, &other)
                            || self.do_select_last_item(&inv, &armor);
                    }
                    ItemKind::Other => {
                        let _ = self.do_select_prev_item(&inv, start)
                            || self.do_select_last_item(&inv, &armor)
                            || self.do_select_last_item(&inv, &weapons)
                            || self.do_select_last_item(&inv, &other);
                    }
                }
            }
        } else {
            if dy == -1 {
                let _ = self.do_select_last_item(&inv, &other)
                    || self.do_select_last_item(&inv, &armor)
                    || self.do_select_last_item(&inv, &weapons);
            } else {
                // If nothing is selected then left or right doesn't mean much so we just
                // handle it like down.
                let _ = self.do_select_first_item(&inv, &weapons)
                    || self.do_select_first_item(&inv, &armor)
                    || self.do_select_first_item(&inv, &other);
            }
        }
        InputAction::UpdatedGame
    }

    fn do_select_first_item(&mut self, items: &Vec<InvItem>, kinds: &Vec<ItemKind>) -> bool {
        for (i, candidate) in items.iter().enumerate() {
            if kinds.contains(&candidate.kind) {
                self.selected = Some(i);
                return true;
            }
        }
        false
    }

    fn do_select_last_item(&mut self, items: &Vec<InvItem>, kinds: &Vec<ItemKind>) -> bool {
        let mut found = false;
        for (i, candidate) in items.iter().enumerate() {
            if kinds.contains(&candidate.kind) {
                self.selected = Some(i);
                found = true;
            }
        }
        found
    }

    fn do_select_next_item(&mut self, items: &Vec<InvItem>, start: usize) -> bool {
        let kinds = match items[start].kind {
            ItemKind::OneHandWeapon | ItemKind::TwoHandWeapon => vec![ItemKind::OneHandWeapon, ItemKind::TwoHandWeapon],
            _ => vec![items[start].kind],
        };
        for (i, candidate) in items.iter().enumerate().skip(start + 1) {
            if kinds.contains(&candidate.kind) {
                self.selected = Some(i);
                return true;
            }
        }
        false
    }

    fn do_select_prev_item(&mut self, items: &Vec<InvItem>, start: usize) -> bool {
        let kinds = match items[start].kind {
            ItemKind::OneHandWeapon | ItemKind::TwoHandWeapon => vec![ItemKind::OneHandWeapon, ItemKind::TwoHandWeapon],
            _ => vec![items[start].kind],
        };
        for (i, candidate) in items.iter().enumerate().take(start).rev() {
            if kinds.contains(&candidate.kind) {
                self.selected = Some(i);
                return true;
            }
        }
        false
    }
}

impl fmt::Display for ContextItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            ContextItem::Drop => "Drop",
            ContextItem::Remove => "Remove",
            ContextItem::WieldBothHands => "Wield (both hands)",
            ContextItem::WieldMainHand => "Wield (main hand)",
            ContextItem::WieldOffHand => "Wield (off hand)",
        };
        write!(f, "{s}")
    }
}
