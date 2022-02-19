use super::time::*;
use super::*;

/// Returns Some((duration, extra)) if the object decided to perform an action. extra is
/// additional time subtracted from the object's time units but not added to other objects
/// time units (it's used with objects that want to schedule future actions further into
/// the future than would normally be the case).
pub fn acted(game: &mut Game, oid: Oid, units: Time) -> Option<(Time, Time)> {
    let obj = game.level.obj(oid).0;
    if obj.has(DEEP_WATER_ID) {
        deep_flood(game, oid, units)
    } else if obj.has(SHALLOW_WATER_ID) {
        shallow_flood(game, oid, units)
    } else if obj.has(SHALLOW_WATER_ID) {
        shallow_flood(game, oid, units)
    } else {
        match obj.value(BEHAVIOR_ID) {
            Some(Behavior::Wandering(end)) => return wander(game, oid, end, units),
            Some(Behavior::Sleeping) => return None, // TODO: implement this?
            None => (),
        }
        panic!("{obj} is scheduled but has no ai handler");
    }
}

pub fn extra_flood_delay(game: &Game) -> Time {
    let rng = &mut *game.rng();
    let t: i64 = 60 + rng.gen_range(0..(400 * 6));
    time::secs(t)
}

fn deep_flood(game: &mut Game, oid: Oid, units: Time) -> Option<(Time, Time)> {
    let base = time::FLOOD;
    let extra = extra_flood_delay(game);
    if units >= base + extra {
        let loc = game.loc(oid).unwrap();
        trace!("{oid} at {loc} is deep flooding");
        game.do_flood_deep(oid, loc);
        Some((base, extra))
    } else {
        None
    }
}

fn shallow_flood(game: &mut Game, oid: Oid, units: Time) -> Option<(Time, Time)> {
    let base = time::FLOOD;
    let extra = extra_flood_delay(game);
    if units >= base + extra {
        let loc = game.loc(oid).unwrap();
        trace!("{oid} at {loc} is shallow flooding");
        game.do_flood_shallow(oid, loc);
        Some((base, extra))
    } else {
        None
    }
}

fn wander(game: &mut Game, oid: Oid, end: Time, units: Time) -> Option<(Time, Time)> {
    let loc = game.loc(oid).unwrap();
    if game.scheduler.now() > end {
        let obj = game.level.get_mut(&loc, BEHAVIOR_ID).unwrap().1;
        debug!("{obj} stopped wandering");
        obj.replace(Tag::Behavior(Behavior::Sleeping));
        None
    } else if units >= DIAGNOL_MOVE {
        let obj = game.level.get(&loc, BEHAVIOR_ID).unwrap().1;
        if let Some(new_loc) = game.find_empty_cell(obj, &loc) {
            game.do_move(oid, &loc, &new_loc);
            if loc.diagnol(&new_loc) {
                Some((DIAGNOL_MOVE, Time::zero())) // TODO: probably should do post move interactions
            } else {
                Some((CARDINAL_MOVE, Time::zero()))
            }
        } else {
            None
        }
    } else {
        None
    }
}
