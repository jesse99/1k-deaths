use super::*;
use ipmpsc::{Sender, SharedRingBuffer};

/// Used by services to communicate with the logic service.
pub struct LogicIO {
    tx: ipmpsc::Sender,
}

// Constructors
impl LogicIO {
    pub fn new() -> LogicIO {
        let tx = match SharedRingBuffer::open("/tmp/logic-sink") {
            Ok(buffer) => Sender::new(buffer),
            Err(err) => panic!("error opening logic-sink: {err:?}"),
        };
        LogicIO { tx }
    }
}

// Messages
impl LogicIO {
    pub fn bump(&self, oid: Oid, loc: Point) {
        let mesg = LogicMessages::Bump(oid, loc);
        let result = self.tx.send(&mesg);
        assert!(!result.is_err(), "error sending {mesg:?} to Logic: {result:?}")
    }
}
