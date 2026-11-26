use std::{fs::File, io::Write, time::Duration};

use anyhow::Ok;
use rubiks_cube::{Cube3x3, Method, Puzzle, Reconstruction, Roux};
use virtual_cube_timer::{
    ValidMethods,
    solves::{SolveList, TimedSolve},
};

const SOLVES: usize = 20;

fn main() -> anyhow::Result<()> {
    let mut cube = Cube3x3::default();

    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/solves.txt");

    let mut file = File::options().create(true).append(true).open(path)?;

    file.set_len(0)?;

    let mut list = SolveList::new();

    for i in 0..SOLVES {
        let scramble = Cube3x3::scramble_with_seed(20 + i as u64)?;
        cube = cube.apply(&scramble);
        let solution = Roux::default().to_technique().solve(&mut cube)?;
        let time = Duration::from_secs_f64(20.0 * fastrand::f64());
        let mut recon = Reconstruction::new(scramble, ValidMethods::Roux);
        recon.extend(solution.moves().copied());

        list.push(TimedSolve::new(recon, time));
    }

    file.write_fmt(format_args!("{list}"))?;

    Ok(())
}
