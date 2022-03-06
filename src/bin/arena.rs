#[macro_use]
extern crate log;
extern crate simplelog;

use one_thousand_deaths::{Game, Opponent, Stats, Weapon};
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

fn run(opponent: Opponent, round: i32) -> bool {
    let mut game = Game::new_arena(round as u64);
    let (oid, pstats, ostats) = game.setup_arena(Weapon::MightySword, opponent);
    if round == 0 {
        print_stats("player", pstats);
        println!("");
        print_stats(&format!("{opponent}"), ostats);
        print!("turns: ");
    }
    let result = game.run_arena(oid);
    print!("{} ", result.turns);
    result.player_won
}

fn main() {
    configure_logging(LevelFilter::Debug);

    // TODO:
    // write a report
    //    print some sort of histogram for turns
    //    need to run this for different combinations
    // probably want to print total run time
    // command line option for
    //    number of rounds
    //    who to test (maybe options are all or one combo)
    //    possibly verbose (like turns histogram)
    let num_rounds = 100;
    let mut player_wins = 0;
    for i in 0..num_rounds {
        if run(Opponent::Rhulad, i) {
            player_wins += 1;
        }
    }
    println!("");

    let p = 100.0 * (player_wins as f64) / (num_rounds as f64);
    println!("player won {player_wins} out of {num_rounds} times ({p:.1}%)");
}
