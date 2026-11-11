//! The integration tests that build the kociemba prune tables.
#![expect(
    clippy::panic_in_result_fn,
    reason = "`?` reports setup failures; `assert!` reports the property under test failing"
)]

mod kociemba;
mod scramble;
