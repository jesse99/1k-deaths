// #[cfg(test)]
// use ipmpsc::{Receiver, Sender, SharedRingBuffer};
// #[cfg(test)]
// use onek_types::*;
// #[cfg(test)]
// use std::time::Duration;

// #[cfg(test)]
// impl StateIO {
//     pub fn send_mutate(&self, mutate: StateMutators) {
//         let mesg = StateMessages::Mutate(mutate);
//         let result = self.tx.send(&mesg);
//         assert!(!result.is_err(), "error sending {mesg} to State: {result:?}")
//     }
// }
