use super::*;

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
            trace!("posting {event}");
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
    fn mutate<F>(&mut self, loc: &Point, tag: TagId, callback: F)
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
        // It'd be slicker to use a different Game type when replaying. This would prevent
        // us, at compile time, from touching fields like stream or rng. In practice however
        // this isn't much of an issue because the bulk of the code is already prevented
        // from doing bad things by the Game1, Game2, etc structs.
        if !replay {
            self.stream.push(event.clone());

            if self.stream.len() >= MAX_QUEUED_EVENTS {
                self.append_stream();
            }
        }

        let mut events = vec![event];
        while !events.is_empty() {
            let event = events.remove(0); // icky remove from front but the vector shouldn't be very large...
            let moved_to = match event {
                Event::PlayerMoved(new_loc) => Some(new_loc),
                _ => None,
            };

            if let Some(event) = self.do_event(event) {
                // This is the type state pattern: as events are posted new state
                // objects are updated and upcoming state objects can safely reference
                // them. To enforce this at compile time Game1, Game2, etc objects
                // are used to provide views into Game.
                let game1 = super::details::Game1 {
                    objects: &self.objects,
                    cells: &self.cells,
                };
                self.pov.posted(&game1, &event);

                let game2 = super::details::Game2 {
                    objects: &self.objects,
                    cells: &self.cells,
                    pov: &self.pov,
                };
                self.old_pov.posted(&game2, &event);
            }

            if let Some(new_loc) = moved_to {
                // Icky recursion: when we do stuff like move into a square
                // we want to immediately take various actions, like printing
                // "You splash through the water".
                self.interact_post_move(&new_loc, &mut events);
            }
        }
    }

    fn do_event(&mut self, event: Event) -> Option<Event> {
        match event {
            Event::AddMessage(message) => {
                if let Topic::Error = message.topic {
                    // TODO: do we really want to do this?
                    error!("Logged error '{}'", message.text);
                }
                self.messages.push(message);
                while self.messages.len() > MAX_MESSAGES {
                    self.messages.remove(0); // TODO: this is an O(N) operation for Vec, may want to switch to circular_queue
                }
                None
            }
            Event::AddObject(loc, obj) => {
                if obj.has(PLAYER_ID) {
                    self.player = loc;
                    self.create_player(&loc, obj);
                } else {
                    let oid = self.create_object(obj);
                    let oids = self.cells.entry(loc).or_insert_with(Vec::new);
                    oids.push(oid);
                };
                None
            }
            Event::AddToInventory(loc) => {
                let oid = {
                    let (oid, obj) = self.get(&loc, PORTABLE_ID).unwrap(); // TODO: this only picks up the topmost item
                    let name: String = obj.value(NAME_ID).unwrap();
                    let mesg = Message {
                        topic: Topic::Normal,
                        text: format!("You pick up the {name}."),
                    };
                    self.messages.push(mesg);
                    oid
                };

                {
                    let oids = self.cells.get_mut(&loc).unwrap();
                    let index = oids.iter().position(|id| *id == oid).unwrap();
                    oids.remove(index);
                }

                let loc = self.player;
                self.mutate(&loc, INVENTORY_ID, |obj| {
                    let inv = obj.as_mut_ref(INVENTORY_ID).unwrap();
                    inv.push(oid);
                });

                Some(event)
            }
            Event::BeginConstructLevel => {
                let oid = ObjId(0);
                let player = self.objects.remove(&oid);
                self.objects = FnvHashMap::default();
                self.cells = FnvHashMap::default();
                if let Some(player) = player {
                    self.objects.insert(oid, player);
                }
                self.constructing = true;
                Some(event)
            }
            Event::DestroyObjectId(loc, oid) => {
                self.destroy_object(&loc, oid);
                None
            }
            Event::EndConstructLevel => {
                self.constructing = false;
                Some(event)
            }
            Event::NewGame => {
                // TODO: do we want this event?
                Some(event)
            }
            Event::NPCMoved(old, new) => {
                let (oid, _) = self.get(&old, CHARACTER_ID).unwrap();

                let oids = self.cells.get_mut(&old).unwrap();
                let index = oids.iter().position(|id| *id == oid).unwrap();
                oids.remove(index);

                let cell = self.cells.entry(new).or_insert_with(Vec::new);
                cell.push(oid);
                Some(event)
            }
            Event::PlayerMoved(loc) => {
                assert!(!self.constructing); // make sure this is reset once things start happening
                let oid = ObjId(0);
                let oids = self.cells.get_mut(&self.player).unwrap();
                let index = oids.iter().position(|id| *id == oid).unwrap();
                oids.remove(index);

                self.player = loc;
                let cell = self.cells.entry(self.player).or_insert_with(Vec::new);
                cell.push(oid);

                Some(event)
            }
            Event::ReplaceObject(loc, old_oid, obj) => {
                let new_oid = self.create_object(obj);

                let oids = self.cells.get_mut(&loc).unwrap();
                let index = oids.iter().position(|id| *id == old_oid).unwrap();
                oids[index] = new_oid;

                self.objects.remove(&old_oid);
                None
            }
            Event::StateChanged(state) => {
                self.state = state;
                Some(event)
            }
        }
    }

    fn create_player(&mut self, loc: &Point, obj: Object) -> ObjId {
        let oid = ObjId(0);
        self.objects.insert(oid, obj);

        let oids = self.cells.entry(*loc).or_insert_with(Vec::new);
        oids.push(oid);
        oid
    }

    // This does not update cells (the object may go elsewhere).
    fn create_object(&mut self, obj: Object) -> ObjId {
        let oid = ObjId(self.next_id);
        self.next_id += 1;
        self.objects.insert(oid, obj);
        oid
    }

    fn destroy_object(&mut self, loc: &Point, old_oid: ObjId) {
        let oids = self.cells.get_mut(loc).unwrap();
        let index = oids.iter().position(|id| *id == old_oid).unwrap();
        let obj = self.objects.get(&old_oid).unwrap();
        if obj.has(TERRAIN_ID) {
            // Terrain cannot be destroyed but has to be mutated into something else.
            let new_obj = if obj.has(WALL_ID) {
                make::rubble()
            } else {
                error!("Need to better handle destroying TagId {obj}"); // Doors, trees, etc
                make::dirt()
            };
            let new_oid = ObjId(self.next_id);
            self.next_id += 1;
            self.objects.insert(new_oid, new_obj);
            oids[index] = new_oid;

            // The player may now be able to see through this cell so we need to ensure
            // that cells around it exist now. TODO: probably should have a LOS changed
            // check.
            self.ensure_neighbors(loc);
        } else {
            // If it's just a normal object or character we can just nuke the object.
            oids.remove(index);
        }
        self.objects.remove(&old_oid);
    }

    fn ensure_neighbors(&mut self, loc: &Point) {
        let deltas = vec![(-1, -1), (-1, 1), (-1, 0), (1, -1), (1, 1), (1, 0), (0, -1), (0, 1)];
        for delta in deltas {
            let new_loc = Point::new(loc.x + delta.0, loc.y + delta.1);
            let _ = self.cells.entry(new_loc).or_insert_with(|| {
                let oid = ObjId(self.next_id);
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
