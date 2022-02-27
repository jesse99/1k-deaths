use super::actions::Scheduled;
use super::primitives::PathFind;
use super::time::*;
use super::*;

pub enum Acted {
    /// An object did something that took time.
    Acted(Time),

    /// An object elected to do nothing (either it doesn't have enough time to do anything
    /// or it decided to wait to do something better when it has more time).
    DidntAct,

    /// The object de-scheduled itself (this could have been as the result of an operation
    /// that takes time but that's moot because it's descheduling).
    Removed,
}

/// Returns Some((duration, extra)) if the object decided to perform an action. extra is
/// additional time subtracted from the object's time units but not added to other objects
/// time units (it's used with objects that want to schedule future actions further into
/// the future than would normally be the case).
pub fn acted(game: &mut Game, oid: Oid, units: Time) -> Acted {
    if let Some(obj) = game.level.try_obj(oid) {
        if obj.has(DEEP_WATER_ID) {
            deep_flood(game, oid, units)
        } else if obj.has(SHALLOW_WATER_ID) {
            shallow_flood(game, oid, units)
        } else if obj.has(SHALLOW_WATER_ID) {
            shallow_flood(game, oid, units)
        } else {
            // TODO: will have to special case alternate goals, eg
            // whether to go grab a good item that is in los
            // whether to move closer to group/pack leader
            //
            // Currently NPCs don't make noise which is probably OK.
            match obj.value(BEHAVIOR_ID) {
                Some(Behavior::Attacking(defender, defender_loc)) => attack(game, oid, defender, defender_loc, units),
                Some(Behavior::MovingTo(loc)) => move_towards(game, oid, &loc, units),
                Some(Behavior::Sleeping) => Acted::DidntAct, // NPCs transition out of this via handle_noise
                Some(Behavior::Wandering(end)) => wander(game, oid, end, units),
                None => panic!("{obj} is scheduled but has no ai handler"),
            }
        }
    } else {
        Acted::Removed
    }
}

fn attack(game: &mut Game, attacker: Oid, defender: Oid, old_defender_loc: Point, units: Time) -> Acted {
    let attacker_loc = game.loc(attacker).unwrap();
    let defender_loc = game.loc(defender).unwrap();

    if wants_to_flee(game, &attacker_loc) {
        if start_fleeing(game, attacker, &attacker_loc, defender, &defender_loc) {
            return Acted::DidntAct;
        }
    }

    // TODO: this assumes that the defender is the player (or the pair are in the player's pov)
    if game.pov.visible(game, &defender_loc) {
        // If the defender can be seen then update where the attacker thinks he is,
        if defender_loc != old_defender_loc {
            let behavior = Behavior::Attacking(defender, defender_loc);
            game.replace_behavior(&attacker_loc, behavior);
        }

        // and either attack him or move towards his actual location.
        if attacker_loc.adjacent(&defender_loc) {
            if units >= time::BASE_ATTACK {
                game.do_melee_attack(&attacker_loc, &defender_loc);
                Acted::Acted(time::BASE_ATTACK) // TODO: should be scaled by weapon speed
            } else {
                Acted::DidntAct
            }
        } else {
            if units >= time::DIAGNOL_MOVE {
                if let Some(acted) = try_move_towards(game, attacker, &defender_loc) {
                    acted
                } else {
                    debug!("{attacker} couldn't attack {defender} and started wandering");
                    let duration = time::DIAGNOL_MOVE * 8;
                    game.replace_behavior(&attacker_loc, Behavior::Wandering(duration));
                    Acted::DidntAct
                }
            } else {
                Acted::DidntAct
            }
        }
    } else {
        // If the defender cannot be seen then move towards his last known location.
        debug!("{attacker} can no longer see {defender} and has started moving towards his last known location");
        let behavior = Behavior::MovingTo(old_defender_loc);
        game.replace_behavior(&attacker_loc, behavior);
        Acted::DidntAct
    }
}

pub fn extra_flood_delay(game: &Game) -> Time {
    let rng = &mut *game.rng();
    let t: i64 = 60 + rng.gen_range(0..(400 * 6));
    time::secs(t)
}

fn deep_flood(game: &mut Game, oid: Oid, units: Time) -> Acted {
    if units >= time::FLOOD {
        let flood = {
            let rng = &mut *game.rng();
            rng.gen_bool(0.05)
        };
        let loc = game.loc(oid).unwrap();
        if flood {
            trace!("{oid} at {loc} is deep flooding");

            match game.do_flood_deep(oid, loc) {
                Scheduled::Yes => (),
                Scheduled::No => return Acted::Removed,
            }
        } else {
            trace!("{oid} at {loc} skipped deep flooding");
        }
        Acted::Acted(time::FLOOD)
    } else {
        Acted::DidntAct
    }
}

/// Returns the next location from start to target using the lowest Time path.
fn find_next_loc_to(game: &Game, ch: &Object, start: &Point, target: &Point) -> Option<Point> {
    let callback = |loc: Point, neighbors: &mut Vec<(Point, Time)>| successors(game, ch, loc, target, neighbors);
    let find = PathFind::new(*start, *target, callback);
    find.next()
}

fn successors(game: &Game, ch: &Object, loc: Point, target: &Point, neighbors: &mut Vec<(Point, Time)>) {
    let deltas = vec![(-1, -1), (-1, 1), (-1, 0), (1, -1), (1, 1), (1, 0), (0, -1), (0, 1)];
    for delta in deltas {
        let new_loc = Point::new(loc.x + delta.0, loc.y + delta.1);
        let character = &game.level.get(&new_loc, CHARACTER_ID);
        if character.is_none() || new_loc == *target {
            let (_, terrain) = game.level.get_bottom(&new_loc);
            if ch.impassible_terrain(terrain).is_none() {
                if loc.diagnol(&new_loc) {
                    neighbors.push((new_loc, time::DIAGNOL_MOVE)); // TODO: should also factor in a post-move handler
                } else {
                    neighbors.push((new_loc, time::CARDINAL_MOVE));
                }
            }
        }
    }
}

fn move_towards(game: &mut Game, oid: Oid, target_loc: &Point, units: Time) -> Acted {
    if let Some(acted) = switched_to_attacking(game, oid, units) {
        info!("{oid} was moving towards {target_loc} but switched to attacking");
        acted
    } else if units >= time::DIAGNOL_MOVE {
        if let Some(acted) = try_move_towards(game, oid, target_loc) {
            info!("{oid} did move towards {target_loc}");
            acted
        } else {
            let old_loc = game.loc(oid).unwrap();
            debug!("{oid} stopping moving towards {target_loc} and started wandering");
            let duration = time::DIAGNOL_MOVE * 8;
            game.replace_behavior(&old_loc, Behavior::Wandering(duration));
            Acted::DidntAct
        }
    } else {
        info!("{oid} didn't have enough time to move: {units}");
        Acted::DidntAct
    }
}

fn shallow_flood(game: &mut Game, oid: Oid, units: Time) -> Acted {
    if units >= time::FLOOD {
        let flood = {
            let rng = &mut *game.rng();
            rng.gen_bool(0.05)
        };
        let loc = game.loc(oid).unwrap();
        if flood {
            trace!("{oid} at {loc} is shallow flooding");
            match game.do_flood_shallow(oid, loc) {
                Scheduled::Yes => (),
                Scheduled::No => return Acted::Removed,
            }
        } else {
            trace!("{oid} at {loc} skipped shallow flooding");
        }
        Acted::Acted(time::FLOOD)
    } else {
        Acted::DidntAct
    }
}

fn find_flee_loc(game: &Game, attacker_loc: &Point, defender_loc: &Point) -> Option<Point> {
    let mut loc = None;
    let mut def_dist2 = 0;
    let mut total_dist2 = 0;
    let attacker = game.level.get(attacker_loc, CHARACTER_ID).unwrap().1;

    // TODO: this isn't great because it's possible that the only place to flee is a relatively
    // small portion of the level. Though that's not the worst thing in the world because it'll
    // look like the NPC simply decided not to flee.
    for _ in 0..40 {
        let candidate = game.level.random_loc(&game.rng);
        let d2 = attacker_loc.distance2(&candidate);
        // don't bother fleeing to a really close point
        if d2 > 4 * 4 {
            let callback = |new_loc: Point, neighbors: &mut Vec<(Point, Time)>| {
                successors(game, attacker, new_loc, &candidate, neighbors)
            };
            let find = PathFind::new(*attacker_loc, candidate, callback);
            if let Some(next_loc) = find.next() {
                // Prefer points where the attacker flees away from the defender (as opposed
                // to sliding around the defender).
                let dd2 = defender_loc.distance2(&next_loc);
                if dd2 > def_dist2 || (dd2 == def_dist2 && d2 > total_dist2) {
                    loc = Some(candidate);
                    def_dist2 = dd2;
                    total_dist2 = d2;
                    if dd2 > 2 && d2 > (3 * pov::RADIUS) * (3 * pov::RADIUS) {
                        // We've found something plenty far enough away.
                        break;
                    }
                }
            }
        }
    }
    loc
}

fn start_fleeing(game: &mut Game, attacker: Oid, attacker_loc: &Point, defender: Oid, defender_loc: &Point) -> bool {
    if let Some(flee_loc) = find_flee_loc(game, attacker_loc, defender_loc) {
        debug!("{attacker} is hurt and has started fleeing from {defender}");
        let behavior = Behavior::MovingTo(flee_loc);
        game.replace_behavior(&attacker_loc, behavior);
        true
    } else {
        debug!("{attacker} is hurt and wanted to flee but was unable to");
        false
    }
}

fn switched_to_attacking(game: &mut Game, oid: Oid, units: Time) -> Option<Acted> {
    if let Some(loc) = game.loc(oid) {
        if game.pov.visible(game, &loc) && !wants_to_flee(game, &loc) {
            let obj = game.level.get_mut(&loc, BEHAVIOR_ID).unwrap().1;
            if let Some(Disposition::Aggressive) = obj.value(DISPOSITION_ID) {
                // we're treating visibility as a symmetric operation, TODO: which is probably not quite right
                game.replace_behavior(&loc, Behavior::Attacking(Oid(0), game.player_loc()));
                return Some(attack(game, oid, Oid(0), game.player_loc(), units));
            }
        }
    }
    None
}

fn try_move_towards(game: &mut Game, oid: Oid, target_loc: &Point) -> Option<Acted> {
    let ch = &game.level.obj(oid).0;
    let old_loc = game.loc(oid).unwrap();
    if old_loc == *target_loc {
        return None; // we're at the target so we're no longer moving towards it
    }

    if let Some(new_loc) = find_next_loc_to(game, ch, &old_loc, target_loc) {
        game.do_move(oid, &old_loc, &new_loc);
        if old_loc.diagnol(&new_loc) {
            Some(Acted::Acted(DIAGNOL_MOVE)) // TODO: probably should do post move interactions
        } else {
            Some(Acted::Acted(CARDINAL_MOVE))
        }
    } else {
        None
    }
}

fn wander(game: &mut Game, oid: Oid, end: Time, units: Time) -> Acted {
    if let Some(acted) = switched_to_attacking(game, oid, units) {
        info!("{oid} was wandering but switched to attacking");
        return acted;
    }
    let loc = game.loc(oid).unwrap();
    if game.scheduler.now() > end {
        debug!("{oid} stopped wandering");
        game.replace_behavior(&loc, Behavior::Sleeping);
        return Acted::DidntAct;
    } else if units >= DIAGNOL_MOVE {
        let obj = game.level.get(&loc, BEHAVIOR_ID).unwrap().1;
        if let Some(new_loc) = game.find_empty_cell(obj, &loc) {
            game.do_move(oid, &loc, &new_loc);
            if loc.diagnol(&new_loc) {
                return Acted::Acted(DIAGNOL_MOVE); // TODO: probably should do post move interactions
            } else {
                return Acted::Acted(CARDINAL_MOVE);
            }
        }
    }
    Acted::DidntAct
}

fn wants_to_flee(game: &Game, attacker_loc: &Point) -> bool {
    let attacker = game.level.get(attacker_loc, CHARACTER_ID).unwrap().1;
    if let Some(percent) = object::flees_value(attacker) {
        let durability: Durability = attacker.value(DURABILITY_ID).unwrap();
        let x = (durability.current as f64) / (durability.max as f64);
        x <= (percent as f64) / 100.0
    } else {
        false
    }
}
