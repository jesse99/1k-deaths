use super::*;
use fnv::FnvHashMap;
// use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;

pub static PLAYER_ID: Oid = Oid::without_tag(0);
pub static DEFAULT_CELL_ID: Oid = Oid::without_tag(1);
pub static GAME_ID: Oid = Oid::without_tag(2);
pub static LAST_ID: u32 = 2;

/// Wrapper around a [`Store`] for cells within a level. These need some special casing
/// because the mapping from a location to an Oid has to be consistent and because level
/// sizes are not static (levels have a default cell which is typically rock and can be
/// dug into to create a new normal cell).
#[derive(Serialize, Deserialize)]
pub struct Level {
    pub store: Store<Oid>,
    pub pov: PoV,
    pub old_pov: OldPoV,
    cell_ids: FnvHashMap<Point, Oid>,
    current_id: u32,
}

impl Level {
    pub fn new() -> Level {
        let mut level = Level {
            store: Store::new(),
            pov: PoV::new(),
            old_pov: OldPoV::new(),
            cell_ids: FnvHashMap::default(),
            current_id: LAST_ID,
        };
        level.store.create(DEFAULT_CELL_ID, Terrain::Wall);
        level
    }

    pub fn create_player(&mut self, loc: Point) {
        let oid = self.get_or_create_cell(loc);
        self.store.create(PLAYER_ID, Character::Player);
        self.store.create(PLAYER_ID, loc);
        // also has an empty list of InvItem

        self.store.append(oid, PLAYER_ID);
    }

    pub fn find_char(&self, loc: Point) -> Option<Character> {
        if let Some(cell_oid) = self.find_cell(loc) {
            if let Some(obj_oid) = self.store.get_last::<Oid>(cell_oid) {
                self.store.find::<Character>(obj_oid)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn create_terrain(&mut self, loc: Point, terrain: Terrain) {
        let oid = self.get_or_create_cell(loc);
        self.store.create(oid, terrain);
        self.store.create(oid, loc);
    }

    /// If the loc was never created then this will extend the level by creating a new
    /// cell.
    pub fn set_terrain(&mut self, loc: Point, terrain: Terrain) {
        if let Some(oid) = self.find_cell(loc) {
            let replaced = self.store.replace(oid, terrain);
            assert!(replaced);
        } else {
            let oid = self.get_or_create_cell(loc);
            self.store.create(oid, terrain);
            self.store.create(oid, loc);
        }
    }

    // TODO: If find_cell turns out to be a bottle-neck then could add a get_cell
    // function with methods like get_terrain, get_portables, and get_char.
    pub fn get_terrain(&self, loc: Point) -> Terrain {
        if let Some(cell_oid) = self.find_cell(loc) {
            let terrain = self.store.find::<Terrain>(cell_oid);
            terrain.expect(&format!("expected a terrain for {cell_oid}"))
        } else {
            let terrain = self.store.find::<Terrain>(DEFAULT_CELL_ID);
            terrain.expect(&format!("expected a terrain for the default cell"))
        }
    }

    pub fn append_portable(&mut self, loc: Point, tag: &str, portable: Portable) {
        if let Some(oid) = self.find_cell(loc) {
            let obj_oid = self.new_oid(tag);
            self.store.create(obj_oid, portable);
            self.store.append(oid, obj_oid);
        } else {
            panic!("Can't add a portable to the default cell");
        }
    }

    pub fn get_portables(&self, loc: Point) -> Vec<Portable> {
        let mut portables = Vec::new();
        if let Some(cell_oid) = self.find_cell(loc) {
            let oids = self.store.get_all::<Oid>(cell_oid);
            for obj_oid in &oids {
                if let Some(p) = self.store.find::<Portable>(*obj_oid) {
                    portables.push(p);
                }
            }
        }
        portables
    }

    pub fn num_objects(&self, loc: Point) -> usize {
        self.find_cell(loc).map(|oid| self.store.len::<Oid>(oid)).unwrap_or(0)
    }

    pub fn append_message(&mut self, message: Message) {
        const MAX_MESSAGES: usize = 1000;
        const EXTRA_MESSAGES: usize = 100; // trim messages once len is more than MAX + EXTRA

        self.store.append(GAME_ID, message);
        let len = self.store.len::<Message>(GAME_ID);
        if len >= MAX_MESSAGES + EXTRA_MESSAGES {
            self.store.remove_range::<Message>(GAME_ID, 0..(len - MAX_MESSAGES));
        }
    }

    /// Note that this has no checks for whether this is a legal move (e.g. moving the
    /// player into a stone wall is normally not allowed).
    pub fn move_char(&mut self, oid: Oid, new_loc: Point) {
        if let Some(old_loc) = self.store.find::<Point>(oid) {
            let cell_id = self.cell_ids.get(&old_loc).unwrap();
            self.store.remove_value(*cell_id, oid);
        }

        self.store.replace(oid, new_loc);

        let cell_id = self.get_or_create_cell(new_loc);
        self.store.append(cell_id, oid);
    }

    // TODO: move_obj would be very similar to move_char except the we need to insert the obj_oid before characters

    pub fn expect_location(&self, oid: Oid) -> Point {
        let loc = self.store.find::<Point>(oid);
        loc.expect(&format!("expected a loc for {oid}"))
    }

    // TODO: will need to extend this for effects, e.g. clouds could have a chance of blocking los
    // TODO: maybe characters or items could also block los
    pub fn blocks_los(&self, loc: Point) -> bool {
        // non-existent cells need to block los
        match self.get_terrain(loc) {
            Terrain::ClosedDoor => true,
            Terrain::DeepWater => false,
            Terrain::Dirt => false,
            Terrain::OpenDoor => false,
            Terrain::Rubble => false,
            Terrain::ShallowWater => false,
            Terrain::Tree => true,
            Terrain::Vitr => false,
            Terrain::Wall => true,
        }
    }

    pub fn cell_iter(&self) -> impl Iterator<Item = (&Point, &Oid)> {
        self.cell_ids.iter()
    }

    // Extent cells always have a terrain and a list of oids associated with that location.
    pub fn find_cell(&self, loc: Point) -> Option<Oid> {
        self.cell_ids.get(&loc).copied()
    }

    fn get_or_create_cell(&mut self, loc: Point) -> Oid {
        // Can't use the entry API because we need to create a new Oid on insert which
        // would require two mutable refs to seld. Should be OK though because the normal
        // case is to have a cell.
        if let Some(oid) = self.find_cell(loc) {
            oid
        } else {
            let oid = self.new_oid(&format!("{loc}"));
            self.cell_ids.insert(loc, oid);
            oid
        }
    }

    fn new_oid(&mut self, tag: &str) -> Oid {
        self.current_id += 1;
        Oid::new(tag, self.current_id)
    }
}

impl Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.dump_pov(f)?;
        self.dump_non_pov(f)?;
        self.dump_game(f)
    }
}

impl Level {
    fn dump_pov(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut details = Vec::new();

        let center = self.expect_location(PLAYER_ID);
        write!(f, "==== Visible ===================================\n")?;
        for y in center.y - RADIUS..center.y + RADIUS {
            for x in center.x - RADIUS..center.x + RADIUS {
                let loc = Point::new(x, y);
                if self.pov.visible(self, loc) {
                    if self.num_objects(loc) > 0 {
                        let cp = (48 + details.len()) as u8;
                        write!(f, "{}", (cp as char))?;
                        details.push(loc);
                    } else {
                        self.dump_terrain(loc, f)?;
                    }
                } else {
                    write!(f, " ")?;
                }
            }
            write!(f, "\n")?;
        }
        write!(f, "\n")?;

        for (i, loc) in details.iter().enumerate() {
            let cp = (48 + i) as u8;
            let label = format!("{} at {loc}", (cp as char));
            self.dump_details(*loc, &label, f)?;
        }
        write!(f, "\n")?;
        Result::Ok(())
    }

    fn dump_non_pov(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "==== Not visible ===================================\n")?;
        for (loc, _) in self.cell_iter() {
            if !self.pov.visible(self, *loc) {
                if self.num_objects(*loc) > 0 {
                    let label = format!("{loc}");
                    self.dump_details(*loc, &label, f)?;
                    write!(f, "\n")?;
                }
            }
        }
        write!(f, "\n")?;
        Result::Ok(())
    }

    fn dump_game(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "==== Game ===================================\n")?;
        let messages = self.store.get_all::<Message>(GAME_ID);
        for mesg in messages {
            write!(f, "{mesg}\n")?;
        }
        Result::Ok(())
    }

    fn dump_details(&self, loc: Point, label: &str, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{label}\n")?;

        if let Some(cell_oid) = self.find_cell(loc) {
            write!(f, "   ")?;
            self.dump_terrain(loc, f)?;
            write!(f, "\n")?;

            let oids = self.store.get_all::<Oid>(cell_oid);
            for oid in oids.iter() {
                if let Some(p) = self.store.find::<Portable>(*oid) {
                    write!(f, "   {p}\n")?;
                }
                if let Some(c) = self.store.find::<Character>(*oid) {
                    write!(f, "   {c}\n")?; // TODO: can dump a lot more here
                }
            }
        } else {
            write!(f, "   couldn't find an oid for {loc}\n")?;
        }
        Result::Ok(())
    }

    fn dump_terrain(&self, loc: Point, f: &mut fmt::Formatter) -> fmt::Result {
        let terrain = self.get_terrain(loc);
        match terrain {
            Terrain::ClosedDoor => write!(f, "+")?,
            Terrain::DeepWater => write!(f, "W")?,
            Terrain::Dirt => write!(f, ".")?,
            Terrain::OpenDoor => write!(f, "-")?,
            Terrain::Rubble => write!(f, "…")?,
            Terrain::ShallowWater => write!(f, "w")?,
            Terrain::Tree => write!(f, "T")?,
            Terrain::Vitr => write!(f, "V")?,
            Terrain::Wall => write!(f, "#")?,
        }
        Result::Ok(())
    }
}
