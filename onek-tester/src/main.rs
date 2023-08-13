use ipmpsc::{Sender, SharedRingBuffer};
use onek_types::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: do a better job with errors, should log and return a decent error
    let tx = Sender::new(SharedRingBuffer::open("/tmp/tester-state-buffer")?);

    let mesg = StateMessages::Mutate(StateMutators::MovePlayer(Point::new(1, 1)));
    tx.send(&mesg)?;

    let mesg = StateMessages::Mutate(StateMutators::MovePlayer(Point::new(2, 1)));
    tx.send(&mesg)?;

    let mesg = StateMessages::Mutate(StateMutators::MovePlayer(Point::new(3, 1)));
    tx.send(&mesg)?;

    Ok(())
}
