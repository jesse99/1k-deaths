use super::actions::Scheduled;
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
            info!("handing ai for {obj} {oid}");
            match obj.value(BEHAVIOR_ID) {
                Some(Behavior::Attacking(defender)) => attack(game, oid, defender, units),
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

fn attack(game: &mut Game, attacker: Oid, defender: Oid, units: Time) -> Acted {
    let attacker_loc = game.loc(attacker).unwrap();
    let defender_loc = game.loc(defender).unwrap();
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
                debug!("{attacker} stopping attacking {defender} and started wandering");
                let duration = time::DIAGNOL_MOVE * 8;
                game.replace_behavior(&attacker_loc, Behavior::Wandering(duration));
                Acted::DidntAct
            }
        } else {
            Acted::DidntAct
        }
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

fn move_towards(game: &mut Game, oid: Oid, target_loc: &Point, units: Time) -> Acted {
    if let Some(acted) = switched_to_attacking(game, oid, units) {
        info!("{oid} switched to attacking");
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

fn can_move(game: &Game, oid: Oid, new_loc: &Point) -> bool {
    if game.level.get(&new_loc, CHARACTER_ID).is_none() {
        let obj = &game.level.obj(oid).0;
        let (_, terrain) = game.level.get_bottom(&new_loc);
        obj.impassible_terrain(terrain).is_none()
    } else {
        false
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

fn switched_to_attacking(game: &mut Game, oid: Oid, units: Time) -> Option<Acted> {
    if let Some(loc) = game.loc(oid) {
        if game.pov.visible(game, &loc) {
            let obj = game.level.get_mut(&loc, BEHAVIOR_ID).unwrap().1;
            if let Some(Disposition::Aggressive) = obj.value(DISPOSITION_ID) {
                // we're treating visibility as a symmetric operation, TODO: which is probably not quite right
                debug!("{obj} switched to attacking player");
                game.replace_behavior(&loc, Behavior::Attacking(Oid(0)));
                return Some(attack(game, oid, Oid(0), units));
            }
        }
    }
    None
}

fn try_move_towards(game: &mut Game, oid: Oid, target_loc: &Point) -> Option<Acted> {
    let old_loc = game.loc(oid).unwrap();
    let dx = if target_loc.x > old_loc.x {
        // TODO: need to avoid obstacles
        1
    } else if target_loc.x < old_loc.x {
        -1
    } else {
        0
    };
    let dy = if target_loc.y > old_loc.y {
        1
    } else if target_loc.y < old_loc.y {
        -1
    } else {
        0
    };
    let new_loc = Point::new(old_loc.x + dx, old_loc.y + dy);
    if can_move(game, oid, &new_loc) {
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
