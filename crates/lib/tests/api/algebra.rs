use crate::IMPLEMENTED_MOVES;
use rubiks_cube::*;

/// `Puzzle`'s composition law: a move does the same thing to every state, so what it does to
/// the solved state says where every piece of any state goes, and how it turns.
#[test]
fn every_move_acts_on_any_state_as_it_acts_on_the_solved_state() {
    for seed in 0..20 {
        let p = Cube3x3::scrambled_with_seed(seed);
        for m in <Cube3x3 as Puzzle>::Move::all() {
            let action = Cube3x3::default() * m;
            let moved = p * m;
            for &s in &Piece3x3::all().collect::<Vec<_>>() {
                let src = action.piece_at(s);
                assert_eq!(
                    moved.piece_at(s),
                    p.piece_at(src),
                    "piece at {s:?} after {m}, seed {seed}"
                );
                assert_eq!(
                    moved.orientation_at(s),
                    p.orientation_at(src) + action.orientation_at(s),
                    "orientation at {s:?} after {m}, seed {seed}"
                );
            }
        }
    }
}

#[test]
fn identity_is_two_sided() -> anyhow::Result<()> {
    let r = Cube3x3::from_moves("R")?;
    assert_eq!(Cube3x3::default() * r, r);
    assert_eq!(r * Cube3x3::default(), r);
    Ok(())
}

#[test]
fn pow_0_gives_identity_cube() -> anyhow::Result<()> {
    assert_eq!(Cube3x3::from_moves("R U R' U'")?.pow(0), Cube3x3::default());
    Ok(())
}

#[test]
fn pow_finishes_correctly_for_large_exponent() -> anyhow::Result<()> {
    let large_u32 = u32::MAX - 2;
    let r = Cube3x3::from_moves("R")?;
    assert_eq!(large_u32 % 4, 1);
    assert_eq!(r.pow(large_u32), r);
    Ok(())
}

#[test]
fn move_inverse_is_move_cubed() -> anyhow::Result<()> {
    for m in IMPLEMENTED_MOVES {
        let cube = Cube3x3::from_moves(m)?;
        assert_eq!(cube.inv(), cube.pow(3));
    }
    Ok(())
}

#[test]
fn inverse_is_an_involution() -> anyhow::Result<()> {
    for m in IMPLEMENTED_MOVES {
        let cube = Cube3x3::from_moves(m)?;
        assert_eq!(cube.inv().inv(), cube);
    }
    assert_eq!(Cube3x3::default().inv(), Cube3x3::default());
    Ok(())
}

#[test]
fn mul_is_associative() -> anyhow::Result<()> {
    let a = Cube3x3::from_moves("R")?;
    let b = Cube3x3::from_moves("U2 L")?.pow(2);
    let c = Cube3x3::from_moves("y")?.inv();
    assert_eq!((a * b) * c, a * (b * c));
    Ok(())
}

#[test]
fn inverse_of_product_reverses_order() -> anyhow::Result<()> {
    let a = Cube3x3::from_moves("U")?.pow(1);
    let b = Cube3x3::from_moves("R")?.pow(2);
    assert_eq!((a * b).inv(), b.inv() * a.inv());
    assert_ne!((a * b).inv(), a.inv() * b.inv());
    Ok(())
}

#[test]
fn random_words_stay_reachable_and_undo_cleanly() -> anyhow::Result<()> {
    // Each pair is a modifier and its inverse; `2` and `2'` reach the same
    // state, so inverting a double is the other spelling of it.
    const MODIFIER_PAIRS: [(&str, &str); 2] = [("", "'"), ("2", "2'")];
    fastrand::seed(11);
    for _ in 0..50 {
        let len = fastrand::usize(1..30);
        let mut forward = Vec::with_capacity(len);
        let mut backward = Vec::with_capacity(len);
        for _ in 0..len {
            let part = IMPLEMENTED_MOVES[fastrand::usize(..IMPLEMENTED_MOVES.len())];
            let (modifier, inverse_modifier) = MODIFIER_PAIRS[fastrand::usize(..2)];
            let (modifier, inverse_modifier) = if fastrand::bool() {
                (inverse_modifier, modifier)
            } else {
                (modifier, inverse_modifier)
            };
            forward.push(format!("{part}{modifier}"));
            backward.push(format!("{part}{inverse_modifier}"));
        }
        backward.reverse();
        let forward = forward.join(" ");
        let backward = backward.join(" ");

        let mut cube = Cube3x3::default();
        for token in forward.split(' ') {
            cube = cube.apply_moves(token)?;
            assert!(cube.is_reachable(), "unreachable somewhere in {forward}");
        }
        assert_eq!(Cube3x3::from_moves(&forward)?, cube);
        assert!(
            cube.apply_moves(&backward)?.is_solved(),
            "{forward} then {backward} should be solved"
        );
        assert_eq!(cube * cube.inv(), Cube3x3::default());
    }
    Ok(())
}
