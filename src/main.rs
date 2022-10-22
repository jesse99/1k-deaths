#[macro_use]
extern crate log;
extern crate simplelog;

use simplelog::*;

use std::fs::File;

// TODO:
// add options to control the logger
//  set_max_level
//  location
//  allow modules
//  ignore modules
fn main() {
    let config = ConfigBuilder::new()
        .set_thread_level(LevelFilter::Off) // don't log thread IDs
        .set_target_level(LevelFilter::Off) // don't log exe name
        .set_location_level(LevelFilter::Off) // include file and line?
        .build();
    let _ = WriteLogger::init(LevelFilter::Debug, config, File::create("1k-deaths.log").unwrap()).unwrap();

    error!("error level");
    info!("info level");
    debug!("debug level");
}
