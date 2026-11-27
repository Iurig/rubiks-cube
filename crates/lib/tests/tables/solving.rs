use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rubiks_cube::{Cube3x3, Method, Puzzle, Reconstruction, roux::*};

#[test]
#[ntest::timeout(15000)]
fn every_roux_option_agrees_with_recon() -> anyhow::Result<()> {
    let seeds = 4;
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
                scrambles.par_iter().for_each(|scramble| {
                    let mut scrambled_cube = Cube3x3::from(scramble);
                    let expected = roux.clone().solve(&mut scrambled_cube).unwrap();
                    let mut recon = Reconstruction::new(scramble.clone(), roux.clone());
                    recon.extend(expected.moves().copied());
                    assert_eq!(expected, recon.into());
                    assert!(scrambled_cube.is_solved());
                });
            }
        }
    }
    Ok(())
}

#[test]
#[ntest::timeout(200)]
fn expand_and_push_agree() {}
