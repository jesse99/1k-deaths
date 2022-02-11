use super::*;
// use rand_distr::StandardNormal;

const MAX_QUEUED_EVENTS: usize = 1_000; // TODO: make this even larger?

// In order to ensure that games are replayable mutation should only happen as a direct
// result of an event. To ensure that this is true this is the only public mutable Game
// method.
impl Game {
    pub fn post(&mut self, events: Vec<Event>, replay: bool) {
        // This is bad because it messes up replay: if it is allowed then an event will
        // post a new event X both of which will be persisted. Then on replay the event
        // will post X but X will have been also saved so X is done twice.
        assert!(!self.posting, "Cannot post an event in response to an event");

        self.posting = true;
        for event in events {
            // trace!("posting {event}");
            if let Event::Action(Oid(0), action) = event {
                let duration = self.duration(action);
                if duration > Time::zero() {
                    self.scheduler.acted(Oid(0), duration, Time::zero(), &self.rng);
                }
            }
            self.do_post(event, replay);
        }

        OldPoV::update(self);
        PoV::refresh(self);
        self.posting = false;
        {
            #[cfg(debug_assertions)]
            self.invariant();
        }
    }
}

// All the mutable Game methods should be here and they should all be private so that we
// have control of when they are called.
impl Game {
    // get_mut would be nicer but couldn't figure out how to write that.
    fn mutate<F>(&mut self, loc: &Point, tag: Tid, callback: F)
    where
        F: Fn(&mut Object),
    {
        let oids = self
            .cells
            .get(loc)
            .expect("get methods should only be called for valid locations");
        for oid in oids {
            let obj = self
                .objects
                .get_mut(oid)
                .expect("All objects in the level should still exist");
            if obj.has(tag) {
                callback(obj);
                return;
            }
        }
        panic!("Didn't find {tag} at {loc}");
    }

    fn append_stream(&mut self) {
        if let Some(se) = &mut self.file {
            if let Err(err) = persistence::append_game(se, &self.stream) {
                self.messages
                    .push(Message::new(Topic::Error, &format!("Couldn't save game: {err}")));
            }
        }
        // If we can't save there's not much we can do other than clear. (Still worthwhile
        // appending onto the stream because we may want a wizard command to show the last
        // few events).
        self.stream.clear();
    }

    fn do_post(&mut self, event: Event, replay: bool) {
        if !replay {
            self.stream.push(event.clone());

            if self.stream.len() >= MAX_QUEUED_EVENTS {
                self.append_stream();
            }
        }

        let mut events = vec![event];
        while !events.is_empty() {
            let event = events.remove(0); // icky remove from front but the vector shouldn't be very large...
            trace!("processing {event}");

            // This is the type state pattern: as events are posted new state
            // objects are updated and upcoming state objects can safely reference
            // them. To enforce this at compile time Game1, Game2, etc objects
            // are used to provide views into Game.
            let game1 = super::details::Game1 {
                objects: &self.objects,
                cells: &self.cells,
            };
            self.pov.posting(&game1, &event);

            let game2 = super::details::Game2 {
                objects: &self.objects,
                cells: &self.cells,
                pov: &self.pov,
            };
            self.old_pov.posting(&game2, &event);

            let moved_to = match event {
                Event::Action(oid, Action::Move(_, to)) if oid.0 == 0 => Some(to),
                _ => None,
            };
            self.do_event(event);
            if let Some(new_loc) = moved_to {
                // When we do stuff like move into a square we want to immediately take
                // various actions, like printing "You splash through the water".
                self.interact_post_move(&new_loc, &mut events);
            }
        }
    }

    // Only returns a new event if it's something that could affect PoV.
    fn do_event(&mut self, event: Event) {
        match event {
            Event::Action(oid, action) => match action {
                Action::Dig(obj_loc, obj_oid, damage) => self.do_dig(oid, obj_loc, obj_oid, damage),
                Action::FightRhulad(char_loc, ch) => self.do_fight_rhulad(oid, char_loc, ch),
                Action::FloodDeep(water_loc) => self.do_flood_deep(oid, water_loc),
                Action::FloodShallow(water_loc) => self.do_flood_shallow(oid, water_loc),
                Action::Move(old_loc, new_loc) => self.do_move(oid, old_loc, new_loc),
                Action::OpenDoor(ch_loc, obj_loc, obj_oid) => self.do_open_door(oid, ch_loc, obj_loc, obj_oid),
                Action::PickUp(obj_loc, obj_oid) => self.do_pick_up(oid, obj_loc, obj_oid),
                Action::ShoveDoorman(old_loc, ch, new_loc) => self.do_shove_doorman(oid, old_loc, ch, new_loc),
            },
            Event::AddObject(loc, obj) => self.do_add_object(loc, obj),
            Event::AddMessage(message) => {
                if let Topic::Error = message.topic {
                    // TODO: do we really want to do this?
                    error!("Logged error '{}'", message.text);
                }
                self.messages.push(message);
                while self.messages.len() > MAX_MESSAGES {
                    self.messages.remove(0); // TODO: this is an O(N) operation for Vec, may want to switch to circular_queue
                }
            }
            Event::BeginConstructLevel => {
                let oid = Oid(0);
                let player = self.objects.remove(&oid);
                self.objects = FnvHashMap::default();
                self.cells = FnvHashMap::default();
                self.scheduler = Scheduler::new();
                if let Some(player) = player {
                    self.objects.insert(oid, player);
                }
                self.constructing = true;
            }
            Event::EndConstructLevel => {
                self.constructing = false;
            }
            Event::NewGame => {
                // TODO: do we want this event?
            }
            Event::StateChanged(state) => {
                self.state = state;
            }
        }
    }

    fn create_player(&mut self, loc: &Point, obj: Object) -> Oid {
        let oid = Oid(0);
        self.objects.insert(oid, obj);

        let oids = self.cells.entry(*loc).or_insert_with(Vec::new);
        oids.push(oid);
        oid
    }

    // This does not update cells (the object may go elsewhere).
    fn create_object(&mut self, obj: Object) -> Oid {
        let oid = Oid(self.next_id);
        self.next_id += 1;
        self.objects.insert(oid, obj);
        oid
    }

    fn ensure_neighbors(&mut self, loc: &Point) {
        let deltas = vec![(-1, -1), (-1, 1), (-1, 0), (1, -1), (1, 1), (1, 0), (0, -1), (0, 1)];
        for delta in deltas {
            let new_loc = Point::new(loc.x + delta.0, loc.y + delta.1);
            let _ = self.cells.entry(new_loc).or_insert_with(|| {
                let oid = Oid(self.next_id);
                self.next_id += 1;
                self.objects.insert(oid, self.default.clone());
                vec![oid]
            });
        }
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        self.append_stream();
    }
}

// Actions
impl Game {
    fn do_dig(&mut self, _oid: Oid, obj_loc: Point, obj_oid: Oid, damage: i32) {
        assert!(damage > 0);

        let (damage, durability) = {
            let obj = self.get(&obj_loc, WALL_ID).unwrap().1;
            let durability: Durability = obj.value(DURABILITY_ID).unwrap();
            (durability.max / damage, durability)
        };

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
            self.do_replace_object(obj_loc, obj_oid, obj);
        } else {
            let mesg = Message::new(Topic::Important, "You destroy the wall!");
            self.messages.push(mesg);
            self.do_destroy_object(obj_loc, obj_oid);
        }
    }

    fn do_fight_rhulad(&mut self, _oid: Oid, char_loc: Point, ch: Oid) {
        let mesg = Message::new(Topic::Important, "After an epic battle you kill the Emperor!");
        self.messages.push(mesg);

        self.do_destroy_object(char_loc, ch);
        self.do_add_object(char_loc, super::make::emp_sword());
        self.state = State::KilledRhulad;
    }

    fn do_flood_deep(&mut self, oid: Oid, loc: Point) {
        if let Some(new_loc) = self.find_neighbor(&loc, |candidate| {
            self.get(candidate, GROUND_ID).is_some() || self.get(candidate, SHALLOW_WATER_ID).is_some()
        }) {
            let bad_oid = self.get(&new_loc, TERRAIN_ID).unwrap().0;
            self.do_replace_object(new_loc, bad_oid, super::make::deep_water());

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
                    self.do_move(Oid(0), self.player, newer_loc);

                    let action = Action::Move(self.player, newer_loc);
                    let units = self.duration(action);
                    self.scheduler.acted(Oid(0), units, Time::zero(), &self.rng);
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

    fn do_flood_shallow(&mut self, oid: Oid, loc: Point) {
        if let Some(new_loc) = self.find_neighbor(&loc, |candidate| self.get(candidate, GROUND_ID).is_some()) {
            let bad_oid = self.get(&new_loc, TERRAIN_ID).unwrap().0;
            self.do_replace_object(new_loc, bad_oid, super::make::shallow_water());
        } else {
            // No where left to flood.
            self.scheduler.remove(oid);
        }
    }

    fn do_move(&mut self, oid: Oid, old_loc: Point, new_loc: Point) {
        assert!(!self.constructing); // make sure this is reset once things start happening

        let oids = self.cells.get_mut(&old_loc).unwrap();
        let index = oids
            .iter()
            .position(|id| *id == oid)
            .expect(&format!("expected {oid} at {old_loc}"));
        oids.remove(index);
        let cell = self.cells.entry(new_loc).or_insert_with(Vec::new);
        cell.push(oid);

        if oid.0 == 0 {
            self.player = new_loc;
        }
    }

    fn do_open_door(&mut self, oid: Oid, ch_loc: Point, obj_loc: Point, obj_oid: Oid) {
        self.do_replace_object(obj_loc, obj_oid, super::make::open_door());
        self.do_move(oid, ch_loc, obj_loc);
    }

    fn do_pick_up(&mut self, _oid: Oid, obj_loc: Point, obj_oid: Oid) {
        let obj = self.objects.get(&obj_oid).unwrap();
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

    fn do_shove_doorman(&mut self, oid: Oid, old_loc: Point, ch: Oid, new_loc: Point) {
        self.do_move(ch, old_loc, new_loc);
        self.do_move(oid, self.player, old_loc);
    }

    fn find_neighbor<F>(&self, loc: &Point, predicate: F) -> Option<Point>
    where
        F: Fn(&Point) -> bool,
    {
        let deltas = vec![(-1, -1), (-1, 1), (-1, 0), (1, -1), (1, 1), (1, 0), (0, -1), (0, 1)];
        // deltas.shuffle(&mut *self.rng());
        // for delta in deltas {

        // TODO: events are supposed to encode all the info required to do the action. In
        // particular randomized values (like damage) should be within the event. This is
        // important because when the events are replayed the code that assembles the
        // events is not executed so the RNG stream gets out of sync. Probably what we
        // should do is create a new game view for events that doesn't include the rng.
        let offset = self.next_id as usize;
        for i in 0..deltas.len() {
            let index = (i + offset) % deltas.len();
            let delta = deltas[index];

            let new_loc = Point::new(loc.x + delta.0, loc.y + delta.1);
            if predicate(&new_loc) {
                return Some(new_loc);
            }
        }
        None
    }
}

// Action helpers
impl Game {
    fn do_add_object(&mut self, loc: Point, obj: Object) {
        let scheduled = obj.has(SCHEDULED_ID);
        let oid = if obj.has(PLAYER_ID) {
            self.player = loc;
            self.create_player(&loc, obj)
        } else {
            let oid = self.create_object(obj);
            let oids = self.cells.entry(loc).or_insert_with(Vec::new);
            oids.push(oid);
            oid
        };
        if scheduled {
            self.do_schedule(oid);
        }
    }

    fn do_destroy_object(&mut self, loc: Point, old_oid: Oid) {
        let obj = self.objects.get(&old_oid).unwrap();
        if obj.has(SCHEDULED_ID) {
            self.scheduler.remove(old_oid);
        }

        let oids = self.cells.get_mut(&loc).unwrap();
        let index = oids.iter().position(|id| *id == old_oid).unwrap();
        if obj.has(TERRAIN_ID) {
            // Terrain cannot be destroyed but has to be mutated into something else.
            let new_obj = if obj.has(WALL_ID) {
                make::rubble()
            } else {
                error!("Need to better handle destroying Tid {obj}"); // Doors, trees, etc
                make::dirt()
            };
            let scheduled = new_obj.has(SCHEDULED_ID);

            let new_oid = Oid(self.next_id);
            self.next_id += 1;
            self.objects.insert(new_oid, new_obj);
            oids[index] = new_oid;

            if scheduled {
                self.do_schedule(new_oid);
            }

            // The player may now be able to see through this cell so we need to ensure
            // that cells around it exist now. TODO: probably should have a LOS changed
            // check.
            self.ensure_neighbors(&loc);
        } else {
            // If it's just a normal object or character we can just nuke the object.
            oids.remove(index);
        }
        self.objects.remove(&old_oid);
    }

    fn do_replace_object(&mut self, loc: Point, old_oid: Oid, new_obj: Object) {
        let old_obj = self.objects.get(&old_oid).unwrap();
        if old_obj.has(SCHEDULED_ID) {
            self.scheduler.remove(old_oid);
        }

        let scheduled = new_obj.has(SCHEDULED_ID);
        let new_oid = self.create_object(new_obj);
        let oids = self.cells.get_mut(&loc).unwrap();
        let index = oids.iter().position(|id| *id == old_oid).unwrap();
        oids[index] = new_oid;

        if scheduled {
            self.do_schedule(new_oid);
        }
    }

    fn do_schedule(&mut self, oid: Oid) {
        let obj = self.objects.get(&oid).unwrap();
        let initial = if oid.0 == 0 {
            time::secs(6) // enough for a move
        } else if obj.has(SHALLOW_WATER_ID) || obj.has(DEEP_WATER_ID) {
            Time::zero() - self.flood_delay()
        } else {
            Time::zero()
        };
        self.scheduler.add(oid, initial);
    }
}
