// Note that the probabilities listed below were computed with scripts/sound_prob.py.
use super::primitives::PathFind;
use super::*;
use rand::rngs::SmallRng;
use rand::Rng;
use std::cell::RefCell;
use std::ops::{Add, AddAssign, Mul};

/// Volume represents the percent chance that an NPC will wake up if it is on top of the
/// noise source. The probability goes down according to 1/distance^1.2 so there is always
/// a chance that an NPC will wake up (inside the cutoff point anyway).
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Sound {
    volume: i32,
}

pub const NONE: Sound = Sound { volume: 0 };

// /// This corresponds to something like resting. Percentages for this work out to:
// /// 1.0 0.4 0.3 0.2 0.1 0.1 0.1 0.1 0.1 0.1 0.1 0.1 0.0.
// pub const SUPER_QUIET: Sound = Sound { volume: 1 };

/// This corresponds to something like drinking a potion. Percentages for this work out to:
/// 20.0 8.7 5.4 3.8 2.9 2.3 1.9 1.6 1.4 1.3 1.1 1.0 0.9.
pub const VERY_QUIET: Sound = Sound { volume: 20 };

/// This corresponds to something like the base movement noise. Percentages for this work
/// out to: 100.0 65.3 40.1 28.4 21.7 17.5 14.5 12.4 10.7 9.5 8.4 7.6 6.9.
pub const QUIET: Sound = Sound { volume: 150 };

// /// This corresponds to something like a creaky door. Percentages for this work out to:
// /// 100.0 87.1 53.5 37.9 29.0 23.3 19.4 16.5 14.3 12.6 11.3 10.1 9.2.
// pub const NOISY: Sound = Sound { volume: 200 };

/// This corresponds to something like a yell. Percentages for this work out to:
/// 100.0 100.0 100.0 94.7 72.5 58.2 48.4 41.2 35.8 31.5 28.1 25.3 23.0.
pub const LOUD: Sound = Sound { volume: 500 };

// /// This corresponds to something like a cusser exploding. Percentages for this work out to:
// /// 100.0 100.0 100.0 100.0 100.0 100.0 96.8 82.5 71.6 63.1 56.3 50.7 46.1.
// pub const VERY_LOUD: Sound = Sound { volume: 1000 };

impl Sound {
    fn was_heard(&self, rng: &RefCell<SmallRng>, distance10: i32, hearing: i32) -> (bool, f64) {
        let scaling = (hearing as f64) / 100.0;
        let distance = (distance10 as f64) / 10.0;
        let p = (self.volume as f64) / distance.powf(1.2);
        let p = p * scaling;

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

impl Add for Sound {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Sound {
            volume: self.volume + rhs.volume,
        }
    }
}

impl AddAssign for Sound {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self {
            volume: self.volume + rhs.volume,
        };
    }
}

impl Game {
    pub fn handle_noise(&mut self, origin: &Point, noise: Sound) {
        // Almost all noises are going to be in the vicinity of the player so we can use
        // level.npcs which is sorted by distance from the player. But we'll want to
        // extend our cutoff point depending upon how far the origin is from the player.
        // TODO: if this becomes an issue we could look at using the rstar crate to find
        // the NPCs near an arbitrary location (not sure how well that'd work with lots
        // of movement though).
        let delta2 = origin.distance2(&self.player_loc());
        let npcs: Vec<Point> = self
            .level
            .npcs()
            .map_while(|oid| {
                let loc = self.loc(oid).unwrap();
                let distance2 = origin.distance2(&loc);
                if distance2 <= pov::RADIUS * pov::RADIUS + delta2 + 4 * 4 {
                    // We don't want to check every NPC since that's expensive and kinda
                    // pointless. So currently we check out to the pov radius + 4.
                    Some(loc)
                } else {
                    None
                }
            })
            .collect();

        for loc in &npcs {
            if let Some(distance10) = self.find_distance10(origin, loc) {
                let hearing: i32 = {
                    if let Some((_, obj)) = self.level.get(&loc, HEARING_ID) {
                        object::hearing_value(obj).unwrap()
                    } else {
                        100
                    }
                };
                let (was_heard, p) = noise.was_heard(&self.rng, distance10, hearing);
                if was_heard {
                    if let Some((_, obj)) = self.level.get_mut(&loc, BEHAVIOR_ID) {
                        if responded_to_noise(obj, origin) {
                            // We could switch to attacking here if an enemy made the noise
                            // and is in sight. But we need to make that check anyway each
                            // time we handle MovingTo so there's little point in doing that
                            // here too.
                            debug!(
                                "{obj} heard a noise and is now moving to {origin}, prob={p:.2}, dist={:.1}",
                                (distance10 as f64) / 10.0
                            );
                            self.replace_behavior(&loc, Behavior::MovingTo(*origin));
                            // } else {
                            //     info!(
                            //         "{obj} heard a noise but ignored it, prob={p:.2}, dist={:.1}",
                            //         (distance10 as f64) / 10.0
                            //     );
                        }
                    }
                    // } else {
                    //     if let Some((_, obj)) = self.level.get(&loc, CHARACTER_ID) {
                    //         info!(
                    //             "{obj} did not hear a noise, prob={p:.2}, dist={:.1}",
                    //             (distance10 as f64) / 10.0
                    //         );
                    //     }
                }
            }
        }
    }

    // Returns the distance sound must travel to reach target from origin. Note that this
    // is a bit different from movement distance because sound travels over things like
    // deep water and sound travels through closed/locked doots (although when that happens
    // the distance is artificially extended).
    fn find_distance10(&self, start: &Point, target: &Point) -> Option<i32> {
        let callback = |loc: Point, neighbors: &mut Vec<(Point, i32)>| self.successors(loc, neighbors);
        let find = PathFind::new(*start, *target, callback);
        find.distance()
    }

    fn successors(&self, loc: Point, neighbors: &mut Vec<(Point, i32)>) {
        let deltas = vec![(-1, -1), (-1, 1), (-1, 0), (1, -1), (1, 1), (1, 0), (0, -1), (0, 1)];
        for delta in deltas {
            let new_loc = Point::new(loc.x + delta.0, loc.y + delta.1);
            let (_, obj) = self.level.get_bottom(&new_loc);
            let mut d = match object::terrain_value(obj).unwrap() {
                // sound travels through everything but can be very attenuated
                Terrain::ClosedDoor => 50,
                Terrain::DeepWater => 10,
                Terrain::Ground => 10,
                Terrain::OpenDoor => 10,
                Terrain::Rubble => 10,
                Terrain::ShallowWater => 10,
                Terrain::Tree => 15,
                Terrain::Vitr => 10,
                Terrain::Wall => 100,
            };
            if loc.diagnol(&new_loc) {
                d += 12 * d / 10;
            }
            neighbors.push((new_loc, d));
        }
    }
}

fn responded_to_noise(obj: &Object, origin: &Point) -> bool {
    if obj.has(SPECTATOR_ID) {
        return false;
    }
    match object::behavior_value(obj) {
        Some(Behavior::Attacking(_, _)) => false,
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
