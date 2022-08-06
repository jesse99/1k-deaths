mod make;
mod message;
mod object;
mod tag;
mod terrain;
mod time;

pub use make::*;
pub use message::*;
pub use object::*;
pub use tag::*;
pub use terrain::*;
pub use time::*;

use rand::prelude::*;
use rand::rngs::SmallRng;
// use rand::RngCore;
use rand_distr::StandardNormal;
use std::cell::RefCell;

/// Returns a number with the standard normal distribution centered on x where the
/// values are all within +/- the given percentage.
pub fn rand_normal64(x: i64, percent: i32, rng: &RefCell<SmallRng>) -> i64 {
    assert!(percent > 0);
    assert!(percent <= 100);

    // Could use a generic for this but the type bounds get pretty gnarly.
    let rng = &mut *rng.borrow_mut();
    let scaling: f64 = rng.sample(StandardNormal); // ~95% are in -2..2
    let scaling = if scaling >= -2.0 && scaling <= 2.0 {
        scaling / 2.0 // all are in -1..1
    } else {
        0.0 // the few outliers are mapped to the mode
    };
    let scaling = scaling * (percent as f64) / 100.0; // all are in +/- percent
    let delta = (x as f64) * scaling; // all are in +/- percent of x
    x + (delta as i64)
}

pub fn rand_normal32(x: i32, percent: i32, rng: &RefCell<SmallRng>) -> i32 {
    rand_normal64(x as i64, percent, rng) as i32
}
