//! This code is for the arena binary which is used to simulate the results of combat.
use super::*;
use fnv::FnvHashMap;
use std::io::{Error, Write};

#[derive(Clone, Copy, Debug)]
enum Opponents {
    PlayerVsGuard,
    PlayerVsRhulad,
    PlayerVsBroken,
}

struct Stats {
    hps: i32,
    dps: f64,
    hits: f64,
    crits: f64,
}

#[derive(Clone, Copy, Debug)]
struct ArenaResult {
    player_won: bool,
    turns: i32,
}

// TODO: add a wizard command to run arena for player vs examined target
// clone player and target and pass them in as a new Opponents variant
//    cloning player wouldn't work quite right for inv items (opponent too for that matter)
//    unless we changed Object and Tag clone to do a deep clone
//    tho even that wouldn't be quite right because the oids won't make sense in the arena level
// may want to set HPs to full
// write results to a file (message should say where)
pub fn run_arena_matches(
    writer: &mut dyn Write,
    num_rounds: i32,
    seed: u64,
    filter: Option<String>,
) -> Result<(), Box<Error>> {
    // TODO:
    // try some custom setups, eg high dex vs high str build
    // review results, especially rounds
    run_arena_match(writer, num_rounds, seed, &filter, Opponents::PlayerVsGuard)?;
    run_arena_match(writer, num_rounds, seed, &filter, Opponents::PlayerVsRhulad)?;
    run_arena_match(writer, num_rounds, seed, &filter, Opponents::PlayerVsBroken)?;
    Ok(())
}

fn run_arena_match(
    writer: &mut dyn Write,
    num_rounds: i32,
    seed: u64,
    filter: &Option<String>,
    opponents: Opponents,
) -> Result<(), Box<Error>> {
    let name = format!("{opponents:?}");
    if filter.is_none() || name.contains(filter.as_ref().unwrap()) {
        let mut player_wins = 0;
        let mut results = Vec::new();

        writeln!(writer, "---- {name} {}", "-".repeat(50))?;
        for i in 0..num_rounds {
            let result = run_arena(writer, i, seed, opponents)?;
            if result.player_won {
                player_wins += 1;
            }
            results.push(result);
        }
        print_turns(writer, &results)?;

        let p = 100.0 * (player_wins as f64) / (num_rounds as f64);
        writeln!(
            writer,
            "\nplayer won {player_wins} out of {num_rounds} times ({p:.1}%)\n"
        )?;
    }
    Ok(())
}

fn run_arena(writer: &mut dyn Write, round: i32, seed: u64, opponents: Opponents) -> Result<ArenaResult, Box<Error>> {
    let mut game = Game::new_arena(seed + (round as u64));
    let (oid, pstats, ostats) = game.setup_arena(opponents);
    if round == 0 {
        let obj = game.level.obj(oid).0;
        print_stats(writer, "player", pstats, &format!("{obj}"), ostats)?;
        writeln!(writer, "")?;
    }
    Ok(game.run_arena(oid))
}

fn print_stats(
    writer: &mut dyn Write,
    pname: &str,
    pstats: Stats,
    oname: &str,
    ostats: Stats,
) -> Result<(), Box<Error>> {
    writeln!(writer, "{} hps   dps  hits  crits", " ".repeat(oname.len()))?;
    writeln!(
        writer,
        "{pname}{} {:>3}  {:>4.1}  {:>3}%  {:>4.1}%",
        " ".repeat(oname.len() - pname.len()),
        pstats.hps,
        pstats.dps,
        (100.0 * pstats.hits).round() as i32,
        (100.0 * pstats.crits).round() as i32,
    )?;
    writeln!(
        writer,
        "{oname} {:>3}  {:>4.1}  {:>3}%  {:>4.1}%",
        ostats.hps,
        ostats.dps,
        (100.0 * ostats.hits).round() as i32,
        (100.0 * ostats.crits).round() as i32,
    )?;
    Ok(())
}

fn print_turns(writer: &mut dyn Write, results: &Vec<ArenaResult>) -> Result<(), Box<Error>> {
    let limit = 30;

    let mut counts = FnvHashMap::default();
    for result in results {
        let count = counts.entry(result.turns).or_insert_with(|| 0);
        *count = *count + 1;
    }

    let mut turns: Vec<i32> = counts.keys().copied().collect();
    turns.sort_by(|a, b| a.partial_cmp(&b).unwrap());

    let max_count = *counts.values().max().unwrap();
    let scaling = if max_count > limit {
        (max_count as f64) / (limit as f64)
    } else {
        1.0
    };

    let max_stars = ((*counts.values().max().unwrap() as f64) / scaling).round() as usize;
    for turn in turns {
        let n = counts[&turn];
        let count = ((n as f64) / scaling).round() as usize;
        let stars = "*".repeat(count);
        let padding = " ".repeat(max_stars - count + 2);
        writeln!(writer, "{turn:<2}: {stars}{padding}{n}")?;
    }
    Ok(())
}

impl Game {
    fn new_arena(seed: u64) -> Game {
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

    fn setup_arena(&mut self, opponents: Opponents) -> (Oid, Stats, Stats) {
        let (oid1, oid2) = match opponents {
            Opponents::PlayerVsGuard => {
                let loc = Point::new(self.player_loc().x + 1, self.player_loc().y);
                let oid = self.add_object(&loc, make::guard());
                (Oid(0), oid)
            }
            Opponents::PlayerVsRhulad => {
                let oid = self.level.add(make::mighty_sword(), None);
                let player = self.level.get_mut(&self.player_loc(), INVENTORY_ID).unwrap().1;
                let inv = player.inventory_value_mut().unwrap();
                inv.push(oid);

                let loc = Point::new(self.player_loc().x + 1, self.player_loc().y);
                let oid = self.add_object(&loc, make::rhulad());
                (Oid(0), oid)
            }
            Opponents::PlayerVsBroken => {
                let oid = self.level.add(make::emp_sword(), None);
                let player = self.level.get_mut(&self.player_loc(), INVENTORY_ID).unwrap().1;
                let inv = player.inventory_value_mut().unwrap();
                inv.push(oid);

                let loc = Point::new(self.player_loc().x + 1, self.player_loc().y);
                let oid = self.add_object(&loc, make::broken(0));
                (Oid(0), oid)
            }
        };

        (oid2, self.compute_stats(oid1, oid2), self.compute_stats(oid2, oid1))
    }

    fn run_arena(&mut self, opponent: Oid) -> ArenaResult {
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
            match obj.behavior_value() {
                Some(Behavior::Attacking(_, _)) => return true, // both still in combat
                Some(Behavior::Sleeping) => return true,        // opponent hasn't been hit yet
                Some(Behavior::Wandering(_)) => return true,    // opponent hasn't been hit yet
                _ => false,                                     // typically opponent started fleeing
            }
        } else {
            return false; // opponent was killed
        }
    }

    fn compute_stats(&self, attacker: Oid, defender: Oid) -> Stats {
        let obj = self.level.obj(attacker).0;
        let hps = obj.durability_value().unwrap().current;

        let loc = self.loc(attacker).unwrap();
        let delay = self.melee_delay(&loc);
        let damage = self.base_damage(attacker).0;
        let crits = self.crit_prob(attacker);
        let hits = self.hit_prob(attacker, defender);
        Stats {
            hps,
            dps: (damage as f64) / ((delay.as_ms() as f64) / 1000.0),
            hits,
            crits,
        }
    }
}
