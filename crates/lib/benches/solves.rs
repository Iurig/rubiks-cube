//! Benchmarking tool.

#![allow(
    clippy::panic_in_result_fn,
    reason = "benchmarks follow the same convention as tests"
)]

use std::hint::black_box;

use divan::Bencher;
use rubiks_cube::{Cube3x3, Kociemba, Method, Move3x3, Puzzle, Roux, ZZ};
fn main() {
    divan::main();
}

#[divan::bench]
fn to_technique(bencher: Bencher) {
    let roux = &Roux::default();
    bencher.bench_local(move || black_box(roux).to_technique());
}

#[divan::bench(sample_size = 10000, sample_count = 1000)]
fn cube_multiplications(bencher: Bencher) {
    let cube = Cube3x3::default();
    let m: Move3x3 = "R".parse().expect("manually typed");
    bencher.bench_local(move || black_box(cube) * black_box(m));
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
