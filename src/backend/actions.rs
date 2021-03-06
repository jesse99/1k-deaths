use super::*;

pub enum Scheduled {
    Yes,
    No,
}

impl Game {
    pub fn do_dig(&mut self, _oid: Oid, obj_loc: &Point, obj_oid: Oid, damage: i32) {
        assert!(damage > 0);

        let (damage, durability) = {
            let obj = self.level.get(&obj_loc, TERRAIN_ID).unwrap().1;
            let durability = obj.durability_value().unwrap();
            (durability.max / damage, durability)
        };
        debug!("digging at {obj_loc} for {damage} damage");

        if damage < durability.current {
            let mesg = Message::new(
                Topic::Normal,
                "You chip away at the wall with your pick-axe.", // TODO: probably should have slightly differet text for wooden walls (if we ever add them)
            );
            self.messages.push(mesg);

            let obj = self.level.get(&obj_loc, TERRAIN_ID).unwrap().1;
            let mut obj = obj.clone();
            obj.replace(Tag::Durability(Durability {
                current: durability.current - damage,
                max: durability.max,
            }));
            self.replace_object(obj_loc, obj_oid, obj);
        } else {
            let mesg = Message::new(Topic::Important, "You destroy the wall!");
            self.messages.push(mesg);
            self.destroy_object(obj_loc, obj_oid);
            self.pov.dirty();
        }
    }

    pub fn do_flood_deep(&mut self, oid: Oid, loc: Point) -> Scheduled {
        if let Some(new_loc) = self.find_neighbor(&loc, |candidate| {
            let obj = self.level.get(&candidate, TERRAIN_ID).unwrap().1;
            let terrain = obj.terrain_value().unwrap();
            terrain == Terrain::ShallowWater || terrain == Terrain::Ground || terrain == Terrain::Rubble
        }) {
            debug!("flood deep from {loc} to {new_loc}");
            let bad_oid = self.level.get(&new_loc, TERRAIN_ID).unwrap().0;
            self.replace_object(&new_loc, bad_oid, new_obj(ObjectName::DeepWater));

            if new_loc == self.player_loc() {
                if let Some(newer_loc) = self.find_neighbor(&self.player_loc(), |candidate| {
                    let obj = self.level.get(&candidate, TERRAIN_ID).unwrap().1;
                    let terrain = obj.terrain_value().unwrap();
                    terrain == Terrain::OpenDoor
                        || terrain == Terrain::ShallowWater
                        || terrain == Terrain::Ground
                        || terrain == Terrain::Rubble
                }) {
                    let mesg = Message {
                        topic: Topic::Normal,
                        text: "You step away from the rising water.".to_string(),
                    };
                    self.messages.push(mesg);

                    trace!("flood is moving player from {} to {}", self.player_loc(), newer_loc);
                    let player_loc = self.player_loc();
                    self.do_force_move(Oid(0), &player_loc, &newer_loc);

                    let units = if player_loc.diagnol(&newer_loc) {
                        time::DIAGNOL_MOVE
                    } else {
                        time::CARDINAL_MOVE
                    };
                    self.scheduler.force_acted(Oid(0), units, &self.rng);
                } else {
                    let mesg = Message {
                        topic: Topic::Important,
                        text: "You drown!".to_string(),
                    };
                    self.messages.push(mesg);

                    self.state = State::LostGame;
                }
            }
            Scheduled::Yes
        } else {
            // No where left to flood.
            self.scheduler.remove(oid);
            Scheduled::No
        }
    }

    pub fn do_flood_shallow(&mut self, oid: Oid, loc: Point) -> Scheduled {
        if let Some(new_loc) = self.find_neighbor(&loc, |candidate| {
            let obj = self.level.get(&candidate, TERRAIN_ID).unwrap().1;
            let terrain = obj.terrain_value().unwrap();
            terrain == Terrain::Ground || terrain == Terrain::Rubble
        }) {
            debug!("flood shallow from {loc} to {new_loc}");
            let bad_oid = self.level.get(&new_loc, TERRAIN_ID).unwrap().0;
            self.replace_object(&new_loc, bad_oid, new_obj(ObjectName::ShallowWater));
            Scheduled::Yes
        } else {
            // No where left to flood.
            self.scheduler.remove(oid);
            Scheduled::No
        }
    }

    pub fn do_force_move(&mut self, oid: Oid, old_loc: &Point, new_loc: &Point) {
        self.do_move(oid, old_loc, new_loc);

        let taken = if old_loc.diagnol(new_loc) {
            time::DIAGNOL_MOVE
        } else {
            time::CARDINAL_MOVE
        };
        let taken = taken + self.interact_post_move(new_loc).0;
        self.scheduler.force_acted(oid, taken, &self.rng);
    }

    pub fn do_move(&mut self, oid: Oid, old_loc: &Point, new_loc: &Point) {
        debug!("{oid} moving from {old_loc} to {new_loc}");

        self.level.moved(oid, old_loc, new_loc);
        if oid.0 == 0 {
            self.pov.dirty();
        }
    }

    pub fn do_open_door(&mut self, oid: Oid, ch_loc: &Point, obj_loc: &Point, obj_oid: Oid) {
        debug!("{oid} is opening the door at {obj_loc}");
        self.replace_object(obj_loc, obj_oid, new_obj(ObjectName::OpenDoor));
        self.do_move(oid, ch_loc, obj_loc);
        self.pov.dirty();
    }

    pub fn do_ignore(&mut self, oid: Oid, obj_loc: &Point, obj_oid: Oid, why: &str) {
        let obj = self.level.obj(obj_oid).0;
        debug!("{oid} is ignoring {obj_oid}/{obj} at {obj_loc}");
        let name: &'static str = obj.name_value().unwrap();
        let mesg = Message {
            topic: Topic::Normal,
            text: format!("{why} pick up the {name}."),
        };
        self.messages.push(mesg);
    }

    pub fn do_pick_up(&mut self, oid: Oid, obj_loc: &Point, obj_oid: Oid) {
        let obj = self.level.obj(obj_oid).0;
        debug!("{oid} is picking up {obj_oid}/{obj} at {obj_loc}");
        let name: &'static str = obj.name_value().unwrap();
        let mesg = Message {
            topic: Topic::Normal,
            text: format!("You pick up the {name}."),
        };
        self.messages.push(mesg);

        self.level.pickup(obj_loc, obj_oid);
    }

    pub fn do_shove_doorman(&mut self, oid: Oid, old_loc: &Point, ch: Oid, new_loc: &Point) {
        debug!("shoving doorman from {old_loc} to {new_loc}");
        self.do_force_move(ch, old_loc, new_loc);
        let player_loc = self.player_loc();
        self.do_move(oid, &player_loc, old_loc);
    }
}
