use std::{fs::File, io::Write, time::Duration};

use anyhow::Ok;
use rubiks_cube::{Cube3x3, Method, Puzzle, Roux};
use virtual_cube_timer::{
    ValidMethods,
    solves::{ReconWithScramble, SolveList, TimedSolve},
};

const SOLVES: usize = 10;

fn main() -> anyhow::Result<()> {
    let mut cube = Cube3x3::default();

    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/solves.txt");

    let mut file = File::options().create(true).append(true).open(path)?;

    file.set_len(0)?;

    let mut list = SolveList::new();

    for _ in 0..SOLVES {
        let scramble = Cube3x3::scramble()?;
        cube = cube.apply(&scramble);
        let solution = Roux::default().to_technique().solve(&mut cube)?;
        let time = Duration::from_secs_f64(20.0 * fastrand::f64());

        list.push(TimedSolve::new(
            ReconWithScramble::new(scramble, solution),
            time,
            ValidMethods::Roux,
        ));
    }

    file.write_fmt(format_args!("{list}"))?;

    Ok(())
}
