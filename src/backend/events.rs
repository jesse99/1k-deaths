use super::*;
use rand_distr::StandardNormal;

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
            self.do_post(event, replay);
        }

        OldPoV::update(self);
        PoV::refresh(self);
        self.posting = false;
        self.invariant();
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

    pub fn handle_scheduled_action(&mut self, oid: Oid, saction: ScheduledAction) {
        let events = match saction {
            ScheduledAction::DamageWall(obj_loc, obj_oid) => self.damage_wall_events(obj_loc, obj_oid),
            ScheduledAction::FightRhulad(char_loc, ch) => self.fight_rhulad_events(char_loc, ch),
            ScheduledAction::FloodDeep(loc) => self.flood_deep_events(loc),
            ScheduledAction::FloodShallow(loc) => self.flood_shallow_events(loc),
            ScheduledAction::Move(old, new) => self.move_events(oid, old, new),
            ScheduledAction::OpenDoor(ch_loc, obj_loc, obj_oid) => self.open_door_events(ch_loc, oid, obj_loc, obj_oid),
            ScheduledAction::PickUp(obj_loc, obj_oid) => self.pickup_events(oid, obj_loc, obj_oid),
            ScheduledAction::ShoveDoorman(old_loc, ch, new_loc) => self.shove_doorman_events(oid, old_loc, ch, new_loc),
        };
        self.post(events, false);
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

            // All scheduled actions do is generate events, but if we're replaying we
            // already have the generated events in the stream. TODO: though because we
            // ignore the scheduled actions the scheduler time won't be updated. Perhaps
            // we should have some sort of replay_push that just updates time.
            if replay {
                if let Event::ScheduledAction(_, _) = event {
                    continue;
                }
                if let Event::ForceAction(_, _) = event {
                    continue;
                }
            }

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
                Event::Action(Action::Move(oid, _, to)) if oid.0 == 0 => Some(to),
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
            Event::Action(action) => match action {
                Action::AddObject(loc, obj) => self.do_add_object(loc, obj),
                Action::DestroyObject(loc, oid) => self.do_destroy_object(loc, oid),
                Action::Move(oid, old, new) => self.do_move(oid, old, new),
                Action::PickUp(ch_id, loc, obj_id) => self.do_pickup(ch_id, loc, obj_id),
                Action::ReplaceObject(loc, old_oid, new_obj) => self.do_replace_object(loc, old_oid, new_obj),
            },
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
            Event::ScheduledAction(oid, saction) => {
                let delay = self.action_delay(saction);
                self.scheduler.push(oid, saction, delay, &self.rng);
            }
            Event::ForceAction(oid, saction) => {
                let delay = self.action_delay(saction);
                self.scheduler.force_push(oid, saction, delay, &self.rng);
            }
            Event::StateChanged(state) => {
                self.state = state;
            }
        }
    }

    fn action_delay(&self, saction: ScheduledAction) -> Time {
        match saction {
            ScheduledAction::DamageWall(_, _) => time::secs(20),
            ScheduledAction::FightRhulad(_, _) => time::secs(30),
            ScheduledAction::Move(old, new) if old.distance2(&new) == 1 => time::secs(4),
            ScheduledAction::FloodDeep(_) => self.flood_delay(),
            ScheduledAction::FloodShallow(_) => self.flood_delay(),
            ScheduledAction::Move(_, _) => time::secs(6), // TODO: should be 5.6
            ScheduledAction::OpenDoor(_, _, _) => time::secs(20),
            ScheduledAction::PickUp(_, _) => time::secs(5),
            ScheduledAction::ShoveDoorman(_, _, _) => time::secs(8),
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

// Scheduled Actions
impl Game {
    fn damage_wall(&self, loc: &Point, scaled_damage: i32) -> Vec<Event> {
        assert!(scaled_damage > 0);
        let (oid, obj) = self.get(loc, WALL_ID).unwrap();
        let durability: Durability = obj.value(DURABILITY_ID).unwrap();
        let damage = durability.max / scaled_damage;

        if damage < durability.current {
            let mesg = Message::new(
                Topic::Normal,
                "You chip away at the wall with your pick-axe.", // TODO: probably should have slightly differet text for wooden walls (if we ever add them)
            );

            let mut obj = obj.clone();
            obj.replace(Tag::Durability(Durability {
                current: durability.current - damage,
                max: durability.max,
            }));
            let action = Action::ReplaceObject(*loc, oid, obj);
            vec![Event::AddMessage(mesg), Event::Action(action)]
        } else {
            let mesg = Message::new(Topic::Important, "You destroy the wall!");
            let action = Action::DestroyObject(*loc, oid);
            vec![Event::AddMessage(mesg), Event::Action(action)]
        }
    }

    fn damage_wall_events(&self, obj_loc: Point, _obj_oid: Oid) -> Vec<Event> {
        let obj = self.get(&obj_loc, WALL_ID).unwrap().1;
        let material: Option<Material> = obj.value(MATERIAL_ID);
        match material {
            Some(Material::Stone) => self.damage_wall(&obj_loc, 6),
            _ => panic!("Should only be called for walls that can be damaged"),
        }
    }

    fn fight_rhulad_events(&self, char_loc: Point, _chr: Oid) -> Vec<Event> {
        let mesg = Message::new(Topic::Important, "After an epic battle you kill the Emperor!");
        let oid = self.get(&char_loc, CHARACTER_ID).unwrap().0;
        let action1 = Action::DestroyObject(char_loc, oid);
        let action2 = Action::AddObject(char_loc, super::make::emp_sword());
        vec![
            Event::AddMessage(mesg),
            Event::Action(action1),
            Event::Action(action2),
            Event::StateChanged(State::KilledRhulad),
        ]
    }

    fn find_neighbor<F>(&self, loc: &Point, predicate: F) -> Option<Point>
    where
        F: Fn(&Point) -> bool,
    {
        let mut deltas = vec![(-1, -1), (-1, 1), (-1, 0), (1, -1), (1, 1), (1, 0), (0, -1), (0, 1)];
        deltas.shuffle(&mut *self.rng());
        for delta in deltas {
            let new_loc = Point::new(loc.x + delta.0, loc.y + delta.1);
            if predicate(&new_loc) {
                return Some(new_loc);
            }
        }
        None
    }

    fn flood_deep_events(&self, loc: Point) -> Vec<Event> {
        if let Some(new_loc) = self.find_neighbor(&loc, |candidate| {
            self.get(candidate, GROUND_ID).is_some() || self.get(candidate, SHALLOW_WATER_ID).is_some()
        }) {
            let bad_oid = self.get(&new_loc, TERRAIN_ID).unwrap().0;
            let action = Action::ReplaceObject(new_loc, bad_oid, super::make::deep_water());
            let event1 = Event::Action(action);

            let old_oid = self.get(&loc, TERRAIN_ID).unwrap().0;
            let saction = ScheduledAction::FloodDeep(loc);
            let event2 = Event::ScheduledAction(old_oid, saction);

            if new_loc == self.player {
                if let Some(newer_loc) = self.find_neighbor(&self.player, |candidate| {
                    self.get(candidate, GROUND_ID).is_some()
                        || self.get(candidate, SHALLOW_WATER_ID).is_some()
                        || self.get(candidate, OPEN_DOOR_ID).is_some()
                }) {
                    let saction = ScheduledAction::Move(self.player, newer_loc);
                    let event3 = Event::ForceAction(Oid(0), saction);

                    let mesg = Message {
                        topic: Topic::Normal,
                        text: "You step away from the rising water.".to_string(),
                    };
                    let event4 = Event::AddMessage(mesg);
                    vec![event1, event2, event3, event4]
                } else {
                    let mesg = Message {
                        topic: Topic::Important,
                        text: "You drown!".to_string(),
                    };
                    let event3 = Event::AddMessage(mesg);
                    let event4 = Event::StateChanged(State::LostGame);
                    vec![event1, event2, event3, event4]
                }
            } else {
                vec![event1, event2]
            }
        } else {
            Vec::new()
        }
    }

    fn flood_shallow_events(&self, loc: Point) -> Vec<Event> {
        if let Some(new_loc) = self.find_neighbor(&loc, |candidate| {
            self.get(candidate, GROUND_ID).is_some() || self.get(candidate, OPEN_DOOR_ID).is_some()
        }) {
            let bad_oid = self.get(&new_loc, TERRAIN_ID).unwrap().0;
            let action = Action::ReplaceObject(new_loc, bad_oid, super::make::shallow_water());
            let event1 = Event::Action(action);

            let old_oid = self.get(&loc, TERRAIN_ID).unwrap().0;
            let saction = ScheduledAction::FloodShallow(loc);
            let event2 = Event::ScheduledAction(old_oid, saction);
            vec![event1, event2]
        } else {
            Vec::new()
        }
    }

    fn move_events(&self, oid: Oid, old: Point, new: Point) -> Vec<Event> {
        let action = Action::Move(oid, old, new);
        vec![Event::Action(action)]
    }

    fn open_door_events(&self, ch_loc: Point, oid: Oid, obj_loc: Point, obj_oid: Oid) -> Vec<Event> {
        let action1 = Action::ReplaceObject(obj_loc, obj_oid, make::open_door());
        let action2 = Action::Move(oid, ch_loc, obj_loc);
        vec![Event::Action(action1), Event::Action(action2)]
    }

    fn pickup_events(&self, oid: Oid, obj_loc: Point, obj_oid: Oid) -> Vec<Event> {
        let action = Action::PickUp(oid, obj_loc, obj_oid);
        vec![Event::Action(action)]
    }

    fn shove_doorman_events(
        &self,
        player_oid: Oid,
        old_doorman_loc: Point,
        doorman_oid: Oid,
        new_doorman_loc: Point,
    ) -> Vec<Event> {
        let action1 = Action::Move(doorman_oid, old_doorman_loc, new_doorman_loc);
        let action2 = Action::Move(player_oid, self.player, old_doorman_loc);
        vec![Event::Action(action1), Event::Action(action2)]
    }
}

// Actions
impl Game {
    fn flood_delay(&self) -> Time {
        let rng = &mut *self.rng();
        let t: f64 = rng.sample(StandardNormal); // most are in -2..2
        let t = t / 2.0; // most are in -1..1
        let t = t * 100.0; // most are in -100..100
        let t = t + 200.0; // most are in 100..300
        let t = f64::max(t, 1.0); // times have to be positive
        time::secs(t as i64)
    }

    fn do_add_object(&mut self, loc: Point, obj: Object) {
        if obj.has(PLAYER_ID) {
            self.player = loc;
            self.create_player(&loc, obj);
        } else {
            let is_shallow = obj.has(SHALLOW_WATER_ID);
            let is_deep = obj.has(DEEP_WATER_ID);
            let oid = self.create_object(obj);
            let oids = self.cells.entry(loc).or_insert_with(Vec::new);
            oids.push(oid);

            // TODO: probably want to get rid of flooding at some point
            if is_deep {
                let saction = ScheduledAction::FloodDeep(loc);
                self.scheduler.push(oid, saction, self.flood_delay(), &self.rng);
            } else if is_shallow {
                let saction = ScheduledAction::FloodShallow(loc);
                self.scheduler.push(oid, saction, self.flood_delay(), &self.rng);
            }
        }
    }

    fn do_destroy_object(&mut self, loc: Point, old_oid: Oid) {
        let oids = self.cells.get_mut(&loc).unwrap();
        let index = oids.iter().position(|id| *id == old_oid).unwrap();
        let obj = self.objects.get(&old_oid).unwrap();
        if obj.has(TERRAIN_ID) {
            // Terrain cannot be destroyed but has to be mutated into something else.
            let new_obj = if obj.has(WALL_ID) {
                make::rubble()
            } else {
                error!("Need to better handle destroying Tid {obj}"); // Doors, trees, etc
                make::dirt()
            };
            let new_oid = Oid(self.next_id);
            self.next_id += 1;
            self.objects.insert(new_oid, new_obj);
            oids[index] = new_oid;

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

    fn do_move(&mut self, oid: Oid, old: Point, new: Point) {
        assert!(!self.constructing); // make sure this is reset once things start happening

        let oids = self.cells.get_mut(&old).unwrap();
        let index = oids.iter().position(|id| *id == oid).unwrap();
        oids.remove(index);
        let cell = self.cells.entry(new).or_insert_with(Vec::new);
        cell.push(oid);

        if oid.0 == 0 {
            self.player = new;
        }
    }

    fn do_pickup(&mut self, _ch_id: Oid, loc: Point, obj_id: Oid) {
        let obj = self.objects.get(&obj_id).unwrap();
        let name: String = obj.value(NAME_ID).unwrap();
        let mesg = Message {
            topic: Topic::Normal,
            text: format!("You pick up the {name}."),
        };
        self.messages.push(mesg);
        {
            let oids = self.cells.get_mut(&loc).unwrap();
            let index = oids.iter().position(|id| *id == obj_id).unwrap();
            oids.remove(index);
        }
        self.mutate(&loc, INVENTORY_ID, |obj| {
            let inv = obj.as_mut_ref(INVENTORY_ID).unwrap();
            inv.push(obj_id);
        });
    }

    fn do_replace_object(&mut self, loc: Point, old_oid: Oid, new_obj: Object) {
        let is_shallow = new_obj.has(SHALLOW_WATER_ID);
        let is_deep = new_obj.has(DEEP_WATER_ID);

        let new_oid = self.create_object(new_obj);
        let oids = self.cells.get_mut(&loc).unwrap();
        let index = oids.iter().position(|id| *id == old_oid).unwrap();
        oids[index] = new_oid;
        self.objects.remove(&old_oid);

        // TODO: probably want to get rid of flooding at some point
        if is_deep {
            let saction = ScheduledAction::FloodDeep(loc);
            self.scheduler.push(new_oid, saction, self.flood_delay(), &self.rng);
        } else if is_shallow {
            let saction = ScheduledAction::FloodShallow(loc);
            self.scheduler.push(new_oid, saction, self.flood_delay(), &self.rng);
        }
    }
}
