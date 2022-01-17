use super::{Event, Game, Message, Point, State, Tag, Topic};
use rand::prelude::*;

pub fn interact_with_spectator(game: &Game, events: &mut Vec<Event>) {
    let messages = if matches!(game.state, State::Bumbling) {
        vec![
            "I hope you're prepared to die!",
            "The last champion only lasted thirty seconds.",
            "How can you defeat a man who will not stay dead?",
            "I have 10 gold on you lasting over two minutes!",
            "You're just another dead man walking.",
        ]
    } else {
        vec![
            "I can't believe that the Emperor is dead.",
            "You're my hero!",
            "You've done the impossible!",
        ]
    };
    let text = messages.iter().choose(&mut *game.rng()).unwrap();

    let mesg = Message::new(Topic::NPCSpeaks, text);
    events.push(Event::AddMessage(mesg));
}

pub fn interact_with_doorman(game: &Game, loc: &Point, events: &mut Vec<Event>) {
    let cell = game.level.get(&game.level.player());
    let obj = cell.get(&Tag::Character);
    match obj.inventory() {
        Some(items) if items.iter().any(|obj| obj.description.contains("Doom")) => {
            let mesg = Message::new(Topic::NPCSpeaks, "Ahh, a new champion for the Emperor!");
            events.push(Event::AddMessage(mesg));

            if let Some(new_loc) = game.find_empty_cell(loc) {
                events.push(Event::NPCMoved(*loc, new_loc));
            }
        }
        _ => {
            let mesg = Message::new(Topic::NPCSpeaks, "You are not worthy.");
            events.push(Event::AddMessage(mesg));
        }
    }
}

pub fn interact_with_rhulad(_game: &Game, loc: &Point, events: &mut Vec<Event>) {
    let mesg = Message::new(
        Topic::Important,
        "After an epic battle you kill the Emperor!",
    );
    events.push(Event::AddMessage(mesg));

    events.push(Event::DestroyObject(*loc, Tag::Character));
    events.push(Event::AddObject(*loc, super::make::emp_sword()));
    events.push(Event::AddToInventory(*loc));
    events.push(Event::StateChanged(State::KilledRhulad));
}
