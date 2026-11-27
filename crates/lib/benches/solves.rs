//! Benchmarking tool.

#![allow(
    clippy::panic_in_result_fn,
    clippy::wildcard_imports,
    reason = "benchmarks follow the same convention as tests"
)]

use std::hint::black_box;

use criterion::{BatchSize::SmallInput, *};
use rubiks_cube::{BuildContext, Cube3x3, Kociemba, Method, Move3x3, Puzzle, Roux, Technique, ZZ};

fn to_technique(c: &mut Criterion) {
    let roux = &Roux::default();
    c.bench_function("single-to-technique", |b| {
        b.iter(|| black_box(roux).to_technique());
    });
}

fn cube_multiplications(c: &mut Criterion) {
    let cube = Cube3x3::default();
    let m: Move3x3 = "R".parse().expect("manually typed");
    c.bench_function("cube3x3-mul-move3x3", |b| {
        b.iter(|| black_box(cube) * black_box(m));
    });
}

fn roux_default_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("roux-context-building");
    group.sample_size(10);
    group.bench_function("roux-solve", |b| {
        b.iter_batched(
            || {
                let context = Box::leak(Box::new(BuildContext::new()));
                Roux::default().build_context(context)
            },
            |roux| black_box(roux).solve(&mut Cube3x3::scrambled_with_seed(496)),
            SmallInput,
        );
    });
}

// #[divan::bench(sample_count = 1)]
// fn zz_default_build(c: &mut Criterion) -> anyhow::Result<()> {
//     let mut cube = Cube3x3::scrambled_with_seed(79);
//     ZZ::default().solve(&mut cube)?;
//     assert!(cube.is_solved());
//     Ok(())
// }
//
// #[divan::bench(sample_count = 1)]
// fn kociemba_build(c: &mut Criterion) -> anyhow::Result<()> {
//     let mut cube = Cube3x3::scrambled();
//     Kociemba.solve(&mut cube)?;
//     assert!(cube.is_solved());
//     Ok(())
// }
//
// #[divan::bench]
// fn roux_default_warm(c: &mut Criterion) -> anyhow::Result<()> {
//     let mut cube = Cube3x3::scrambled();
//     Roux::default().solve(&mut cube)?;
//     assert!(cube.is_solved());
//     Ok(())
// }
// #[divan::bench]
// fn zz_default_warm(c: &mut Criterion) -> anyhow::Result<()> {
//     let mut cube = Cube3x3::scrambled();
//     ZZ::default().solve(&mut cube)?;
//     assert!(cube.is_solved());
//     Ok(())
// }
//
// #[divan::bench]
// fn kociemba_warm(c: &mut Criterion) -> anyhow::Result<()> {
//     let mut cube = Cube3x3::scrambled();
//     Kociemba.solve(&mut cube)?;
//     assert!(cube.is_solved());
//     Ok(())
// }
//
criterion_group!(
    benches,
    to_technique,
    cube_multiplications,
    roux_default_build
);
criterion_main!(benches);
