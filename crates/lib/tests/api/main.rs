//! The integration testing suite for the library.

#![expect(
    clippy::panic_in_result_fn,
    reason = "`?` reports setup failures; `assert!` reports the property under test failing"
)]

mod algebra;
mod mask;
mod moves;
mod notation;
mod reconstructions;
mod scramble;
mod solving;
mod stats;

const IMPLEMENTED_MOVES: [&str; 12] = ["R", "U", "D", "L", "F", "B", "E", "S", "M", "y", "z", "x"];
