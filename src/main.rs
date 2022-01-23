extern crate derive_more;
#[macro_use]
extern crate log;
extern crate simplelog;
#[macro_use]
extern crate serde_derive;

mod backend;
mod terminal;

use backend::Game;
use simplelog::{CombinedLogger, ConfigBuilder, LevelFilter, WriteLogger};
use std::fs::File;

fn main() {
    let config = ConfigBuilder::new()
        .set_target_level(LevelFilter::Off)
        .set_thread_level(LevelFilter::Off)
        .set_location_level(LevelFilter::Off)
        .build();
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Debug,
        config,
        File::create("1k-deaths.log").unwrap(),
    )])
    .unwrap();
    let local = chrono::Local::now();
    info!(
        "started up on {} with version {}",
        local.to_rfc2822(),
        env!("CARGO_PKG_VERSION")
    );

    let (mut game, mut events) = Game::new();
    if events.is_empty() {
        game.new_game(&mut events);
    }
    game.post(events);
    let mut terminal = terminal::Terminal::new(game);
    terminal.run();
}
