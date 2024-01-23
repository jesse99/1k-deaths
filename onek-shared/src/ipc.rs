use super::*;
use ipmpsc::{Receiver, Result, Sender, SharedRingBuffer};
use log::info;
use std::time::Duration;

/// Used by frontend's to communicate with the backend.
pub struct IPC {
    tx: ipmpsc::Sender,
    rx: ipmpsc::Receiver,
    rx_name: ChannelName,
}

// Constructors
impl IPC {
    /// Typically rx_channel_name is something like "/tmp/to-APP_NAME".
    pub fn new(rx_channel_name: &str) -> IPC {
        let tx = match SharedRingBuffer::open("/tmp/backend-sink") {
            Ok(buffer) => Sender::new(buffer),
            Err(err) => panic!("error opening backend-sink: {err:?}"),
        };

        let rx_name = ChannelName::new(rx_channel_name);
        let rx = match SharedRingBuffer::create(rx_channel_name, 32 * 1024) {
            Ok(buffer) => Receiver::new(buffer),
            Err(err) => panic!("error opening {rx_channel_name}: {err:?}"),
        };

        let mesg = StateMessages::RegisterForQuery(rx_name.clone());
        info!("sending {mesg}");
        let result = tx.send(&mesg);
        assert!(!result.is_err(), "error sending RegisterForQuery to State: {result:?}");

        IPC { tx, rx, rx_name }
    }

    pub fn reset(&self, reason: &str, map: &str) {
        let mesg = StateMessages::Mutate(StateMutators::Reset {
            reason: reason.to_string(),
            map: map.to_string(),
        });
        info!("sending {mesg}");
        let result = self.tx.send(&mesg);
        assert!(!result.is_err(), "error sending Reset to State: {result:?}");
    }
}

// Queries
impl IPC {
    pub fn get_cell_at(&self, loc: Point) -> Cell {
        let query = StateQueries::CellAt(loc);
        let response = self.send_query(query);
        match response {
            StateResponse::Cell(cell) => cell,
            _ => panic!("Expected Cell from CellAt query not {response}"),
        }
    }

    pub fn get_notes(&self, count: usize) -> Vec<Note> {
        let query = StateQueries::Notes(count);
        let response = self.send_query(query);
        match response {
            StateResponse::Notes(notes) => notes,
            _ => panic!("Expected Notes from Notes query not {response}"),
        }
    }

    pub fn get_player_view(&self) -> View {
        let query = StateQueries::PlayerView();
        let response = self.send_query(query);
        match response {
            StateResponse::Map(map) => map,
            _ => panic!("Expected View from PlayerView query not {response}"),
        }
    }

    pub fn get_player_loc(&self) -> Point {
        let query = StateQueries::PlayerLoc();
        let response = self.send_query(query);
        match response {
            StateResponse::Location(loc) => loc,
            _ => panic!("Expected Point from PlayerLoc query not {response}"),
        }
    }

    fn send_query(&self, query: StateQueries) -> StateResponse {
        let mesg = StateMessages::Query(self.rx_name.clone(), query);
        let result = self.tx.send(&mesg);
        assert!(!result.is_err(), "error sending {mesg} to State: {result:?}");

        let result = self.rx.recv_timeout(Duration::from_millis(100));
        assert!(!result.is_err(), "error receiving from State: {result:?}");

        let option = result.unwrap();
        assert!(option.is_some(), "timed out receiving {mesg} from State");

        option.unwrap()
    }
}

// Mutators
impl IPC {
    pub fn send_mutate(&self, mutate: StateMutators) {
        let mesg = StateMessages::Mutate(mutate);
        let result = self.tx.send(&mesg);
        assert!(!result.is_err(), "error sending {mesg} to State: {result:?}");

        let _: Result<Option<StateResponse>> = self.rx.recv_timeout(Duration::from_millis(100));
    }
}
