use anyhow::Ok;
#[expect(clippy::wildcard_imports, reason = "roux-specific example")]
use rubiks_cube::{Cube3x3, Method, Puzzle, Reconstruction, Roux, roux::*};
use std::{fs::File, io::Write};

fn main() -> anyhow::Result<()> {
    let seeds = 4;

    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/roux_solves.txt"
    );

    let mut file = File::options().create(true).append(true).open(path)?;

    file.set_len(0)?;

    let scrambles = (206..206 + seeds)
        .map(Cube3x3::scramble_with_seed)
        .collect::<Result<Vec<_>, _>>()?;
    for fb in [
        FirstBlock::OneLook,
        FirstBlock::SquarePair,
        FirstBlock::EdgePairPair,
    ] {
        for sb in [
            SecondBlock::OneLook,
            SecondBlock::SquarePair,
            SecondBlock::EdgePairPair,
        ] {
            for cmll in [Cmll::OneLook, Cmll::TwoLook] {
                let roux = Roux::new()
                    .first_block(fb.clone())
                    .second_block(sb.clone())
                    .cmll(cmll);
                for scramble in &scrambles {
                    let mut scrambled_cube = Cube3x3::from(scramble);
                    let expected = roux.clone().solve(&mut scrambled_cube)?;
                    let mut recon = Reconstruction::new(scramble.clone(), roux.clone());
                    recon.extend(expected.moves().copied());
                    file.write_fmt(format_args!("{recon}\n\n"))?;
                    assert_eq!(expected, recon.into());
                    assert!(scrambled_cube.is_solved());
                }
            }
        }
    }
    Ok(())
}
