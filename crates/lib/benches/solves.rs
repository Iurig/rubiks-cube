//! Benchmarking tool.

#![allow(
    clippy::panic_in_result_fn,
    reason = "bechmarks follow the same convention as tests"
)]

use rubiks_cube::{Cube3x3, Kociemba, Method, Puzzle, Roux, ZZ};
fn main() {
    divan::main();
}

#[divan::bench(sample_count = 1)]
fn roux_default_build() -> anyhow::Result<()> {
    let mut cube = Cube3x3::scrambled_with_seed(496);
    Roux::default().solve(&mut cube)?;
    assert!(cube.is_solved());
    Ok(())
}

#[divan::bench(sample_count = 1)]
fn zz_default_build() -> anyhow::Result<()> {
    let mut cube = Cube3x3::scrambled_with_seed(79);
    ZZ::default().solve(&mut cube)?;
    assert!(cube.is_solved());
    Ok(())
}

#[divan::bench(sample_count = 1)]
fn kociemba_build() -> anyhow::Result<()> {
    let mut cube = Cube3x3::scrambled();
    Kociemba.solve(&mut cube)?;
    assert!(cube.is_solved());
    Ok(())
}

#[divan::bench]
fn roux_default_warm() -> anyhow::Result<()> {
    let mut cube = Cube3x3::scrambled();
    Roux::default().solve(&mut cube)?;
    assert!(cube.is_solved());
    Ok(())
}
#[divan::bench]
fn zz_default_warm() -> anyhow::Result<()> {
    let mut cube = Cube3x3::scrambled();
    ZZ::default().solve(&mut cube)?;
    assert!(cube.is_solved());
    Ok(())
}

#[divan::bench]
fn kociemba_warm() -> anyhow::Result<()> {
    let mut cube = Cube3x3::scrambled();
    Kociemba.solve(&mut cube)?;
    assert!(cube.is_solved());
    Ok(())
}
