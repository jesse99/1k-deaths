#[macro_use]
extern crate log;
extern crate simplelog;

use ipmpsc::{Receiver, SharedRingBuffer};
use onek_types::*;
use simplelog::{ConfigBuilder, LevelFilter, WriteLogger};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::{fs::File, str::FromStr};

struct App {
    duration: Duration,
    services: HashMap<u32, PokeAction>,
    last_poke: HashMap<u32, Instant>,
}

// TODO: think we need to use signal_child crate, probably better to use sysinfo
// TODO: how do we handle restart? should we? maybe enum would have a script to run?
fn check_services(app: &mut App) {
    debug!("checking poke times");

    let now = Instant::now();
    for (pid, &last) in app.last_poke.iter() {
        let elapsed = now - last;
        if elapsed >= app.duration {
            match app.services.get(&pid) {
                Some(action) => match action {
                    PokeAction::Ignore => (),
                    PokeAction::Restart => todo!(),
                    PokeAction::Shutdown => {
                        error!(
                            "process {pid} hasn't had a poke in {}s: killing all services",
                            elapsed.as_secs()
                        );
                        // force kill hung service
                        // clean kill others, wait a bit and force kill if not killed?
                        // exit App?
                    }
                },
                None => {
                    info!("removing stale pid {pid}");
                    app.services.remove(&pid);
                }
            }
        }
    }
}

fn handle_mesg(app: &mut App, mesg: AppMessages) {
    debug!("received {mesg:?}");
    match mesg {
        AppMessages::Duration(secs) => app.duration = Duration::new(secs, 0),
        AppMessages::Poke(pid) => {
            app.last_poke.insert(pid, Instant::now());
        }
        AppMessages::Register(pid, action) => {
            app.services.insert(pid, action);
        }
    };
}

fn init_logging(config: &Config) {
    // See https://docs.rs/simplelog/0.12.1/simplelog/struct.ConfigBuilder.html
    let location = LevelFilter::from_str(&config.str_value("log_location", "off")).expect("bad log_location");
    let log_level = LevelFilter::from_str(&config.str_value("log_level", "info")).expect("bad log_level");
    let log_path = config.str_value("log_path", "app.log");
    let config = ConfigBuilder::new()
        .set_location_level(location) // file names and line numbers
        .set_target_level(LevelFilter::Off) // don't log exe name
        .set_thread_level(LevelFilter::Off) // don't log thread IDs
        .build();
    // Unwrapping File::create is a little lame but it actually returns a decent error message.
    let _ = WriteLogger::init(log_level, config, File::create(&log_path).unwrap()).unwrap();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load("onek-app");
    init_logging(&config);

    let local = chrono::Local::now();
    info!(
        "started up on {} with version {} ----------------------------",
        local.to_rfc2822(),
        env!("CARGO_PKG_VERSION")
    );

    let err = config.error();
    if err.is_some() {
        error!("error loading config: {}", err.as_ref().unwrap());
    }

    let map_file = "/tmp/app-sink";
    let rx = Receiver::new(SharedRingBuffer::create(map_file, 32 * 1024)?);

    let mut app = App {
        duration: Duration::new(10, 0),
        services: HashMap::new(),
        last_poke: HashMap::new(),
    };
    let mut last_check = Instant::now();
    loop {
        match rx.recv_timeout(app.duration) {
            Ok(result) => match result {
                Some(mesg) => handle_mesg(&mut app, mesg),
                None => (), // will only land here if all services are hung
            },
            Err(err) => {
                error!("rx error: {err}");
                return Result::Err(Box::new(err));
            }
        }

        let now = Instant::now();
        if now >= last_check + app.duration {
            check_services(&mut app);
            last_check = now;
        }
    }
}
