//! The kociemba solves. They build the prune tables, so they live here rather than in the lib's
//! unit tests: a broken hash then fails the fast unit tests first, and cargo stops before any
//! table is built.

use rubiks_cube::zn::Zn;
use rubiks_cube::*;

/// The four E-slice edges. Phase 1 brings them into the E slice; phase 2 puts them in place.
const RIM: [Piece3x3; 4] = [
    Piece3x3::Edge(Edge::Fl),
    Piece3x3::Edge(Edge::Fr),
    Piece3x3::Edge(Edge::Bl),
    Piece3x3::Edge(Edge::Br),
];

/// The step of `Kociemba` called `name`.
#[expect(clippy::panic, reason = "a test helper: a missing step fails the test")]
fn phase<'a>(technique: &'a Technique<Cube3x3>, name: &str) -> &'a dyn Step<Cube3x3> {
    technique
        .steps()
        .find(|step| step.name() == name)
        .unwrap_or_else(|| panic!("kociemba has no step called {name}"))
}

/// Phase 1's goal written from the definition, independent of the prune tables: every
/// corner twist and edge flip is zero, and the four E-slice edges sit in E-slice slots.
fn in_domino_subgroup(puzzle: &Cube3x3) -> bool {
    Piece3x3::all()
        .filter(|p| matches!(p, Piece3x3::Corner(_)))
        .all(|slot| puzzle.orientation_at(slot) == Orientation3x3::Twist(Zn::ZERO))
        && Piece3x3::all()
            .filter(|p| matches!(p, Piece3x3::Edge(_)))
            .all(|slot| puzzle.orientation_at(slot) == Orientation3x3::Flip(Zn::ZERO))
        && RIM.iter().all(|&p| RIM.contains(&puzzle.slot_of(p)))
}

/// The moves of `solution`, all segments in order, applied to `start`.
fn replay(start: &Cube3x3, solution: &Solution<Cube3x3>) -> Cube3x3 {
    solution
        .iter()
        .flat_map(Segment::moves)
        .fold(*start, |cube, &m| cube * m)
}

#[test]
#[expect(clippy::let_underscore_must_use, reason = "this is a test")]
fn phase_1_solves() {
    let _ = env_logger::builder().is_test(true).try_init();
    let mut cube = Cube3x3::scrambled_with_seed(0);
    let solution = phase(&Kociemba.to_technique(), "Phase 1").solve(&mut cube);

    assert!(in_domino_subgroup(&cube), "{solution:?}\n {cube}");
}

#[test]
fn phase_2_leaves_a_solved_cube_alone() {
    let mut cube = Cube3x3::default();
    let solution = phase(&Kociemba.to_technique(), "Phase 2")
        .solve(&mut cube)
        .unwrap();
    assert_eq!(solution.move_count(), 0);
    assert_eq!(cube, Cube3x3::default());
}

#[test]
fn phase_2_is_not_done_on_an_unsolved_domino_state() {
    let cube = Cube3x3::from_moves("U R2").unwrap();
    assert!(!phase(&Kociemba.to_technique(), "Phase 2").is_done(&cube));
}

/// The search is iterative deepening with lower bounds, so it never returns more moves than
/// the sequence that made the state.
#[test]
fn phase_2_solves_a_domino_state_in_at_most_its_length() {
    let start = Cube3x3::from_moves("R2 U F2 D' L2 U2 B2 D R2 U'").unwrap();
    let mut cube = start;
    let solution = phase(&Kociemba.to_technique(), "Phase 2")
        .solve(&mut cube)
        .unwrap();
    assert!(
        cube.is_solved(),
        "{solution}
{cube}"
    );
    assert!(solution.move_count() <= 10, "{solution}");
    assert!(replay(&start, &solution).is_solved(), "{solution}");
}

#[test]
fn phase_2_rejects_a_cube_outside_the_domino_subgroup() {
    let mut cube = Cube3x3::from_moves("R").unwrap();
    let result = phase(&Kociemba.to_technique(), "Phase 2").solve(&mut cube);
    assert!(
        matches!(result, Err(StepError::UnreachableGoal)),
        "{result:?}"
    );
    assert_eq!(cube, Cube3x3::from_moves("R").unwrap());
}

#[test]
fn phase_2_solves_what_phase_1_leaves() {
    let mut cube = Cube3x3::scrambled_with_seed(0);
    let kociemba = Kociemba.to_technique();
    phase(&kociemba, "Phase 1").solve(&mut cube).unwrap();
    let solution = phase(&kociemba, "Phase 2").solve(&mut cube).unwrap();
    assert!(
        cube.is_solved(),
        "{solution}
{cube}"
    );
}

#[test]
fn kociemba_solves_a_scramble() {
    let start = Cube3x3::scrambled_with_seed(0);
    let mut cube = start;
    let solution = Kociemba.solve(&mut cube).unwrap();
    let names: Vec<&str> = solution.iter().map(Segment::name).collect();
    assert_eq!(names, ["Phase 1", "Phase 2"]);
    assert!(
        cube.is_solved(),
        "{solution}
{cube}"
    );
    assert!(replay(&start, &solution).is_solved(), "{solution}");
}
