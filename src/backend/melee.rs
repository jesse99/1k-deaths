use super::*;

const MAX_STAT: i32 = 30; // this is a soft limit: stats can go higher than this but with diminishing (or no) returns

impl Game {
    pub fn melee_delay(&self, attacker_loc: &Point) -> Time {
        let attacker_id = self.level.get(attacker_loc, CHARACTER_ID).unwrap().0;
        let attacker = self.level.obj(attacker_id).0;
        if let Some(weapon) = self.find_main_hand(attacker) {
            // TODO: extra delay if off hand?
            weapon.delay_value().unwrap()
        } else {
            attacker.delay_value().unwrap()
        }
    }

    pub fn off_hand_prob(&self) -> f64 {
        0.25
    }

    pub fn do_melee_attack(&mut self, attacker_loc: &Point, defender_loc: &Point) {
        let attacker_id = self.level.get(attacker_loc, CHARACTER_ID).unwrap().0;
        let defender_id = self.level.get_mut(defender_loc, CHARACTER_ID).unwrap().0;
        debug!("{attacker_id} is meleeing {defender_id}");
        self.react_to_attack(attacker_loc, attacker_id, defender_loc);

        let mut damage = 0;
        let mut text = String::new();

        let weapon = {
            let attacker = self.level.obj(attacker_id).0;
            attacker.equipped_value().map(|e| e[Slot::MainHand]).flatten()
        };
        let (dam, msg) = self.do_attack(attacker_id, defender_id, defender_loc, weapon);
        damage += dam;
        text += &msg;

        // Weapon delay is ignored for the off hand so it's best to use the highest damage
        // weapon you can find.
        let weapon = {
            let attacker = self.level.obj(attacker_id).0;
            attacker.equipped_value().map(|e| e[Slot::OffHand]).flatten()
        };
        if weapon.is_some() {
            let off_hand = {
                let rng = &mut *self.rng();
                rng.gen_bool(self.off_hand_prob())
            };
            if off_hand {
                // TODO: probability should depend on skill (very low at no skill)
                // TODO: may want to allow an off hand unarmed attack for some races (or for a skill?)
                let (dam, msg) = self.do_attack(attacker_id, defender_id, defender_loc, weapon);
                damage += dam;
                text += &format!(" {msg}");
            }
        }

        let topic = self.topic(attacker_id, defender_id, damage);
        let mesg = Message::new(topic, &text);
        self.messages.push(mesg);
    }
}

impl Game {
    fn do_attack(
        &mut self,
        attacker_id: Oid,
        defender_id: Oid,
        defender_loc: &Point,
        weapon: Option<Oid>,
    ) -> (i32, String) {
        // It'd be more efficient to use Objects here but the borrow checker whines a lot.
        let attacker_name = self.attacker_name(attacker_id);
        let defender_name = self.defender_name(defender_id);
        if let Some((damage, crit)) = self.do_strike(attacker_id, defender_id, weapon) {
            let (new_hps, max_hps) = self.hps(defender_id, damage);
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

            (damage, msg)
        } else {
            let msg = format!("{attacker_name} missed {defender_name}.");
            (0, msg)
        }
    }

    fn do_strike(&mut self, attacker_id: Oid, defender_id: Oid, weapon: Option<Oid>) -> Option<(i32, bool)> {
        let (damage, crit) = self.base_damage(attacker_id, weapon);
        if self.hit_defender(attacker_id, defender_id) {
            let damage = self.mitigate_damage(attacker_id, defender_id, damage);
            Some((damage, crit))
        } else {
            None
        }
    }

    fn attacker_name(&self, attacker_id: Oid) -> String {
        if attacker_id.0 == 0 {
            "You".to_string()
        } else {
            let attacker = self.level.obj(attacker_id).0;
            let name: &'static str = attacker.name_value().unwrap();
            format!("{name}")
        }
    }

    pub fn base_damage(&self, attacker_id: Oid, weapon: Option<Oid>) -> (i32, bool) {
        let attacker = self.level.obj(attacker_id).0;
        let (damage, min_str) = if let Some(weapon) = weapon.map(|w| self.level.obj(w).0) {
            (weapon.damage_value().unwrap(), weapon.strength_value())
        } else {
            (
                attacker
                    .damage_value()
                    .expect(&format!("{attacker} should have an (unarmed) damage tag")),
                Some(MAX_STAT / 6), // strength helps quite a bit with unarmed
            )
        };

        // Scales base damage according to how much stronger the character is then the
        // min weapon strength. Because we cap this at 2x stacking strength will not further
        // help light weapons. Also there can be significant penalties for using weapons
        // that are too heavy for a character. TODO: need some sort of indication for these
        // penalties, maybe status effect warning.
        let mut damage = if let Some(min_str) = min_str {
            let cur_str = attacker.strength_value().unwrap();
            let scaling = f64::max((cur_str as f64) / (min_str as f64), 2.0);
            ((damage as f64) * scaling) as i32
        } else {
            damage
        };

        // Crit chance is based on the weapon scaled by how much more dexterity the
        // character has then the min dexterity required by the weapon to begin criting.
        // TODO: reduce chance of crits for off hand?
        let p = self.crit_prob(attacker_id, weapon);
        let crit = self.rng.borrow_mut().gen_bool(p);
        if crit {
            damage *= 2;
        }
        (super::rand_normal32(damage, 20, &self.rng), crit)
    }

    pub fn crit_prob(&self, attacker_id: Oid, weapon: Option<Oid>) -> f64 {
        let attacker = self.level.obj(attacker_id).0;
        let (min_dex, crit_percent) = if let Some(weapon) = weapon.map(|w| self.level.obj(w).0) {
            (weapon.dexterity_value(), weapon.crit_value().unwrap_or(0))
        } else {
            (
                Some(MAX_STAT / 2), // hard to crit more with unarmed
                2,                  // and a low chance of critting at all
            )
        };

        if let Some(min_dex) = min_dex {
            let dex = attacker.dexterity_value().unwrap() - min_dex;
            let scaling = linear_scale(dex, 0, MAX_STAT, 1.0, 4.0);
            scaling * (crit_percent as f64) / 100.0
        } else {
            0.0
        }
    }

    fn defender_name(&self, defender_id: Oid) -> String {
        if defender_id.0 == 0 {
            "you".to_string()
        } else {
            let defender = self.level.obj(defender_id).0;
            format!("{defender}")
        }
    }

    pub fn find_main_hand(&self, attacker: &Object) -> Option<&Object> {
        let equipped = attacker.equipped_value()?;
        if let Some(oid) = equipped[Slot::MainHand] {
            return Some(self.level.obj(oid).0);
        }
        None
    }

    pub fn find_off_hand(&self, attacker: &Object) -> Option<&Object> {
        let equipped = attacker.equipped_value()?;
        if let Some(oid) = equipped[Slot::OffHand] {
            return Some(self.level.obj(oid).0);
        }
        None
    }

    fn hps(&self, defender_id: Oid, damage: i32) -> (i32, i32) {
        let defender = self.level.obj(defender_id).0;
        let durability = defender.durability_value().unwrap();
        (durability.current - damage, durability.max)
    }

    fn hit_defender(&self, attacker_id: Oid, defender_id: Oid) -> bool {
        let p = self.hit_prob(attacker_id, defender_id);
        let rng = &mut *self.rng();
        rng.gen_bool(p)
    }

    // TODO: use dexterity/evasion
    pub fn hit_prob(&self, attacker_id: Oid, defender_id: Oid) -> f64 {
        let attacker = self.level.obj(attacker_id).0;
        let defender = self.level.obj(defender_id).0;

        let adex = attacker.dexterity_value().unwrap(); // TODO: this should be adjusted by heavy gear
        let ddex = defender.dexterity_value().unwrap();
        let max_delta = (2 * MAX_STAT) / 3;
        linear_scale(adex - ddex, -max_delta, max_delta, 0.1, 1.0)
    }

    // TODO: use skill
    // TODO: there should be penalties if the character isn't stromg enough to wear the
    // armor well. Maybe sliding penalties to movement and weapon speed. Description
    // and status effect should have text for that. Maybe something for magic too? Or maybe
    // can prevent mage tanks using skills (can't be both great at armor and casting).
    fn mitigate_damage(&self, _attacker_id: Oid, defender_id: Oid, damage: i32) -> i32 {
        let defender = self.level.obj(defender_id).0;
        if let Some(equipped) = defender.equipped_value() {
            let mut mitigation = 0;
            for item in equipped.values() {
                if let Some(oid) = item {
                    let obj = self.level.obj(*oid).0;
                    if let Some(m) = obj.mitigation_value() {
                        mitigation += m;
                    }
                }
            }
            let scaling = 1.0 - (mitigation as f64) / 100.0;
            let scaling = scaling.max(0.0);
            (scaling * (damage as f64)) as i32
        } else {
            damage
        }
    }

    fn npc_died(&mut self, defender_loc: &Point, defender_id: Oid) {
        let defender = self.level.obj(defender_id).0;
        let is_rhulad = defender.has(RHULAD_ID);

        self.destroy_object(defender_loc, defender_id);

        if is_rhulad {
            self.add_object(defender_loc, new_obj(ObjectName::EmperorSword)); // TODO: should drop inv items
            self.state = State::KilledRhulad;

            let msg = "The Crippled God whispers, 'You shall pay for this mortal'.";
            let mesg = Message::new(Topic::Important, &msg);
            self.messages.push(mesg);
            self.spawn_the_broken();
        }
    }

    fn react_to_attack(&mut self, attacker_loc: &Point, attacker_id: Oid, defender_loc: &Point) {
        let defender = self.level.get_mut(defender_loc, CHARACTER_ID).unwrap().1;
        let attack = match defender.behavior_value() {
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
        let mut bindex = 0;
        let broken = vec![
            ObjectName::BerokeSoftVoice,
            ObjectName::HaladRackBearer,
            ObjectName::ImrothTheCruel,
            ObjectName::KahlbTheSilentHunter,
            ObjectName::SiballeTheUnfound,
            ObjectName::ThenikTheShattered,
            ObjectName::UrugalTheWoven,
        ];
        for _ in 0..21 {
            let loc = self.level.random_loc(&self.rng);
            let existing = &self.level.get(&loc, CHARACTER_ID);
            if existing.is_none() {
                let ch = new_obj(broken[bindex]);
                let (_, terrain) = self.level.get_bottom(&loc);
                if ch.impassible_terrain(terrain).is_none() {
                    self.add_object(&loc, ch);
                    bindex += 1;
                    if bindex == 7 {
                        break;
                    }

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

fn linear_scale(x: i32, min_x: i32, max_x: i32, min_p: f64, max_p: f64) -> f64 {
    assert!(min_x < max_x);
    assert!(min_p < max_p);

    let x = if x <= min_x {
        0.0
    } else if x >= max_x {
        1.0
    } else {
        ((x - min_x) as f64) / ((max_x - min_x) as f64)
    };

    let p = min_p + x * (max_p - min_p);
    debug_assert!(p >= min_p);
    debug_assert!(p <= max_p);

    p
}
