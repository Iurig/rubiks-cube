use rubiks_cube::*;

#[test]
fn scramble_and_solve_with_parsing_round_trip() -> anyhow::Result<()> {
    let scramble = Cube3x3::scramble_with_seed(0)?;
    let solution = Roux::default()
        .solve(&mut Cube3x3::default().apply(&scramble))?
        .to_string();

    assert!(Cube3x3::from_moves(&format!("{scramble}\n {solution}"))?.is_solved());
    Ok(())
}

#[test]
fn a_random_scramble_algorithm_leaves_the_cube_unsolved() -> anyhow::Result<()> {
    let scramble = Cube3x3::scramble()?;
    assert!(
        !Cube3x3::default().apply(&scramble).is_solved(),
        "{scramble}"
    );
    Ok(())
}
