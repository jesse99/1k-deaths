use ipmpsc::{Receiver, SharedRingBuffer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let map_file = "/tmp/tester-state-buffer";
    let rx = Receiver::new(SharedRingBuffer::create(map_file, 32 * 1024)?);

    println!(
        "Ready!  Now run `cargo run --example ipmpsc-send {}` in another terminal.",
        map_file
    );

    loop {
        println!("received {:?}", rx.recv::<String>()?); // TODO: do we want zero-copy?
    }
}
