use super::*;

impl Game {
    pub fn do_dig(&mut self, _oid: Oid, obj_loc: &Point, obj_oid: Oid, damage: i32) {
        assert!(damage > 0);

        let (damage, durability) = {
            let obj = self.get(&obj_loc, WALL_ID).unwrap().1;
            let durability: Durability = obj.value(DURABILITY_ID).unwrap();
            (durability.max / damage, durability)
        };
        debug!("digging at {obj_loc} for {damage} damage");

        if damage < durability.current {
            let mesg = Message::new(
                Topic::Normal,
                "You chip away at the wall with your pick-axe.", // TODO: probably should have slightly differet text for wooden walls (if we ever add them)
            );
            self.messages.push(mesg);

            let obj = self.get(&obj_loc, WALL_ID).unwrap().1;
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

    pub fn do_fight_rhulad(&mut self, _oid: Oid, char_loc: &Point, ch: Oid) {
        debug!("fighting Rhulad at {char_loc}");
        let mesg = Message::new(Topic::Important, "After an epic battle you kill the Emperor!");
        self.messages.push(mesg);

        self.destroy_object(char_loc, ch);
        self.add_object(char_loc, make::emp_sword());
        self.state = State::KilledRhulad;
    }

    pub fn do_flood_deep(&mut self, oid: Oid, loc: Point) {
        if let Some(new_loc) = self.find_neighbor(&loc, |candidate| {
            self.get(candidate, GROUND_ID).is_some() || self.get(candidate, SHALLOW_WATER_ID).is_some()
        }) {
            debug!("flood deep from {loc} to {new_loc}");
            let bad_oid = self.get(&new_loc, TERRAIN_ID).unwrap().0;
            self.replace_object(&new_loc, bad_oid, make::deep_water());

            if new_loc == self.player {
                if let Some(newer_loc) = self.find_neighbor(&self.player, |candidate| {
                    self.get(candidate, GROUND_ID).is_some()
                        || self.get(candidate, SHALLOW_WATER_ID).is_some()
                        || self.get(candidate, OPEN_DOOR_ID).is_some()
                }) {
                    let mesg = Message {
                        topic: Topic::Normal,
                        text: "You step away from the rising water.".to_string(),
                    };
                    self.messages.push(mesg);

                    trace!("flood is moving player from {} to {}", self.player, newer_loc);
                    let player_loc = self.player;
                    self.do_move(Oid(0), &player_loc, &newer_loc);

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
        } else {
            // No where left to flood.
            self.scheduler.remove(oid);
        }
    }

    pub fn do_flood_shallow(&mut self, oid: Oid, loc: Point) {
        if let Some(new_loc) = self.find_neighbor(&loc, |candidate| self.get(candidate, GROUND_ID).is_some()) {
            debug!("flood shallow from {loc} to {new_loc}");
            let bad_oid = self.get(&new_loc, TERRAIN_ID).unwrap().0;
            self.replace_object(&new_loc, bad_oid, make::shallow_water());
        } else {
            // No where left to flood.
            self.scheduler.remove(oid);
        }
    }

    pub fn do_move(&mut self, oid: Oid, old_loc: &Point, new_loc: &Point) {
        assert!(!self.constructing); // make sure this is reset once things start happening
        debug!("{oid} moving from {old_loc} to {new_loc}");

        let oids = self.cells.get_mut(&old_loc).unwrap();
        let index = oids
            .iter()
            .position(|id| *id == oid)
            .expect(&format!("expected {oid} at {old_loc}"));
        oids.remove(index);
        let cell = self.cells.entry(*new_loc).or_insert_with(Vec::new);
        cell.push(oid);

        if oid.0 == 0 {
            self.player = *new_loc;
            self.pov.dirty();
        }

        // TODO: player actions should be in a table so that we can ensure that they
        // schedule properly
        let taken = if old_loc.diagnol(new_loc) {
            time::DIAGNOL_MOVE
        } else {
            time::CARDINAL_MOVE
        };
        let taken = taken + self.interact_post_move(new_loc);
        self.scheduler.force_acted(oid, taken, &self.rng);
    }

    pub fn do_open_door(&mut self, oid: Oid, ch_loc: &Point, obj_loc: &Point, obj_oid: Oid) {
        debug!("{oid} is opening the door at {obj_loc}");
        self.replace_object(obj_loc, obj_oid, make::open_door());
        self.do_move(oid, ch_loc, obj_loc);
        self.pov.dirty();
    }

    pub fn do_pick_up(&mut self, oid: Oid, obj_loc: &Point, obj_oid: Oid) {
        let obj = self.objects.get(&obj_oid).unwrap();
        debug!("{oid} is picking up {obj} at {obj_loc}");
        let name: String = obj.value(NAME_ID).unwrap();
        let mesg = Message {
            topic: Topic::Normal,
            text: format!("You pick up the {name}."),
        };
        self.messages.push(mesg);
        {
            let oids = self.cells.get_mut(&obj_loc).unwrap();
            let index = oids.iter().position(|id| *id == obj_oid).unwrap();
            oids.remove(index);
        }
        self.mutate(&obj_loc, INVENTORY_ID, |obj| {
            let inv = obj.as_mut_ref(INVENTORY_ID).unwrap();
            inv.push(obj_oid);
        });
    }

    pub fn do_shove_doorman(&mut self, oid: Oid, old_loc: &Point, ch: Oid, new_loc: &Point) {
        debug!("shoving doorman from {old_loc} to {new_loc}");
        self.do_move(ch, old_loc, new_loc);
        let player_loc = self.player;
        self.do_move(oid, &player_loc, old_loc);
    }
}
