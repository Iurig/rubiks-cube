use std::sync::{Arc, LazyLock};

use crate::{
    AlgSet, Algorithm, ByIdentity, ByMembership, Cube3x3, Edge, Indexed, Mask, Method,
    Orientation3x3, Piece3x3, PieceSet, Puzzle, Step,
    methods::{
        Technique,
        step::combine_pruned::{PruneTable, PrunedCombine, PrunedGoal},
    },
    puzzles::cube3x3::{
        moves::{MovablePart, Move3x3, MoveModifier},
        pieces::Face,
    },
    zn::Zn,
};

/// The four E-slice edges. Phase 1 brings them into the E slice; phase 2 puts them in place.
const RIM: [Piece3x3; 4] = [
    Piece3x3::Edge(Edge::Fl),
    Piece3x3::Edge(Edge::Fr),
    Piece3x3::Edge(Edge::Bl),
    Piece3x3::Edge(Edge::Br),
];

/// Every face turn, one per sequence: phase 1's moveset. `2'` is left out, since on a face it
/// reaches the same state as `2` and would only double the branching.
fn face_turns() -> AlgSet<Cube3x3> {
    Move3x3::all()
        .filter(|m| {
            matches!(m.part(), MovablePart::Face(_)) && m.modifier() != MoveModifier::DoublePrime
        })
        .map(|m| Algorithm::from_iter([m]))
        .collect()
}

/// Phase 2's moveset, the moves that keep a cube in the domino subgroup: any turn of U or D,
/// and half turns of the other faces.
fn domino_turns() -> AlgSet<Cube3x3> {
    Move3x3::all()
        .filter(|m| match m.part() {
            MovablePart::Face(Face::U | Face::D) => m.modifier() != MoveModifier::DoublePrime,
            MovablePart::Face(_) => m.modifier() == MoveModifier::Double,
            _ => false,
        })
        .map(|m| Algorithm::from_iter([m]))
        .collect()
}

fn corners() -> impl Iterator<Item = Piece3x3> + Clone {
    Piece3x3::all().filter(|p| matches!(p, Piece3x3::Corner(_)))
}

fn edges() -> impl Iterator<Item = Piece3x3> + Clone {
    Piece3x3::all().filter(|p| matches!(p, Piece3x3::Edge(_)))
}

static CORNERS_PHASE_1_GOAL: LazyLock<PieceSet<Cube3x3>> = LazyLock::new(|| {
    PieceSet::<Cube3x3>::from_pieces_and_orientations([] as [Piece3x3; 0], corners())
});

static CORNERS_PHASE_1_TABLE: LazyLock<PruneTable<PieceSet<Cube3x3>>> =
    LazyLock::new(|| PruneTable::from_goal(&CORNERS_PHASE_1_GOAL, &face_turns()));

static EDGES_PHASE_1_GOAL: LazyLock<PieceSet<Cube3x3>> =
    LazyLock::new(|| PieceSet::<Cube3x3>::from_pieces_and_orientations(RIM, edges()));

static EDGES_PHASE_1_TABLE: LazyLock<PruneTable<PieceSet<Cube3x3>>> =
    LazyLock::new(|| PruneTable::from_goal(&EDGES_PHASE_1_GOAL, &face_turns()));

#[derive(Debug)]
enum Phase1Goal {
    Corners,
    Edges,
}
impl PrunedGoal<Cube3x3> for Phase1Goal {
    type Marker = ByMembership;
    fn goal(&self) -> &PieceSet<Cube3x3> {
        match self {
            Self::Corners => &CORNERS_PHASE_1_GOAL,
            Self::Edges => &EDGES_PHASE_1_GOAL,
        }
    }
    fn table(&self) -> &PruneTable<PieceSet<Cube3x3>> {
        match self {
            Self::Corners => &CORNERS_PHASE_1_TABLE,
            Self::Edges => &EDGES_PHASE_1_TABLE,
        }
    }
}

/// Phase 1: orient every piece and bring the E-slice edges into the E slice, with face turns.
fn phase_1() -> PrunedCombine<Cube3x3> {
    PrunedCombine::<Cube3x3>::new(
        "Phase 1",
        |_| true,
        [Phase1Goal::Corners, Phase1Goal::Edges],
        face_turns(),
    )
}

// Phase 2's goals use `ByIdentity`, because they must tell the pieces of a group apart:
// `ByMembership` would only say which slots hold them. As in Kociemba's own solver, each goal pairs
// a group of pieces with the E-slice edges, so one table knows how both interact: 8! * 4! = 967680
// entries each. Separate tables for the three groups give a lower bound only as good as the
// worst-placed group, which leaves the search to explore far more states.

static CORNERS_AND_E_PHASE_2_GOAL: LazyLock<Mask<Cube3x3>> =
    LazyLock::new(|| Mask::<Cube3x3>::from_pieces(corners().chain(RIM)));

static CORNERS_AND_E_PHASE_2_TABLE: LazyLock<PruneTable<Mask<Cube3x3>>> =
    LazyLock::new(|| PruneTable::from_goal(&CORNERS_AND_E_PHASE_2_GOAL, &domino_turns()));

static EDGES_PHASE_2_GOAL: LazyLock<Mask<Cube3x3>> =
    LazyLock::new(|| Mask::<Cube3x3>::from_pieces(edges()));

static EDGES_PHASE_2_TABLE: LazyLock<PruneTable<Mask<Cube3x3>>> =
    LazyLock::new(|| PruneTable::from_goal(&EDGES_PHASE_2_GOAL, &domino_turns()));

#[derive(Debug)]
enum Phase2Goal {
    CornersAndE,
    /// The U/D-layer edges with the E-slice edges, which is every edge.
    Edges,
}
impl PrunedGoal<Cube3x3> for Phase2Goal {
    type Marker = ByIdentity;
    fn goal(&self) -> &Mask<Cube3x3> {
        match self {
            Self::CornersAndE => &CORNERS_AND_E_PHASE_2_GOAL,
            Self::Edges => &EDGES_PHASE_2_GOAL,
        }
    }
    fn table(&self) -> &PruneTable<Mask<Cube3x3>> {
        match self {
            Self::CornersAndE => &CORNERS_AND_E_PHASE_2_TABLE,
            Self::Edges => &EDGES_PHASE_2_TABLE,
        }
    }
}

/// Phase 2: solve a cube in the domino subgroup, with only the moves that keep it there. A cube
/// outside the subgroup gets [`StepError::UnreachableGoal`](crate::StepError::UnreachableGoal),
/// since every goal asks for oriented pieces and these moves never change orientation.
fn phase_2() -> PrunedCombine<Cube3x3> {
    PrunedCombine::<Cube3x3>::new(
        "Phase 2",
        |puzzle| {
            <Cube3x3 as Puzzle>::Piece::all().all(|slot| {
                [
                    Orientation3x3::Flip(Zn::ZERO),
                    Orientation3x3::Twist(Zn::ZERO),
                    Orientation3x3::Fixed,
                ]
                .contains(&puzzle.orientation_at(slot))
            }) && RIM.iter().all(|&slot| RIM.contains(&puzzle.piece_at(slot)))
        },
        [Phase2Goal::CornersAndE, Phase2Goal::Edges],
        domino_turns(),
    )
}

/// Kociemba's two-phase method.
/// The first call builds the pruning tables, which takes a few seconds.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Kociemba;

impl Method<Cube3x3> for Kociemba {
    fn to_technique(&self) -> Technique<Cube3x3> {
        Technique::new([
            Arc::new(phase_1()) as Arc<dyn Step<Cube3x3>>,
            Arc::new(phase_2()),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Orientation3x3, Puzzle, zn::Zn};

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

    #[test]
    fn domino_turns_are_u_and_d_turns_and_other_half_turns() {
        let names: Vec<String> = domino_turns()
            .algs()
            .iter()
            .map(|alg| alg.iter().map(ToString::to_string).collect())
            .collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(
            sorted,
            ["B2", "D", "D'", "D2", "F2", "L2", "R2", "U", "U'", "U2"],
            "{names:?}"
        );
    }

    #[test]
    fn domino_turns_keep_a_cube_in_the_domino_subgroup() {
        let mut cube = Cube3x3::default();
        for alg in domino_turns().algs() {
            cube = cube.apply(alg);
            assert!(
                in_domino_subgroup(&cube),
                "after {alg:?}
{cube}"
            );
        }
    }
}
