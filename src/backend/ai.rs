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
            match obj.value(BEHAVIOR_ID) {
                Some(Behavior::Attacking(defender)) => return attack(game, oid, defender, units),
                Some(Behavior::Sleeping) => return Acted::DidntAct, // TODO: implement this?
                Some(Behavior::Wandering(end)) => return wander(game, oid, end, units),
                None => (),
            }
            panic!("{obj} is scheduled but has no ai handler");
        }
    } else {
        Acted::Removed
    }
}

fn attack(game: &mut Game, attacker: Oid, defender: Oid, units: Time) -> Acted {
    if units >= time::BASE_ATTACK {
        let attacker_loc = game.loc(attacker).unwrap();
        let defender_loc = game.loc(defender).unwrap();
        game.do_melee_attack(&attacker_loc, &defender_loc);
        Acted::Acted(time::BASE_ATTACK) // TODO: should be scaled by weapon speed
    } else {
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

fn wander(game: &mut Game, oid: Oid, end: Time, units: Time) -> Acted {
    let loc = game.loc(oid).unwrap();
    if game.scheduler.now() > end {
        let obj = game.level.get_mut(&loc, BEHAVIOR_ID).unwrap().1;
        debug!("{obj} stopped wandering");
        obj.replace(Tag::Behavior(Behavior::Sleeping));
        Acted::DidntAct
    } else if units >= DIAGNOL_MOVE {
        let obj = game.level.get(&loc, BEHAVIOR_ID).unwrap().1;
        if let Some(new_loc) = game.find_empty_cell(obj, &loc) {
            game.do_move(oid, &loc, &new_loc);
            if loc.diagnol(&new_loc) {
                Acted::Acted(DIAGNOL_MOVE) // TODO: probably should do post move interactions
            } else {
                Acted::Acted(CARDINAL_MOVE)
            }
        } else {
            Acted::DidntAct
        }
    } else {
        Acted::DidntAct
    }
}
