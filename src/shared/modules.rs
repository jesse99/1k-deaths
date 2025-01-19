//! Traits used to wrap modules. This is essentially a modular monolith design, see
//! https://www.geeksforgeeks.org/what-is-a-modular-monolith for more.
use std::io;

pub trait UI {
    fn run(&self) -> io::Result<()>;
}
