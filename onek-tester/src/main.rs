#[cfg(test)]
use ipmpsc::{Receiver, Sender, SharedRingBuffer};
use onek_types::*;
#[cfg(test)]
use std::time::Duration;

trait ToSnapshot {
    fn to_snapshot(&self, tx_state: &ipmpsc::Sender) -> String;
}

fn terrain_to_char(terrain: Terrain) -> char {
    match terrain {
        Terrain::Dirt => ' ',
        Terrain::Wall => '#',
    }
}

impl ToSnapshot for EditCount {
    fn to_snapshot(&self, _tx_state: &ipmpsc::Sender) -> String {
        format!("edit {self}")
    }
}

impl ToSnapshot for StateResponse {
    fn to_snapshot(&self, tx_state: &ipmpsc::Sender) -> String {
        match self {
            StateResponse::Map(map) => map.to_snapshot(tx_state),
            StateResponse::Updated(count) => count.to_snapshot(tx_state),
        }
    }
}

impl ToSnapshot for View {
    fn to_snapshot(&self, _tx_state: &ipmpsc::Sender) -> String {
        let mut result = String::with_capacity(200);
        for y in self.top_left.y..=self.top_left.y + self.size().height {
            for x in self.top_left.x..=self.top_left.x + self.size().width {
                let loc = Point::new(x, y);
                match self.cells.get(&loc) {
                    Some(cell) => {
                        if cell.character.unwrap_or(NULL_ID) == PLAYER_ID {
                            result.push('@');
                        } else {
                            result.push(terrain_to_char(cell.terrain));
                        }
                    }
                    None => result.push(' '),
                }
            }
            result.push('\n');
        }
        // At some point will need to use tx_state to include details about objects.
        result
    }
}

#[cfg(test)]
fn state_sender() -> ipmpsc::Sender {
    match SharedRingBuffer::open("/tmp/state-sink") {
        Ok(buffer) => Sender::new(buffer),
        Err(err) => panic!("error opening state-sink: {err:?}"),
    }
}

#[cfg(test)]
fn state_receiver(tx_state: &ipmpsc::Sender) -> (String, ipmpsc::Receiver) {
    let name = "/tmp/tester-sink";
    let rx_state = match SharedRingBuffer::create(name, 32 * 1024) {
        Ok(buffer) => Receiver::new(buffer),
        Err(err) => panic!("error opening tester-sink: {err:?}"),
    };

    let mesg = StateMessages::RegisterForQuery(ChannelName::new(name));
    let result = tx_state.send(&mesg);
    assert!(!result.is_err(), "error sending RegisterForQuery to State: {result:?}");

    (name.to_string(), rx_state)
}

#[cfg(test)]
fn send_reset(tx_state: &ipmpsc::Sender, map: &str) {
    let mesg = StateMessages::Mutate(StateMutators::Reset(map.to_string()));
    let result = tx_state.send(&mesg);
    assert!(!result.is_err(), "error sending to State: {result:?}")
}

#[cfg(test)]
fn get_player_view(tx_state: &ipmpsc::Sender, rx_state: &ipmpsc::Receiver, name: &str) -> StateResponse {
    let name = ChannelName::new(name);
    let mesg = StateMessages::Query(StateQueries::PlayerView(name));
    let result = tx_state.send(&mesg);
    assert!(!result.is_err(), "error sending to State: {result:?}");

    let result = rx_state.recv_timeout(Duration::from_millis(100));
    assert!(!result.is_err(), "error receiving from State: {result:?}");

    let option = result.unwrap();
    assert!(option.is_some(), "timed out receiving from State");

    option.unwrap()
}

#[test]
fn test_from_str() {
    let tx_state = state_sender();
    send_reset(
        &tx_state,
        "###\n\
             #@#\n\
             ###",
    );

    let (rx_name, rx_state) = state_receiver(&tx_state);
    let state = get_player_view(&tx_state, &rx_state, &rx_name);
    insta::assert_display_snapshot!(state.to_snapshot(&tx_state));
}

fn main() {
    println!("Run this as a cargo insta test");
}
