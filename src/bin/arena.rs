#[macro_use]
extern crate log;
extern crate simplelog;

use fnv::FnvHashMap;
use one_thousand_deaths::{ArenaResult, Game, Opponent, Stats, Weapon};
use simplelog::{CombinedLogger, ConfigBuilder, LevelFilter, WriteLogger};
use std::fs::File;

fn configure_logging(level: LevelFilter) {
    let logging = ConfigBuilder::new()
        .set_target_level(LevelFilter::Off)
        .set_thread_level(LevelFilter::Off)
        .set_location_level(LevelFilter::Off)
        .build();
    let file = File::create("1k-deaths.log").unwrap();
    CombinedLogger::init(vec![WriteLogger::new(level, logging, file)]).unwrap();

    let local = chrono::Local::now();
    info!(
        "started up on {} with version {} ----------------------------------------------",
        local.to_rfc2822(),
        env!("CARGO_PKG_VERSION")
    );
}

fn print_stats(name: &str, stats: Stats) {
    println!("{name} dps:   {:.1}", stats.dps);
    println!("{name} hits:  {}%", (100.0 * stats.hits).round() as i32);
    println!("{name} crits: {}%", (100.0 * stats.crits).round() as i32);
}

fn print_turns(results: &Vec<ArenaResult>) {
    let limit = 30;

    let max_turns = results
        .iter()
        .max_by(|x, y| x.turns.cmp(&y.turns))
        .map_or_else(|| 0, |z| z.turns);
    let scaling = if max_turns > limit {
        (max_turns as f64) / (limit as f64)
    } else {
        1.0
    };

    let mut counts = FnvHashMap::default();
    for result in results {
        let count = counts.entry(result.turns).or_insert_with(|| 0);
        *count = *count + 1;
    }

    let mut turns: Vec<i32> = counts.keys().copied().collect();
    turns.sort_by(|a, b| a.partial_cmp(&b).unwrap());

    let max_count = ((*counts.values().max().unwrap() as f64) / scaling).round() as usize;
    for turn in turns {
        let n = counts[&turn];
        let count = ((n as f64) / scaling).round() as usize;
        let stars = "*".repeat(count);
        let padding = " ".repeat(max_count - count + 2);
        println!("{turn:>2}: {stars}{padding}{n}");
    }
}

fn run(opponent: Opponent, round: i32) -> ArenaResult {
    let mut game = Game::new_arena(round as u64);
    let (oid, pstats, ostats) = game.setup_arena(Weapon::MightySword, opponent);
    if round == 0 {
        print_stats("player", pstats);
        println!("");
        print_stats(&format!("{opponent}"), ostats);
    }
    game.run_arena(oid)
}

fn main() {
    configure_logging(LevelFilter::Debug);

    // TODO:
    // probably want to print total run time
    // try some custom setups, eg high dex vs high str build
    // command line option for
    //    number of rounds
    //    who to test (maybe options are all or one combo)
    //    possibly verbose (like turns histogram)
    // think about moving pretty much all this into backend
    //    want a way to simulate combat in wizard mode
    //    probably have to pass a Writer into the arena code, also num_rounds
    //    make the melee functions private? or can we use one of the fancy visibility modifiers?
    let num_rounds = 100;
    let mut player_wins = 0;
    let mut results = Vec::new();
    for i in 0..num_rounds {
        let result = run(Opponent::Rhulad, i);
        if result.player_won {
            player_wins += 1;
        }
        results.push(result);
    }
    print_turns(&results);

    let p = 100.0 * (player_wins as f64) / (num_rounds as f64);
    println!("player won {player_wins} out of {num_rounds} times ({p:.1}%)");
}
