use ipmpsc::{Receiver, SharedRingBuffer};
use onek_types::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let map_file = "/tmp/tester-state-buffer";
    let rx = Receiver::new(SharedRingBuffer::create(map_file, 32 * 1024)?);

    println!("Ready!  Now run `./target/debug/onek-tester` in another terminal.",);

    loop {
        let mesg: StateMessages = rx.recv()?;
        println!("received {:?}", mesg); // TODO: do we want zero-copy?
    }
}
