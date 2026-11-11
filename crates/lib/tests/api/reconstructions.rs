use rubiks_cube::*;

/// A reconstruction is a scramble followed by a solution. The scrambled state
/// must be reachable, and the final state must be reachable and solved.
fn assert_reconstruction(scramble: &str, solution: &str) -> anyhow::Result<()> {
    let scrambled = Cube3x3::from_moves(scramble)?;
    assert!(
        scrambled.is_reachable(),
        "scramble reached an unreachable state: {scrambled:?}"
    );
    let finished = scrambled.apply_moves(solution)?;
    assert!(finished.is_reachable());
    assert!(finished.is_solved(), "not solved: {finished:?}");
    Ok(())
}

#[test]
fn fmc_wr_as_multiplication() -> anyhow::Result<()> {
    let scramble = Cube3x3::from_moves(
        "R' U' F D2 L2 F R2 U2 R2 B D2 L B2 D' B2 L' R' B D2 B U2 L U2 R' U' F",
    )?;
    let solve = Cube3x3::from_moves("    D2 F' D2 U2 F' L2 D R2 D B2 F L2 R' F' D U'")?;
    assert!(scramble.is_reachable());
    assert!((scramble * solve).is_solved(), "{:?}", scramble * solve);
    Ok(())
}

#[test]
fn composition_works_on_fmc_wr() -> anyhow::Result<()> {
    let scramble_string = "R' U' F D2 L2 F R2 U2 R2 B D2 L B2 D' B2 L' R' B D2 B U2 L U2 R' U' F";
    let solution_string = "D2 F' D2 U2 F' L2 D R2 D B2 F L2 R' F' D U'";
    let scramble = Cube3x3::from_moves(scramble_string)?;
    assert_eq!(
        Cube3x3::from_moves(&format!("{scramble_string} {solution_string}"))?,
        scramble.apply_moves(solution_string)?,
        "composing {scramble_string} and {solution_string} doesn't result in applying {scramble_string} {solution_string}"
    );
    Ok(())
}

const CFOP_SCRAMBLE: &str = "R2 F' L2 D2 F2 U2 B' L2 F R2 D2 F2 D L' U B R' F' R D R2 U2 ";
const ROUX_SCRAMBLE: &str = "U' L2 D' B2 D R2 F2 D' B2 R2 D B' R F2 R D' B' F U2 R' U D ";

#[test]
fn cfop_solve_hygienized() -> anyhow::Result<()> {
    assert_reconstruction(
        CFOP_SCRAMBLE,
        "z y2
            U' R' L2 x'
            F' U F
            R' U' R U R' U' R
            L U' L'
            U y' U R U' R' U' R U' R2 F R
            U R U' R' U R U2 R' U' R U R' F'",
    )
}

#[test]
fn cfop_solve() -> anyhow::Result<()> {
    assert_reconstruction(
        CFOP_SCRAMBLE,
        "z y2
            U' R' L2 x'
            F' U F
            R' U' R U R' U' R
            L U' L'
            U y' U R U' R' U' R U' R2' F R
            U R U' R' U R U2' R' U' R U R' F'",
    )
}

#[test]
fn roux_solve_with_comments() -> anyhow::Result<()> {
    assert_reconstruction(
        ROUX_SCRAMBLE,
        "y2 F' M F' R U' R U' Fw z' // FB
            U R U r M' U' R U2' R' // SS
            U R' U' R U' R' U' r // SP (CMLL skip)
            U M' U' M U' U' M' U M // EOLR
            U' U' M2' U' M U' U' M' U' U' M2' // EP",
    )
}

#[test]
fn roux_solve_without_comments() -> anyhow::Result<()> {
    assert_reconstruction(
        ROUX_SCRAMBLE,
        "y2 F' M F' R U' R U' Fw z'
            U R U r M' U' R U2' R'
            U R' U' R U' R' U' r
            U M' U' M U' U' M' U M
            U' U' M2' U' M U' U' M' U' U' M2' ",
    )
}

#[test]
fn s_based_roux_solve() -> anyhow::Result<()> {
    assert_reconstruction(
        ROUX_SCRAMBLE,
        "y2 F' M F' R U' R U' Fw z' // FB
U R U r M' U' R U2' R' // SS
U R' U' R U' R' U' r // SP (CMLL skip)
y // mean rotation
U S U' S' U2 S U S' U2 S2 U' // EOLR
 S' U2 S U2 S2 // 4c",
    )
}

#[test]
fn e_based_roux_solve() -> anyhow::Result<()> {
    assert_reconstruction(
        ROUX_SCRAMBLE,
        "y2 F' M F' R U' R U' Fw z' // FB
U R U r M' U' R U2' R' // SS
U R' U' R U' R' U' r // SP (CMLL skip)
z // mean rotation
R E R' E' R2 E R E' R2 E2 R'   // EOLR
E' R2 E R2 E2// 4c",
    )
}

#[test]
fn roux_solve_without_wide_moves() -> anyhow::Result<()> {
    assert_reconstruction(
        ROUX_SCRAMBLE,
        "y2 F' M F' R U' R U' B
            U R U R M2 U' R U2 R'
            U R' U' R U' R' U' R M'
            U M' U' M U' U' M' U M
            U' U' M2 U' M U' U' M' U' U' M2 ",
    )
}

#[test]
fn roux_solve_removes_comments() -> anyhow::Result<()> {
    assert_eq!(
        Cube3x3::from_moves(concat!(
            "U' L2 D' B2 D R2 F2 D' B2 R2 D B' R F2 R D' B' F U2 R' U D ",
            "y2 F' M F' R U' R U' Fw z'
            U R U r M' U' R U2' R'
            U R' U' R U' R' U' r
            U M' U' M U' U' M' U M
            U' U' M2' U' M U' U' M' U' U' M2' "
        ))?,
        Cube3x3::from_moves(concat!(
            "U' L2 D' B2 D R2 F2 D' B2 R2 D B' R F2 R D' B' F U2 R' U D ",
            "y2 F' M F' R U' R U' Fw z' // FB
            U R U r M' U' R U2' R' // SS
            U R' U' R U' R' U' r // SP (CMLL skip)
            U M' U' M U' U' M' U M // EOLR
            U' U' M2' U' M U' U' M' U' U' M2' // EP"
        ))?
    );
    Ok(())
}
