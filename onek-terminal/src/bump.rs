use onek_shared::*;

fn player_can_move(state: &StateIO, to: Point) -> Option<Note> {
    let cell = state.get_cell_at(to);
    if cell.is_empty() {
        panic!("Attempt to move into unknown cell")
    } else {
        match cell[0].get("id").unwrap().to_id().0.as_str() {
            "deep water" => Some(Note::new(NoteKind::Error, "The water is too deep.".to_owned())),
            "shallow water" => Some(Note::new(
                NoteKind::Environmental,
                "You splash through the water.".to_owned(),
            )),
            "stone wall" => Some(Note::new(NoteKind::Error, "You bounce off the wall.".to_owned())),
            _ => None,
        }
    }
}

// TODO: should we rule out bumps more than one square from oid?
// TODO: what about stuff like hopping? maybe those are restricted to moves?
pub fn handle_bump(state: &StateIO, oid: Oid, loc: Point) {
    if oid != PLAYER_ID {
        todo!("non-player movement isn't implemented yet");
    }

    // At some point may want a dispatch table, e.g.
    // (Actor, Action, Obj) -> Handler(actor, obj)

    // If the move resulted in a note then add it to state.
    let note = player_can_move(state, loc);
    info!("note: {note:?}");
    match &note {
        None => (),
        Some(note) => state.send_mutate(StateMutators::AddNote(note.clone())),
    }

    // Do the move as long as the note isn't an error note.
    match note {
        Some(Note {
            kind: NoteKind::Error,
            text: _,
        }) => (),
        _ => state.send_mutate(StateMutators::MovePlayer(loc)),
    }
}
