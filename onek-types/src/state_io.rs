use super::*;
use ipmpsc::{Receiver, Sender, SharedRingBuffer};
use std::time::Duration;

/// Used by services to communicate with the state service.
pub struct StateIO {
    tx: ipmpsc::Sender,
    rx: ipmpsc::Receiver,
    rx_name: ChannelName,
}

// Constructors
impl StateIO {
    /// Typically rx_channel_name is something like "/tmp/state-to-SERVICE_NAME".
    pub fn new(rx_channel_name: &str) -> StateIO {
        StateIO::new_with_option(None, rx_channel_name)
    }

    /// For testing
    pub fn new_with_map(map: &str, rx_channel_name: &str) -> StateIO {
        StateIO::new_with_option(Some(map), rx_channel_name)
    }

    fn new_with_option(map: Option<&str>, rx_channel_name: &str) -> StateIO {
        let tx = match SharedRingBuffer::open("/tmp/state-sink") {
            Ok(buffer) => Sender::new(buffer),
            Err(err) => panic!("error opening state-sink: {err:?}"),
        };

        if let Some(map) = map {
            // Note that we have to do this early because Reset will zap the RegisterForQuery below.
            let mesg = StateMessages::Mutate(StateMutators::Reset(map.to_string()));
            let result = tx.send(&mesg);
            assert!(!result.is_err(), "error sending Reset to State: {result:?}");
        }

        let rx = match SharedRingBuffer::create(rx_channel_name, 32 * 1024) {
            Ok(buffer) => Receiver::new(buffer),
            Err(err) => panic!("error opening {rx_channel_name}: {err:?}"),
        };

        let rx_name = ChannelName::new(rx_channel_name);
        let mesg = StateMessages::RegisterForQuery(rx_name.clone());
        let result = tx.send(&mesg);
        assert!(!result.is_err(), "error sending RegisterForQuery to State: {result:?}");

        StateIO { tx, rx, rx_name }
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

    pub fn send_mutate(&self, mutate: StateMutators) {
        let mesg = StateMessages::Mutate(mutate);
        let result = self.tx.send(&mesg);
        assert!(!result.is_err(), "error sending {mesg} to State: {result:?}")
    }
}
