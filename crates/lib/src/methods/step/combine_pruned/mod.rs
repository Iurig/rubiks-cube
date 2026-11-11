mod prune_table;

pub use prune_table::PruneTable;
use rayon::prelude::*;

use crate::{AlgSet, Algorithm, Labeled, Marker, Puzzle, Solution, Step, StepError};

use std::{
    borrow::Cow,
    fmt::Debug,
    hash::Hash,
    ops::ControlFlow,
    sync::atomic::{AtomicU16, Ordering},
};

pub struct PrunedCombine<P: Puzzle> {
    name: Cow<'static, str>,
    steps: Vec<Box<dyn DistanceStep<P>>>,
    ready_to_solve: Box<dyn Fn(&P) -> bool + Send + Sync>,
    moveset: AlgSet<P>,
    /// `may_follow[a][b]`: whether the search tries sequence `b` of the moveset right after
    /// sequence `a`. See [`may_follow`].
    may_follow: Vec<Vec<bool>>,
}

impl<P: Puzzle> Debug for PrunedCombine<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrunedCombine")
            .field("name", &self.name)
            .field("steps", &self.steps)
            .field("moveset", &self.moveset)
            .field("may_follow", &self.may_follow)
            .finish_non_exhaustive()
    }
}

/// One goal of a [`PrunedCombine`]: how many moveset sequences a puzzle is from it.
pub trait DistanceStep<P: Puzzle>: Debug + Send + Sync {
    /// How many moveset sequences `puzzle` is from the goal, or `None` if no sequence reaches
    /// it.
    fn distance_from_moves(&self, puzzle: &P) -> Option<u8>;
}

/// A goal with a table of how far each state is from it. Every `PrunedGoal` is a
/// [`DistanceStep`] through the impl below, which is the only code that reads the table, so the
/// table is always read with the same key it was filled with.
pub trait PrunedGoal<P: Puzzle>: Debug + Send + Sync {
    /// The labels the goal uses: [`ByIdentity`](crate::ByIdentity) when the table must tell pieces
    /// apart, [`ByMembership`](crate::ByMembership) when it only needs to know which slots hold
    /// them.
    type Marker: Marker<P, Label: Send + Sync> + Clone + Eq + Hash + Debug + Send + Sync;

    /// The goal the table was built from.
    fn goal(&self) -> &Labeled<P, Self::Marker>;

    /// The table [`PruneTable::from_goal`] built from [`goal`](Self::goal).
    fn table(&self) -> &PruneTable<Labeled<P, Self::Marker>>;
}

impl<P: Puzzle, T: PrunedGoal<P>> DistanceStep<P> for T {
    fn distance_from_moves(&self, puzzle: &P) -> Option<u8> {
        // The table holds the goal carried along by sequences of moves, so the key is the goal
        // carried along by the whole puzzle. `filter_by_piece` would not do: it keeps some
        // orientation requirements on fixed slots, which builds keys the table never stored.
        self.table()
            .get_distance(&self.goal().composed_with(puzzle))
    }
}

impl<P: Puzzle> PrunedCombine<P> {
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        ready_to_solve: impl Fn(&P) -> bool + Send + Sync + 'static,
        iter: impl IntoIterator<Item = impl DistanceStep<P> + 'static>,
        moveset: AlgSet<P>,
    ) -> Self {
        Self {
            name: name.into(),
            steps: iter
                .into_iter()
                .map(|s| Box::new(s) as Box<dyn DistanceStep<P>>)
                .collect(),
            ready_to_solve: Box::new(ready_to_solve),
            may_follow: may_follow(&moveset),
            moveset,
        }
    }
}

/// For each pair of sequences `(a, b)` of `moveset`, whether a search should try `b` right after
/// `a`. It should not when `a b` does no more than something shorter, or than the same pair in
/// the other order:
///
/// - `a b` leaves the puzzle as it was (`U U'`), or as one sequence of the moveset does (`U U` is
///   `U2`): the shorter path reaches the same state, at a smaller depth.
/// - `a` and `b` commute (`U D` is `D U`) and `b` comes before `a` in the moveset: only the order
///   with the earlier sequence first is tried.
///
/// Neither rule loses a state. Among the shortest paths to a state, the one that comes first in
/// moveset order has no pair either rule rejects: the first rule would give a shorter path, and
/// the second the same path with an earlier sequence first.
fn may_follow<P: Puzzle>(moveset: &AlgSet<P>) -> Vec<Vec<bool>> {
    let algs = moveset.algs();
    let alone: Vec<P> = algs.iter().map(|alg| P::default().apply(alg)).collect();

    alone
        .iter()
        .zip(algs)
        .enumerate()
        .map(|(a_index, (after_a, a))| {
            alone
                .iter()
                .zip(algs)
                .enumerate()
                .map(|(b_index, (after_b, b))| {
                    let a_then_b = after_a.apply(b);
                    let shortens = a_then_b == P::default() || alone.contains(&a_then_b);
                    let commutes_out_of_order = b_index < a_index && a_then_b == after_b.apply(a);
                    !shortens && !commutes_out_of_order
                })
                .collect()
        })
        .collect()
}

impl<P: Puzzle> Step<P> for PrunedCombine<P> {
    fn name(&self) -> &str {
        &self.name
    }

    fn can_solve(&self, puzzle: &P) -> bool {
        (self.ready_to_solve)(puzzle)
    }

    fn is_done(&self, puzzle: &P) -> bool {
        self.steps
            .iter()
            .all(|s| s.distance_from_moves(puzzle) == Some(0))
    }

    #[expect(
        clippy::panic_in_result_fn,
        reason = "panics only on a broken invariant, errors only on unsolvable input"
    )]
    fn solve(&self, puzzle: &mut P) -> Result<crate::Solution<P>, crate::StepError> {
        /// The largest distance any step reports: a lower bound on the moves left, because
        /// every step must be done at the end. `None` when some step cannot reach its goal.
        fn estimate<P: Puzzle>(steps: &[Box<dyn DistanceStep<P>>], puzzle: &P) -> Option<u8> {
            steps.iter().try_fold(0, |max, s| {
                s.distance_from_moves(puzzle).map(|d| max.max(d))
            })
        }

        /// One depth-first pass that never goes past `bound`, counting the sequences already
        /// applied (`depth`) plus the estimate. `last` is the index of the sequence applied just
        /// before, and `path` holds every move applied so far. `Break` means `path` now
        /// solves every step. `Continue(Some(f))` is the smallest total over `bound` it
        /// saw, the next bound to try; `Continue(None)` means no branch can lead to the
        /// goal.
        fn search<P: Puzzle>(
            combine: &PrunedCombine<P>,
            last: Option<usize>,
            puzzle: &P,
            depth: u8,
            bound: u8,
        ) -> ControlFlow<Algorithm<P>, Option<u8>> {
            let Some(estimate) = estimate(&combine.steps, puzzle) else {
                return ControlFlow::Continue(None);
            };
            let Some(total) = depth.checked_add(estimate) else {
                return ControlFlow::Continue(None);
            };
            if total > bound {
                return ControlFlow::Continue(Some(total));
            }
            if estimate == 0 {
                return ControlFlow::Break(Algorithm::new());
            }
            let Some(next_depth) = depth.checked_add(1) else {
                return ControlFlow::Continue(None);
            };
            let next_bound = AtomicU16::new(u16::MAX);
            let allowed = last.and_then(|l| combine.may_follow.get(l));
            let path = combine
                .moveset
                .algs()
                .par_iter()
                .enumerate()
                .filter(|(index, _)| allowed.is_none_or(|row| row.get(*index) != Some(&false)))
                .find_map_first(|(index, alg)| {
                    match search(combine, Some(index), &puzzle.apply(alg), next_depth, bound) {
                        ControlFlow::Break(rest) => {
                            let mut path = Algorithm::new();
                            path.extend_from(alg);
                            path.extend_from(&rest);
                            Some(path)
                        }
                        ControlFlow::Continue(Some(f)) => {
                            next_bound.fetch_min(u16::from(f), Ordering::Relaxed);
                            None
                        }
                        ControlFlow::Continue(None) => None,
                    }
                });
            path.map_or_else(
                || ControlFlow::Continue(u8::try_from(next_bound.into_inner()).ok()),
                ControlFlow::Break,
            )
        }

        let mut bound = estimate(&self.steps, puzzle).ok_or(StepError::UnreachableGoal)?;
        let path = loop {
            let result = search(self, None, puzzle, 0, bound);
            match result {
                ControlFlow::Break(path) => break path,
                ControlFlow::Continue(Some(next)) => {
                    assert!(next > bound);
                    bound = next;
                }
                ControlFlow::Continue(None) => return Err(StepError::UnreachableGoal),
            }
        };
        *puzzle = puzzle.apply(&path);
        Ok(Solution::single_segment(self.name(), path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cube3x3, fast_hash::FxSet};

    /// Every face, clockwise, counterclockwise, and double: 18 single moves.
    fn face_turns() -> AlgSet<Cube3x3> {
        AlgSet::from_static_parts("U D F B L R")
    }

    /// Whether `second` may follow `first` in the `may_follow` table of `moveset`, both written
    /// as one move.
    fn follows(moveset: &AlgSet<Cube3x3>, first: &str, second: &str) -> bool {
        let index = |name: &str| {
            moveset
                .algs()
                .iter()
                .position(|alg| alg.iter().map(ToString::to_string).collect::<String>() == name)
                .unwrap()
        };
        may_follow(moveset)[index(first)][index(second)]
    }

    #[test]
    fn a_face_never_follows_itself() {
        let moveset = face_turns();
        for first in ["U", "U'", "U2"] {
            for second in ["U", "U'", "U2"] {
                assert!(!follows(&moveset, first, second), "{first} {second}");
            }
        }
    }

    #[test]
    fn opposite_faces_follow_each_other_in_one_order_only() {
        let moveset = face_turns();
        for (first, second) in [("U", "D"), ("U'", "D2"), ("R", "L'"), ("F2", "B")] {
            assert!(
                follows(&moveset, first, second) != follows(&moveset, second, first),
                "{first} {second}"
            );
        }
    }

    #[test]
    fn faces_that_do_not_commute_follow_each_other_both_ways() {
        let moveset = face_turns();
        for (first, second) in [("U", "R"), ("R'", "F2"), ("D2", "L")] {
            assert!(follows(&moveset, first, second), "{first} {second}");
            assert!(follows(&moveset, second, first), "{second} {first}");
        }
    }

    /// With only `R` in the moveset, `R R` is the only way to reach `R2`, so it can't be pruned.
    #[test]
    fn a_sequence_follows_itself_when_repeating_it_reaches_a_new_state() {
        let moveset = AlgSet::<Cube3x3>::from_static("R");
        assert_eq!(may_follow(&moveset), [[true]]);
    }

    /// The pruning must not lose a state: up to three moves, the paths `may_follow` allows reach
    /// every state that unpruned paths reach.
    #[test]
    fn pruned_paths_reach_every_state_within_three_moves() {
        let moveset = face_turns();
        let table = may_follow(&moveset);
        let algs = moveset.algs();
        let mut all: FxSet<Cube3x3> = FxSet::from_iter([Cube3x3::default()]);
        let mut pruned = all.clone();
        let mut all_frontier = vec![Cube3x3::default()];
        let mut pruned_frontier = vec![(Cube3x3::default(), None::<usize>)];
        for _ in 0..3 {
            all_frontier = all_frontier
                .iter()
                .flat_map(|cube| algs.iter().map(|alg| cube.apply(alg)))
                .collect();
            all.extend(all_frontier.iter().copied());
            pruned_frontier = pruned_frontier
                .iter()
                .flat_map(|&(cube, last)| {
                    let table = &table;
                    algs.iter()
                        .enumerate()
                        .filter(move |&(index, _)| last.is_none_or(|l| table[l][index]))
                        .map(move |(index, alg)| (cube.apply(alg), Some(index)))
                })
                .collect();
            pruned.extend(pruned_frontier.iter().map(|&(cube, _)| cube));
        }
        assert_eq!(pruned.len(), all.len());
    }
}
