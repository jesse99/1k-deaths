mod console;
mod game;
mod shared;

use std::io;

fn main() -> io::Result<()> {
    let game = game::new();
    let mut console = console::new(game);
    console.run()
}
