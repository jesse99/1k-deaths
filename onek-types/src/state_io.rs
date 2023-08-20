use super::*;
use ipmpsc::{Receiver, Sender, SharedRingBuffer};
use std::time::Duration;

/// Used by services to communicate with the state service.
pub struct StateIO {
    tx: ipmpsc::Sender,
    rx: ipmpsc::Receiver,
    rx_name: ChannelName,
}

// New functions
impl StateIO {
    // TODO: rename this new_for_tests (have a private new that takes an Option<Map>)
    pub fn new(map: &str) -> StateIO {
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

        StateIO {
            tx,
            rx,
            rx_name: ChannelName::new(name),
        }
    }
}

// Queries
impl StateIO {
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
    fn send_query(&self, query: StateQueries) -> StateResponse {
        let mesg = StateMessages::Query(query);
        let result = self.tx.send(&mesg);
        assert!(!result.is_err(), "error sending {mesg} to State: {result:?}");

        let result = self.rx.recv_timeout(Duration::from_millis(100));
        assert!(!result.is_err(), "error receiving from State: {result:?}");

        let option = result.unwrap();
        assert!(option.is_some(), "timed out receiving {mesg} from State");

        option.unwrap()
    }
}

// Mutators (in general only the logic service should send these)
impl StateIO {
    pub fn begin_read_transaction(&self, name: String) {
        let mutate = StateMutators::BeginReadTransaction(name);
        self.send_mutate(mutate);
    }

    pub fn end_read_transaction(&self, name: String) {
        let mutate = StateMutators::EndReadTransaction(name);
        self.send_mutate(mutate);
    }

    fn send_mutate(&self, mutate: StateMutators) {
        let mesg = StateMessages::Mutate(mutate);
        let result = self.tx.send(&mesg);
        assert!(!result.is_err(), "error sending {mesg} to State: {result:?}")
    }
}
