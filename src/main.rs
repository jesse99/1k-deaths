mod console;
mod shared;

use std::io;

fn main() -> io::Result<()> {
    let console = console::new();
    console.run()
}
