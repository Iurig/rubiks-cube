use anyhow::Context;
use rubiks_cube::*;

#[test]
fn full_solve_and_checking_bfs() -> anyhow::Result<()> {
    let scr = "U B' D L2 U B2 R2 D2 L2 D' U2 B2 R2 L U2 F' R' B L D'\n";
    let mut scrambled = Cube3x3::from_moves(scr)?;

    let recon: String = Roux::default().solve(&mut scrambled)?.to_string();

    assert_eq!(
        Cube3x3::from_moves(&(scr.to_string() + recon.as_str()))?,
        Cube3x3::default(),
        "{recon}"
    );
    Ok(())
}

#[test]
fn every_roux_option_solves_on_multiple_seeds() -> anyhow::Result<()> {
    use rubiks_cube::roux::*;
    let seeds = 4;
    let scrambles: Vec<Cube3x3> = (2026..2026 + seeds)
        .map(Cube3x3::scrambled_with_seed)
        .collect();
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
                let roux = Roux::default()
                    .first_block(fb.clone())
                    .second_block(sb.clone())
                    .cmll(cmll);
                let described = format!("{roux:?}");
                for (i, scrambled) in scrambles.iter().enumerate() {
                    let recon = roux
                        .solve(&mut scrambled.clone())
                        .with_context(|| format!("{described}, scramble {i}"))?
                        .to_string();
                    assert!(
                        scrambled.apply_moves(&recon)?.is_solved(),
                        "{described} did not solve scramble {i}:\n{scrambled}\n{recon}"
                    );
                }
            }
        }
    }
    Ok(())
}

#[test]
fn manually_defined_roux_solves() -> anyhow::Result<()> {
    let eolr = SearchStep::builder("EOLR", AlgSet::from_static_parts("M U"))
        .goal(PieceSet::from_algset(&AlgSet::<Cube3x3>::from_static(
            "M' U2 M U2\n M U2 M' U2",
        )))
        .clone();

    let advanced_roux = Technique::<Cube3x3>::start_with(SearchStep::builder(
        "FB",
        AlgSet::from_static_parts("R U F B D M r"),
    ))
    .then(SearchStep::builder(
        "SB",
        AlgSet::from_static_parts("R U r M"),
    ))
    .then(
        SearchStep::builder(
            "CMLL",
            AlgSet::from_static(include_str!(
                "../../src/methods/cube3x3/algsets/cmll/one_look.txt"
            )),
        )
        .free_algs(AlgSet::from_static_parts("U M"))
        .goal(PieceSet::from_algset(&AlgSet::from_static("U M' U' \n M")))
        .clone(),
    )
    .then(eolr)
    .finish(SearchStep::builder("LSE", AlgSet::from_static_parts("M U")))?;

    let mut cube = Cube3x3::scrambled_with_seed(3);

    advanced_roux.solve(&mut cube)?;

    assert!(cube.is_solved());
    Ok(())
}

#[test]
fn skips_work() -> anyhow::Result<()> {
    let mut scrambled = Cube3x3::from_moves("U2")?;
    let recon = Roux::default().solve(&mut scrambled)?.to_string();
    assert!(scrambled.is_solved(), "{recon}");
    Ok(())
}

#[test]
fn zz_solves_the_cube() -> anyhow::Result<()> {
    let mut cube = Cube3x3::scrambled_with_seed(2);

    let _ = ZZ::default()
        .solve_steps(&mut cube)
        .collect::<Result<Solution<Cube3x3>, _>>()?;
    assert!(cube.is_solved());
    Ok(())
}

#[test]
fn every_zz_option_combination_solves() -> anyhow::Result<()> {
    use rubiks_cube::zz::{FirstLook, LL, SecondLook};
    let mut last_layers = vec![LL::OneLook];
    for first in [FirstLook::Ocll, FirstLook::Coll] {
        for second in [SecondLook::OneLook, SecondLook::TwoLook] {
            last_layers.push(LL::TwoLook {
                first: first.clone(),
                second,
            });
        }
    }
    for (seed, ll) in (0..).zip(last_layers) {
        let zz = ZZ::default().ll(ll);
        let mut cube = Cube3x3::scrambled_with_seed(seed);
        zz.solve(&mut cube)
            .with_context(|| format!("{zz:?}, seed {seed}"))?;
        assert!(
            cube.is_solved(),
            "{zz:?} did not solve seed {seed}:\n{cube}"
        );
    }
    Ok(())
}

/// ZZ's EO Line: the goal names a piece in two edge slots and asks only for orientation in the
/// other ten. When the search key left out the flip of a stray edge in a named slot, seed 2 ended
/// this step with two edges flipped.
#[test]
fn a_goal_mixing_named_and_orientation_only_slots_is_met() -> anyhow::Result<()> {
    let eo_line_moves = AlgSet::from_static_parts("F B U R L D");
    let step = SearchStep::builder("EO Line", eo_line_moves.clone())
        .goal(PieceSet::from_algset(&AlgSet::from_static_parts("U R L")))
        .expect_solved(PieceSet::from_algset(&eo_line_moves))
        .build()?;
    for seed in 0..4 {
        let mut cube = Cube3x3::scrambled_with_seed(seed);
        step.solve(&mut cube)?;
        assert!(step.is_done(&cube), "seed {seed}:\n{cube}");
    }
    Ok(())
}

#[test]
fn lse_solves_with_either_move_group_free() -> anyhow::Result<()> {
    let scrambles = [
        "M U M' U2 M2 U' M",
        "U M2 U' M U2 M' U",
        "M' U' M2 U M' U2 M2",
        "U2 M U M' U' M2 U2 M'",
    ];
    for (costly, free) in [("M", "U"), ("U", "M")] {
        for scramble in scrambles {
            let step = SearchStep::builder("LSE", AlgSet::from_static_parts(costly))
                .free_algs(AlgSet::from_static_parts(free))
                .build_in(&BuildContext::new())?;
            let mut cube = Cube3x3::from_moves(scramble)?;
            step.solve(&mut cube)
                .with_context(|| format!("costly {costly}, free {free}, {scramble}"))?;
            assert!(
                cube.is_solved(),
                "costly {costly}, free {free}, {scramble}:\n{cube}"
            );
        }
    }
    Ok(())
}

#[test]
fn fb_solves_with_any_split_of_free_moves() -> anyhow::Result<()> {
    for (costly, free) in [
        ("", "F U R L D B M r"),
        ("U", "F R L D B M r"),
        ("F R L D B M r", "U"),
    ] {
        for seed in 1..2 {
            let step = SearchStep::builder("FB", AlgSet::from_static_parts(costly))
                .free_algs(AlgSet::from_static_parts(free))
                .goal(PieceSet::from_algset(
                    &AlgSet::<Cube3x3>::from_static_parts("U R M r"),
                ))
                .build_in(&BuildContext::new())?;
            let mut cube = Cube3x3::scrambled_with_seed(seed);
            step.solve(&mut cube)
                .with_context(|| format!("costly {costly}, free {free}, seed {seed}"))?;
            assert!(
                step.is_done(&cube),
                "costly {costly}, free {free}, seed {seed}:\n{cube}"
            );
        }
    }
    Ok(())
}
