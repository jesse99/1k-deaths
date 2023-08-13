#[cfg(test)]
use ipmpsc::{Receiver, Sender, SharedRingBuffer};
#[cfg(test)]
use onek_types::*;
#[cfg(test)]
use std::time::Duration;

#[cfg(test)]
fn state_sender() -> ipmpsc::Sender {
    match SharedRingBuffer::open("/tmp/state-sink") {
        Ok(buffer) => Sender::new(buffer),
        Err(err) => panic!("error opening state-sink: {err:?}"),
    }
}

#[cfg(test)]
fn state_receiver(tx: &ipmpsc::Sender) -> (String, ipmpsc::Receiver) {
    let name = "/tmp/tester-sink";
    let rx = match SharedRingBuffer::create(name, 32 * 1024) {
        Ok(buffer) => Receiver::new(buffer),
        Err(err) => panic!("error opening tester-sink: {err:?}"),
    };

    let mesg = StateMessages::RegisterForQuery(ChannelName::new(name));
    let result = tx.send(&mesg);
    assert!(!result.is_err(), "error sending RegisterForQuery to State: {result:?}");

    (name.to_string(), rx)
}

#[cfg(test)]
fn send_reset(tx: &ipmpsc::Sender, map: &str) {
    let mesg = StateMessages::Mutate(StateMutators::Reset(map.to_string()));
    let result = tx.send(&mesg);
    assert!(!result.is_err(), "error sending to State: {result:?}")
}

#[cfg(test)]
fn get_player_view(tx: &ipmpsc::Sender, rx: &ipmpsc::Receiver, name: &str) -> StateResponse {
    let name = ChannelName::new(name);
    let mesg = StateMessages::Query(StateQueries::PlayerView(name));
    let result = tx.send(&mesg);
    assert!(!result.is_err(), "error sending to State: {result:?}");

    let result = rx.recv_timeout(Duration::from_millis(100));
    assert!(!result.is_err(), "error receiving from State: {result:?}");

    let option = result.unwrap();
    assert!(option.is_some(), "timed out receiving from State");

    option.unwrap()
}

#[test]
fn test_from_str() {
    let tx = state_sender();
    send_reset(
        &tx,
        "###\n\
             #@#\n\
             ###",
    );

    let (rx_name, rx) = state_receiver(&tx);
    let state = get_player_view(&tx, &rx, &rx_name);
    insta::assert_display_snapshot!(state);
}

fn main() {
    println!("Run this as a unit test");
}
