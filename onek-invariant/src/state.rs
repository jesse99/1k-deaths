#[cfg(test)]
use ipmpsc::{Receiver, Sender, SharedRingBuffer};
#[cfg(test)]
use onek_types::*;
#[cfg(test)]
use std::time::Duration;

#[cfg(test)] // TODO: get rid of this once we start listening to State updates
pub struct State {
    pub tx: ipmpsc::Sender,
    pub rx: ipmpsc::Receiver,
    pub rx_name: ChannelName,
}

#[cfg(test)]
impl State {
    pub fn new(map: &str) -> State {
        let tx = match SharedRingBuffer::open("/tmp/state-sink") {
            Ok(buffer) => Sender::new(buffer),
            Err(err) => panic!("error opening state-sink: {err:?}"),
        };

        // Note that we have to do this early because Reset will zap the RegisterForQuery below.
        let mesg = StateMessages::Mutate(StateMutators::Reset(map.to_string()));
        let result = tx.send(&mesg);
        assert!(!result.is_err(), "error sending Reset to State: {result:?}");

        let name = "/tmp/tester-sink";
        let rx = match SharedRingBuffer::create(name, 32 * 1024) {
            Ok(buffer) => Receiver::new(buffer),
            Err(err) => panic!("error opening tester-sink: {err:?}"),
        };

        let mesg = StateMessages::RegisterForQuery(ChannelName::new(name));
        let result = tx.send(&mesg);
        assert!(!result.is_err(), "error sending RegisterForQuery to State: {result:?}");

        State {
            tx,
            rx,
            rx_name: ChannelName::new(name),
        }
    }

    // pub fn send_mutate(&self, mutate: StateMutators) {
    //     let mesg = StateMessages::Mutate(mutate.clone());
    //     let result = self.tx.send(&mesg);
    //     assert!(!result.is_err(), "error sending {mutate:?} to State: {result:?}")
    // }

    pub fn get_player_view(&self) -> View {
        let query = StateQueries::PlayerView(self.rx_name.clone());
        let response = self.send_query(query);
        match response {
            StateResponse::Map(map) => map,
            _ => panic!("Expected View from PlayerView query not {response}"),
        }
    }

    pub fn get_player_loc(&self) -> Point {
        let query = StateQueries::PlayerLoc(self.rx_name.clone());
        let response = self.send_query(query);
        match response {
            StateResponse::Location(loc) => loc,
            _ => panic!("Expected Point from PlayerLoc query not {response}"),
        }
    }

    pub fn send_query(&self, query: StateQueries) -> StateResponse {
        let mesg = StateMessages::Query(query.clone());
        let result = self.tx.send(&mesg);
        assert!(!result.is_err(), "error sending {query:?} to State: {result:?}");

        let result = self.rx.recv_timeout(Duration::from_millis(100));
        assert!(!result.is_err(), "error receiving from State: {result:?}");

        let option = result.unwrap();
        assert!(option.is_some(), "timed out receiving {query:?} from State");

        option.unwrap()
    }
}
