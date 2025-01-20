mod backend;
mod console;
mod shared;

use std::io;

fn main() -> io::Result<()> {
    let game = backend::new();
    let mut console = console::new(game);
    console.run()
}
