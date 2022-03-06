//! This code is for the arena binary which is used to simulate the results of combat.
use super::*;

pub enum Weapon {
    None,
    WeakSword,
    MightySword,
    EmpSword,
}

#[derive(Clone, Copy, Debug, Display, Eq, PartialEq)]
pub enum Opponent {
    Guard,
    Rhulad,
}

pub struct Stats {
    pub dps: f64,
    pub hits: f64,
    pub crits: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct ArenaResult {
    pub player_won: bool,
    pub turns: i32,
}

impl Game {
    pub fn new_arena(seed: u64) -> Game {
        let mut game = Game {
            stream: Vec::new(),
            file: None,
            state: State::Adventuring,
            scheduler: Scheduler::new(),

            rng: RefCell::new(SmallRng::seed_from_u64(seed)),

            level: Level::new(),
            players_move: false,

            messages: Vec::new(),
            interactions: Interactions::new(),
            pov: PoV::new(),
            old_pov: OldPoV::new(),
        };
        game.init_game(include_str!("maps/arena.txt"));
        game
    }

    pub fn setup_arena(&mut self, weapon: Weapon, opponent: Opponent) -> (Oid, Stats, Stats) {
        let oid = match weapon {
            Weapon::None => None,
            Weapon::WeakSword => Some(self.level.add(make::weak_sword(self), None)),
            Weapon::MightySword => Some(self.level.add(make::mighty_sword(), None)),
            Weapon::EmpSword => Some(self.level.add(make::emp_sword(), None)),
        };
        if let Some(oid) = oid {
            let player = self.level.get_mut(&self.player_loc(), INVENTORY_ID).unwrap().1;
            let inv = object::inventory_value_mut(player).unwrap();
            inv.push(oid);
        }

        let obj = match opponent {
            Opponent::Guard => make::guard(),
            Opponent::Rhulad => make::rhulad(),
        };
        let loc = Point::new(self.player_loc().x + 1, self.player_loc().y);
        let oid = self.add_object(&loc, obj);

        (oid, self.compute_stats(Oid(0), oid), self.compute_stats(oid, Oid(0)))
    }

    pub fn run_arena(&mut self, opponent: Oid) -> ArenaResult {
        let mut turns = 0;
        while self.active_arena(opponent) {
            if self.players_turn() {
                self.player_acted(Action::Move { dx: 1, dy: 0 });
                turns += 1;
            } else {
                self.advance_time(false);
            }
        }

        ArenaResult {
            player_won: self.state != State::LostGame,
            turns,
        }
    }

    fn active_arena(&self, oid: Oid) -> bool {
        if self.state == State::LostGame {
            return false; // player was killed
        }

        if let Some(obj) = self.level.try_obj(oid) {
            match object::behavior_value(obj) {
                Some(Behavior::Attacking(_, _)) => return true, // both still in combat
                Some(Behavior::Sleeping) => return true,        // opponent hasn't been hit yet
                _ => false,                                     // typically opponent started fleeing
            }
        } else {
            return false; // opponent was killed
        }
    }

    fn compute_stats(&self, attacker: Oid, defender: Oid) -> Stats {
        let loc = self.loc(attacker).unwrap();
        let delay = self.melee_delay(&loc);
        let damage = self.base_damage(attacker).0;
        let crits = self.crit_prob(attacker);
        let hits = self.hit_prob(attacker, defender);
        Stats {
            dps: (damage as f64) / ((delay.as_ms() as f64) / 1000.0),
            hits,
            crits,
        }
    }
}
