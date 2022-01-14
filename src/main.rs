extern crate derive_more;
#[macro_use]
extern crate slog; // for info!, debug!, etc

mod backend;
mod terminal;

use slog::Logger;
use sloggers::Build; // for build trait
use std::path::Path;
use std::process;
use std::str::FromStr; // for from_str trait

use backend::Game;

// Note that logging is async which should be OK as long as we continue to unwind
// on panics (though note that aborting would shrink binary size).
fn make_logger() -> Logger {
    // let severity = match sloggers::types::Severity::from_str(&options.log_level) {
    let severity = match sloggers::types::Severity::from_str("debug") {
        Ok(l) => l,
        Err(_) => {
            eprintln!("--log-level should be critical, error, warning, info, debug, or trace");
            process::exit(1);
        }
    };

    // "event" => event			uses slog::Value trait (so that output is structured)
    // "event" => %event		uses Display trait
    // "event" => ?event		uses Debug trait
    let path = Path::new("1k-deaths.log");
    let mut builder = sloggers::file::FileLoggerBuilder::new(path);
    builder.format(sloggers::types::Format::Compact);
    builder.overflow_strategy(sloggers::types::OverflowStrategy::Block); // TODO: logging is async which is kinda lame
    builder.source_location(sloggers::types::SourceLocation::None);
    builder.level(severity);
    builder.truncate();
    builder.build().unwrap()
}

fn main() {
    let root_logger = make_logger();
    let local = chrono::Local::now();
    info!(root_logger, "started up"; "on" => local.to_rfc2822(), "version" => env!("CARGO_PKG_VERSION"));
    //	info!(root_logger, "started up"; "seed" => options.seed, "on" => local.to_rfc2822());

    let mut game = Game::new(root_logger.new(o!()));
    game.start();
    let mut terminal = terminal::Terminal::new(root_logger, game);
    terminal.run();
}
