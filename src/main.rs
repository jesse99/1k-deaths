mod console;
mod shared;

use std::io;

fn main() -> io::Result<()> {
    let mut console = console::new();
    console.run()
}
