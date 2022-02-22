// Note that the probabilities listed below were computed with scripts/sound_prob.py.
use super::*;
use rand::rngs::SmallRng;
use rand::Rng;
use std::cell::RefCell;
use std::ops::Mul;

/// Volume represents the percent chance that an NPC will wake up if it is on top of the
/// noise source. The probability goes down according to 0.75^distance so there is always
/// a chance that an NPC will wake up (inside the cutoff point anyway).
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Sound {
    volume: i32,
}

// /// This corresponds to something like resting. Percentages for this work out to:
// /// 1.5 1.1 0.8 0.6 0.5 0.4 0.3 0.2 0.2.
// pub const SUPER_QUIET: Sound = Sound { volume: 1 };

/// This corresponds to something like drinking a potion. Percentages for this work out to:
/// 15.0 11.2 8.4 6.3 4.7 3.6 2.7 2.0 1.5.
pub const VERY_QUIET: Sound = Sound { volume: 20 };

/// This corresponds to something like the base movement noise. Percentages for this work
/// out to: 100.0 84.4 63.3 47.5 35.6 26.7 20.0 15.0 11.3.
pub const QUIET: Sound = Sound { volume: 150 };

// /// This corresponds to something like a creaky door. Percentages for this work out to:
// /// 100.0 100.0 100.0 94.9 71.2 53.4 40.0 30.0 22.5.
// pub const NOISY: Sound = Sound { volume: 200 };

/// This corresponds to something like a yell. Percentages for this work out to:
/// 100.0 100.0 100.0 100.0 100.0 89.0 66.7 50.1 37.5.
pub const LOUD: Sound = Sound { volume: 500 };

// /// This corresponds to something like a cusser exploding. Percentages for this work out to:
// /// 100.0 100.0 100.0 100.0 100.0 100.0 100.0 100.0 75.1.
// pub const VERY_LOUD: Sound = Sound { volume: 1000 };

impl Sound {
    pub fn was_heard(&self, rng: &RefCell<SmallRng>, distance2: i32) -> (bool, f64) {
        let distance = (distance2 as f64).sqrt();
        let p = (self.volume as f64) / 100.0;
        let p = p * 0.75_f64.powf(distance);

        let rng = &mut *rng.borrow_mut();
        let x: f64 = rng.gen();
        (x <= p, p)
    }
}

impl Mul<f64> for Sound {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        let v = (self.volume as f64) * rhs;
        Sound { volume: v as i32 }
    }
}

impl Game {
    pub fn handle_noise(&mut self, origin: &Point, noise: Sound) {
        let npcs: Vec<(Point, i32)> = self
            .level
            .npcs()
            .map_while(|oid| {
                let loc = self.loc(oid).unwrap();
                let distance = origin.distance2(&loc);
                if distance <= 2 * pov::RADIUS * pov::RADIUS {
                    // We don't want to check every NPC since that's expensive and kinda
                    // pointless. So we'll only check twice the visible radius (which is
                    // still quite a bit).
                    Some((loc, distance))
                } else {
                    None
                }
            })
            .collect();

        for (loc, distance) in &npcs {
            let (was_heard, p) = noise.was_heard(&self.rng, *distance);
            if was_heard {
                if let Some((_, obj)) = self.level.get_mut(&loc, BEHAVIOR_ID) {
                    if responded_to_noise(obj, origin) {
                        // We could switch to attacking here if an enemy made the noise
                        // and is in sight. But we need to make that check anyway each
                        // time we handle MovingTo so there's little point in doing that
                        // here to.
                        debug!("{obj} heard a noise with prob {p:.2} and is now moving to {origin}");
                        self.replace_behavior(&loc, Behavior::MovingTo(*origin));
                    }
                }
            }
        }
    }
}

fn responded_to_noise(obj: &Object, origin: &Point) -> bool {
    match obj.value(BEHAVIOR_ID) {
        Some(Behavior::Attacking(_)) => false,
        Some(Behavior::MovingTo(_)) => false, // TODO: change target if the new noise is louder?
        Some(Behavior::Sleeping) => {
            debug!("{obj} stopped sleeping and is moving towards noise at {origin}");
            true
        }
        Some(Behavior::Wandering(_)) => {
            debug!("{obj} stopped wandering and is moving towards noise at {origin}");
            true
        }
        None => false,
    }
}
