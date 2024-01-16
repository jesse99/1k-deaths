use onek_types::*;

fn player_can_move(state: &StateIO, to: Point) -> Option<Note> {
    let cell = state.get_cell_at(to);
    match cell.terrain {
        Terrain::Dirt => None, // TODO: these should depend on character type (and maybe affects)
        Terrain::ShallowWater => Some(Note::new(
            NoteKind::Environmental,
            "You splash through the water.".to_owned(),
        )),
        Terrain::DeepWater => Some(Note::new(NoteKind::Error, "The water is too deep.".to_owned())),
        Terrain::Wall => Some(Note::new(NoteKind::Error, "You bounce off the wall.".to_owned())),
        Terrain::Unknown => panic!("Attempt to move into unknown cell"),
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
