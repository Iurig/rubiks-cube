#![doc = include_str!("DESIGN.md")]
#![allow(
    clippy::pedantic,
    clippy::nursery,
    clippy::allow_attributes_without_reason,
    missing_docs
)]

use rubiks_cube::{cube3x3_method_helpers::*, *};

fn main() {
    println!("{}", Cube3x3::from_moves("R").unwrap().colored());
}
