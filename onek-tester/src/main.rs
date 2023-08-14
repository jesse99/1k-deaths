#[cfg(test)]
use ipmpsc::{Receiver, Sender, SharedRingBuffer};
#[cfg(test)]
use onek_types::*;
#[cfg(test)]
use std::time::Duration;

#[cfg(test)]
trait ToSnapshot {
    fn to_snapshot(&self, test: &SnapshotTest) -> String;
}

#[cfg(test)]
fn terrain_to_char(terrain: Terrain) -> char {
    match terrain {
        Terrain::Dirt => ' ',
        Terrain::Wall => '#',
    }
}

#[cfg(test)]
impl ToSnapshot for EditCount {
    fn to_snapshot(&self, _test: &SnapshotTest) -> String {
        format!("edit {self}")
    }
}

#[cfg(test)]
impl ToSnapshot for StateResponse {
    fn to_snapshot(&self, test: &SnapshotTest) -> String {
        match self {
            StateResponse::Map(map) => map.to_snapshot(test),
            StateResponse::Updated(count) => count.to_snapshot(test),
        }
    }
}

#[cfg(test)]
impl ToSnapshot for View {
    fn to_snapshot(&self, _test: &SnapshotTest) -> String {
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
struct SnapshotTest {
    tx_state: ipmpsc::Sender,
    rx_state: ipmpsc::Receiver,
    rx_name: ChannelName,
}

#[cfg(test)]
impl SnapshotTest {
    fn new(map: &str) -> SnapshotTest {
        let tx_state = match SharedRingBuffer::open("/tmp/state-sink") {
            Ok(buffer) => Sender::new(buffer),
            Err(err) => panic!("error opening state-sink: {err:?}"),
        };

        // Note that we have to do this early because Reset will zap the RegisterForQuery below.
        let mesg = StateMessages::Mutate(StateMutators::Reset(map.to_string()));
        let result = tx_state.send(&mesg);
        assert!(!result.is_err(), "error sending Reset to State: {result:?}");

        let name = "/tmp/tester-sink";
        let rx_state = match SharedRingBuffer::create(name, 32 * 1024) {
            Ok(buffer) => Receiver::new(buffer),
            Err(err) => panic!("error opening tester-sink: {err:?}"),
        };

        let mesg = StateMessages::RegisterForQuery(ChannelName::new(name));
        let result = tx_state.send(&mesg);
        assert!(!result.is_err(), "error sending RegisterForQuery to State: {result:?}");

        SnapshotTest {
            tx_state,
            rx_state,
            rx_name: ChannelName::new(name),
        }
    }

    // fn send_mutate(&self, mutate: StateMutators) {
    //     let mesg = StateMessages::Mutate(mutate.clone());
    //     let result = self.tx_state.send(&mesg);
    //     assert!(!result.is_err(), "error sending {mutate:?} to State: {result:?}")
    // }

    fn send_query(&self, query: StateQueries) -> StateResponse {
        let mesg = StateMessages::Query(query.clone());
        let result = self.tx_state.send(&mesg);
        assert!(!result.is_err(), "error sending {query:?} to State: {result:?}");

        let result = self.rx_state.recv_timeout(Duration::from_millis(100));
        assert!(!result.is_err(), "error receiving from State: {result:?}");

        let option = result.unwrap();
        assert!(option.is_some(), "timed out receiving {query:?} from State");

        option.unwrap()
    }
}

// Queries
#[cfg(test)]
impl SnapshotTest {
    fn get_player_view(&self) -> StateResponse {
        let query = StateQueries::PlayerView(self.rx_name.clone());
        self.send_query(query)
    }
}

// Mutators
#[cfg(test)]
impl SnapshotTest {}

#[test]
fn test_from_str() {
    let test = SnapshotTest::new(
        "###\n\
             #@#\n\
             ###",
    );

    let state = test.get_player_view();
    insta::assert_display_snapshot!(state.to_snapshot(&test));
}

fn main() {
    println!("Run this as a cargo insta test");
}
