use ipmpsc::{Sender, SharedRingBuffer};
use std::io::{self, BufRead};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Receiver mist have already created this.
    // TODO: do a better job with errors, should log and return a decent error
    let tx = Sender::new(SharedRingBuffer::open("/tmp/tester-state-buffer")?);

    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    println!("Ready!  Enter some lines of text to send them to the receiver.");

    while handle.read_line(&mut buffer)? > 0 {
        tx.send(&buffer)?;
        buffer.clear();
    }

    Ok(())
}
