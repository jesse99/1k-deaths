use super::*;

impl Game {
    pub fn do_melee_attack(&mut self, attacker_loc: &Point, defender_loc: &Point) {
        // It'd be more efficient to use Objects here but the borrow checker whines a lot.
        let attacker = self.level.get(attacker_loc, CHARACTER_ID).unwrap().0;
        let defender = self.level.get_mut(defender_loc, CHARACTER_ID).unwrap().0;
        debug!("{attacker} is meleeing {defender}");

        self.react_to_attack(attacker_loc, attacker, defender_loc);

        let attacker_name = self.attacker_name(attacker);
        let defender_name = self.defender_name(defender);
        if self.missed(attacker, defender) {
            let msg = format!("{attacker_name} missed {defender_name}.");
            let mesg = Message::new(Topic::Normal, &msg);
            self.messages.push(mesg);
        } else {
            let (damage, crit) = self.base_damage(attacker);
            let damage = self.mitigate_damage(attacker, defender, damage);
            let (new_hps, max_hps) = self.hps(defender, damage);
            let hit = if crit { "critcally hit" } else { "hit" };
            debug!("   {hit} for {damage}, new HPs are {new_hps}");
            let msg = if damage == 0 {
                format!("{attacker_name} {hit} {defender_name} for no damage.")
            } else {
                let (oid, defender) = self.level.get_mut(defender_loc, CHARACTER_ID).unwrap();
                let durability = Tag::Durability(Durability {
                    current: new_hps,
                    max: max_hps,
                });
                defender.replace(durability);

                if new_hps <= 0 {
                    if oid.0 == 0 {
                        let msg = "You've lost the game!";
                        let mesg = Message::new(Topic::Important, msg);
                        self.messages.push(mesg);
                        self.state = State::LostGame;
                    } else {
                        self.npc_died(defender_loc, oid);
                    }
                    if new_hps < 0 {
                        format!(
                            "{attacker_name} {hit} {defender_name} for {damage} damage ({} over kill).",
                            -new_hps
                        )
                    } else {
                        format!("{attacker_name} {hit} {defender_name} for {damage} damage.",)
                    }
                } else {
                    format!("{attacker_name} {hit} {defender_name} for {damage} damage.")
                }
            };

            let topic = self.topic(attacker, defender, damage);
            let mesg = Message::new(topic, &msg);
            self.messages.push(mesg);
        };
    }
}

impl Game {
    fn attacker_name(&self, attacker_id: Oid) -> String {
        if attacker_id.0 == 0 {
            "You".to_string()
        } else {
            let attacker = self.level.obj(attacker_id).0;
            let name: &'static str = object::name_value(attacker).unwrap();
            format!("{name}")
        }
    }

    // TODO: use strength/weapon skill
    fn base_damage(&self, attacker_id: Oid) -> (i32, bool) {
        let attacker = self.level.obj(attacker_id).0;
        // TODO: this should be using what is eqiupped instead of a max of unarmed and inv weapon damage
        let mut damage = object::damage_value(attacker).expect(&format!("{attacker_id} should have a damage tag"));
        if let Some(oids) = object::inventory_value(attacker) {
            for oid in oids {
                let obj = self.level.obj(*oid).0;
                if let Some(candidate) = object::damage_value(obj) {
                    damage = max(damage, candidate);
                }
            }
        }
        let crit = self.rng.borrow_mut().gen_bool(0.05); // TODO: should be scaled by skill
        if crit {
            damage *= 2;
        }
        (super::rand_normal32(damage, 20, &self.rng), crit)
    }

    fn defender_name(&self, defender_id: Oid) -> String {
        if defender_id.0 == 0 {
            "you".to_string()
        } else {
            let defender = self.level.obj(defender_id).0;
            format!("{defender}")
        }
    }

    fn hps(&self, defender_id: Oid, damage: i32) -> (i32, i32) {
        let defender = self.level.obj(defender_id).0;
        let durability = object::durability_value(defender).unwrap();
        (durability.current - damage, durability.max)
    }

    // TODO: use dexterity/evasion
    fn missed(&self, _attacker_id: Oid, defender_id: Oid) -> bool {
        let defender = self.level.obj(defender_id).0;
        let rng = &mut *self.rng();
        if defender.has(PLAYER_ID) {
            rng.gen_bool(0.1)
        } else if defender.has(ICARIUM_ID) {
            rng.gen_bool(0.2)
        } else {
            rng.gen_bool(0.1)
        }
    }

    // TODO: use skill and armor
    fn mitigate_damage(&self, _attacker_id: Oid, defender_id: Oid, damage: i32) -> i32 {
        let defender = self.level.obj(defender_id).0;
        let scaling = if defender.has(PLAYER_ID) {
            0.9
        } else if defender.has(ICARIUM_ID) {
            0.8
        } else {
            0.9
        };
        (scaling * (damage as f64)) as i32
    }

    fn npc_died(&mut self, defender_loc: &Point, defender_id: Oid) {
        let defender = self.level.obj(defender_id).0;
        let is_rhulad = defender.has(RHULAD_ID);

        self.destroy_object(defender_loc, defender_id);

        if is_rhulad {
            self.add_object(defender_loc, make::emp_sword()); // TODO: should drop inv items
            self.state = State::KilledRhulad;

            let msg = "The Crippled God whispers, 'You shall pay for this mortal'.";
            let mesg = Message::new(Topic::Important, &msg);
            self.messages.push(mesg);
            self.spawn_the_broken();
        }
    }

    fn react_to_attack(&mut self, attacker_loc: &Point, attacker_id: Oid, defender_loc: &Point) {
        let defender = self.level.get_mut(defender_loc, CHARACTER_ID).unwrap().1;
        let attack = match object::behavior_value(defender) {
            Some(Behavior::Sleeping) => true,
            Some(Behavior::Attacking(_, _)) => {
                // TODO: If the old attacker is no longer visible (or maybe too far away)
                // then switch to attacking attacker_id.
                false
            }
            Some(_) => true,
            None => {
                assert!(
                    defender.has(PLAYER_ID),
                    "If defender is an NPC being attacked then it should have behaviors"
                );
                false
            }
        };
        if attack {
            self.replace_behavior(defender_loc, Behavior::Attacking(attacker_id, *attacker_loc));
        }
    }

    fn spawn_the_broken(&mut self) {
        for i in 0..7 {
            let loc = self.level.random_loc(&self.rng);
            let existing = &self.level.get(&loc, CHARACTER_ID);
            if existing.is_none() {
                let ch = make::broken(i);
                let (_, terrain) = self.level.get_bottom(&loc);
                if ch.impassible_terrain(terrain).is_none() {
                    self.add_object(&loc, ch);

                    let target = Point::new(46, 35); // they all head for the Vitr lake
                    self.replace_behavior(&loc, Behavior::MovingTo(target));
                }
            }
        }
    }

    fn topic(&self, attacker: Oid, defender: Oid, damage: i32) -> Topic {
        if attacker.0 == 0 {
            if damage > 0 {
                Topic::PlayerDidDamage
            } else {
                Topic::PlayerDidNoDamage
            }
        } else if defender.0 == 0 {
            if damage > 0 {
                Topic::PlayerIsDamaged
            } else {
                Topic::PlayerIsNotDamaged
            }
        } else {
            if damage > 0 {
                Topic::NpcIsDamaged
            } else {
                Topic::NpcIsNotDamaged
            }
        }
    }
}
