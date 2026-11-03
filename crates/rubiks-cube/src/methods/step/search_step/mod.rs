pub mod memorization;
use std::{
    borrow::Cow,
    collections::{BTreeMap, VecDeque, hash_map::Entry},
    sync::{Arc, Mutex},
};

use rayon::prelude::*;

use crate::{
    AlgSet, Algorithm, BuildContext, BuildStepError, Inv, Mask, PieceSet, PuzzleCondition,
    Solution, Step, StepError,
    error::SearchStepError,
    fast_hash::FxMap,
    methods::{
        context::GLOBAL,
        step::{
            StepBuilder,
            search_step::memorization::{BfsMemo, Cost},
        },
    },
};
use crate::{Indexed, Puzzle};

/// Another thread panicked while deepening the step's memo, so the memo may be inconsistent.
#[derive(Debug, thiserror::Error)]
#[error("the step's memo is unusable: another solve panicked while deepening it")]
struct MemoPoisoned;

/// The pieces a memo brings home, and the sequences it searches with: everything a memo's
/// contents depend on.
type MemoKey<P> = (PieceSet<P>, AlgSet<P>, AlgSet<P>);

/// A memo that several steps can grow, one search at a time.
type SharedMemo<P> = Arc<Mutex<BfsMemo<P>>>;

/// One shared memo per memo key, so steps that search for the same thing with the same
/// sequences grow the same memo, even when built at different times by different methods.
///
/// Statics cannot be generic, so each puzzle type that has methods keeps its own cache in static.
#[derive(Debug, Default)]
pub struct SearchCache<P: Puzzle> {
    memos: Mutex<FxMap<MemoKey<P>, SharedMemo<P>>>,
}

impl<P: Puzzle> SearchCache<P> {
    /// Builds an empty [`BuildContext`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            memos: Mutex::new(FxMap::default()),
        }
    }

    /// The memo that brings `solved` home with these sequences, created empty on first use.
    fn memo(
        &self,
        solved: PieceSet<P>,
        search_algs: &AlgSet<P>,
        free_search_algs: &AlgSet<P>,
    ) -> SharedMemo<P> {
        // Inserting cannot leave the map half-changed, so a panic elsewhere while it was locked
        // leaves it usable.
        let mut memos = self
            .memos
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Arc::clone(
            memos
                .entry((
                    solved.clone(),
                    search_algs.clone(),
                    free_search_algs.clone(),
                ))
                .or_insert_with(|| Arc::new(Mutex::new(BfsMemo::new(solved)))),
        )
    }
}

/// A step that searches for the cheapest way to bring a set of pieces home, using only the
/// move sequences it was given.
///
/// The goal is the `after` [`Mask`]: the pieces that must end in place, and those that must
/// also be oriented. Pieces outside the mask may end anywhere. Before it searches, the step
/// checks its `before` mask and returns [`StepError::InvalidStartingState`] if the puzzle does
/// not meet it.
///
/// The search meets in the middle. A forward search from the puzzle meets a backward search
/// from the goal, which the step keeps in a memo. The memo keeps growing across solves, so
/// later solves are faster, and methods that share this step through an `Arc` share its memo.
#[derive(Debug)]
pub struct SearchStep<P: Puzzle> {
    name: Cow<'static, str>,
    before: PieceSet<P>,
    after: PieceSet<P>,
    search_algs: AlgSet<P>,
    free_search_algs: AlgSet<P>,
    memo: SharedMemo<P>,
}

/// Builder for [`SearchStep`].
#[derive(Debug, Clone)]
pub struct SearchStepBuilder<P: Puzzle> {
    name: Cow<'static, str>,
    before: Option<PieceSet<P>>,
    /// `None` until [`goal`](Self::goal) is called; [`build`](Self::build) then solves every
    /// piece.
    after: Option<PieceSet<P>>,
    search_algs: AlgSet<P>,
    free_search_algs: AlgSet<P>,
}

impl<P: Puzzle> StepBuilder<P> for SearchStepBuilder<P> {
    fn name(&self) -> Cow<'_, str> {
        self.name.clone()
    }

    fn get_prereq(&self) -> Option<PuzzleCondition<P>> {
        self.before.clone().map(std::convert::Into::into)
    }

    fn get_goal(&self) -> Option<PuzzleCondition<P>> {
        self.after.clone().map(std::convert::Into::into)
    }

    fn allowed_sequences(&self) -> Option<AlgSet<P>> {
        Some(
            self.search_algs
                .iter()
                .chain(self.free_search_algs.iter())
                .cloned()
                .collect(),
        )
    }
    fn build_wired(
        &self,
        prerequisite: Option<PuzzleCondition<P>>,
        goal: Option<PuzzleCondition<P>>,
        context: &BuildContext,
    ) -> Result<Arc<dyn Step<P>>, BuildStepError> {
        let mut mut_self = self.clone();
        match prerequisite {
            Some(req) if let Some(pieces) = req.clone().as_pieces() => {
                mut_self.prerequisite(pieces);
            }
            Some(_) => {
                return Err(BuildStepError::InvalidRequisite);
            }

            None => (),
        }

        match goal {
            Some(goal) if let Some(pieces) = goal.clone().as_pieces() => {
                mut_self.goal(pieces);
            }
            Some(_) => {
                return Err(BuildStepError::InvalidGoal);
            }

            None => (),
        }

        Ok(Arc::new(
            mut_self
                .build_in(context)
                .map_err(|e| BuildStepError::Custom(Box::new(e)))?,
        ))
    }
}

impl<P: Puzzle> SearchStepBuilder<P> {
    /// Sets a prerequisite to be checked before solving with the [`SearchStep`] being built.
    pub fn prerequisite(&mut self, pieces: PieceSet<P>) -> &mut Self {
        self.before = Some(pieces);
        self
    }

    /// Defines an [`AlgSet`] of [`Algorithm`]s to be considered free when searching.
    #[must_use]
    pub fn free_algs(&mut self, algs: AlgSet<P>) -> &mut Self {
        self.free_search_algs = algs;
        self
    }

    /// Defines the goal of the [`SearchStep`].
    pub fn goal(&mut self, goal: PieceSet<P>) -> &mut Self {
        self.after = Some(goal);
        self
    }

    /// Builds the [`SearchStep`] using a given [`BuildContext`].
    ///
    /// # Errors
    /// Errors if the goal doesn't guarantee the prerequisite.
    pub fn build_in(&self, cache: &BuildContext) -> Result<SearchStep<P>, SearchStepError<P>> {
        let after = self
            .after
            .clone()
            .unwrap_or_else(|| PieceSet::from_pieces(P::Piece::all()));
        if self.before.as_ref().is_none_or(|b| b.is_subset(&after)) {
            let memo = cache.get_or_create::<SearchCache<P>>().memo(
                after.clone(),
                &self.search_algs,
                &self.free_search_algs,
            );
            Ok(SearchStep {
                name: self.name.clone(),
                before: self.before.clone().unwrap_or_default(),
                after,
                search_algs: self.search_algs.clone(),
                free_search_algs: self.free_search_algs.clone(),
                memo,
            })
        } else {
            Err(SearchStepError::InvalidPrerequisite {
                before: self.before.clone().unwrap_or_default(),
                after,
            })
        }
    }

    /// Builds the [`SearchStep`] using a global [`BuildContext`].
    ///
    /// # Errors
    /// Errors if the goal doesn't guarantee the prerequisite.
    pub fn build(&self) -> Result<SearchStep<P>, SearchStepError<P>> {
        self.build_in(&GLOBAL)
    }
}

/// A state the forward search has reached, its mask, and the sequence that reached it.
type ForwardState<'a, P> = (Mask<P>, P, Option<&'a Algorithm<P>>);

/// A state the forward search has settled.
#[derive(Debug)]
struct Reached<'a, P: Puzzle> {
    cost: Cost,
    via: Option<&'a Algorithm<P>>,
    cube: P,
}

/// The cheapest meeting of the two searches so far.
#[derive(Debug)]
struct Best<P: Puzzle> {
    cost: Cost,
    path: Algorithm<P>,
    end: P,
}

/// Replaces `best` if `cost` beats it, building the path only then.
fn keep_cheaper<P: Puzzle>(
    best: &mut Option<Best<P>>,
    cost: Cost,
    path_and_end: impl FnOnce() -> (Algorithm<P>, P),
) {
    if best.as_ref().is_none_or(|b| cost < b.cost) {
        let (path, end) = path_and_end();
        *best = Some(Best { cost, path, end });
    }
}

impl<P: Puzzle> SearchStep<P> {
    /// Constructor for [`SearchStepBuilder`]. The constructed builder builds a [`SearchStep`] using
    /// all moves of the given [`Puzzle`].
    #[must_use]
    pub fn builder(name: impl Into<Cow<'static, str>>, algs: AlgSet<P>) -> SearchStepBuilder<P> {
        SearchStepBuilder {
            name: name.into(),
            before: None,
            after: None,
            search_algs: algs,
            free_search_algs: AlgSet::new(),
        }
    }

    /// Whether this step and `other` grow the same memo.
    #[cfg(test)]
    pub(crate) fn shares_memo_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.memo, &other.memo)
    }

    /// Meet in the middle: a forward search from `p`, one layer at a time, against the memo's
    /// backward search from the goal. Each round grows whichever side has fewer states to expand.
    fn solve_bfs(&self, p: &mut P) -> Result<Algorithm<P>, StepError> {
        let (_, path) = self.search(p)?;
        Ok(path)
    }

    /// [`Self::solve_bfs`], also returning what the path costs.
    #[expect(
        clippy::significant_drop_tightening,
        reason = "the memo is read and deepened on every round, so the lock is held for the whole search"
    )]
    fn search(&self, p: &mut P) -> Result<(Cost, Algorithm<P>), StepError> {
        let mut memo = self
            .memo
            .lock()
            .map_err(|_memo_error| StepError::Custom(Box::new(MemoPoisoned)))?;
        let mut investigated: FxMap<Mask<P>, Reached<'_, P>> = FxMap::default();
        let mut to_search = BTreeMap::from([(Cost::ZERO, vec![(self.mask(p), p.clone(), None)])]);
        let mut best = None;

        // No path can cost less than the two sides' next costs together, so past that `best` is
        // final. A side with nothing left has seen everything it can reach.
        while let (Some((&cost, level)), Some(memo_cost)) =
            (to_search.first_key_value(), memo.next_cost())
        {
            if best
                .as_ref()
                .is_some_and(|b: &Best<P>| cost.plus(memo_cost) >= b.cost)
            {
                break;
            }

            if memo.frontier_len() <= level.len() {
                memo.deepen(
                    &self.search_algs,
                    &self.free_search_algs,
                    |mask, tail_cost, tail| {
                        if let Some(state) = investigated.get(mask) {
                            keep_cheaper(&mut best, state.cost.plus(tail_cost), || {
                                let mut path = self.path_to(&state.cube, &investigated);
                                path.extend(tail.iter().copied());
                                (path, state.cube.apply(tail))
                            });
                        }
                    },
                );
            } else if let Some((cost, level)) = to_search.pop_first() {
                self.next_level(
                    cost,
                    level,
                    &memo,
                    &mut investigated,
                    &mut to_search,
                    &mut best,
                );
            }
            log::trace!(
                "Step: {}\t forward cost: {cost:?}\t states: {}\t memo frontier: {}",
                self.name(),
                investigated.len(),
                memo.frontier_len()
            );
        }

        let best = best.ok_or(StepError::UnreachableGoal)?;
        *p = best.end;
        Ok((best.cost, best.path))
    }

    /// Settles `level`, then queues the states its sequences reach, each at its own cost.
    fn next_level<'a>(
        &'a self,
        cost: Cost,
        level: Vec<ForwardState<'a, P>>,
        memo: &BfsMemo<P>,
        investigated: &mut FxMap<Mask<P>, Reached<'a, P>>,
        to_search: &mut BTreeMap<Cost, Vec<ForwardState<'a, P>>>,
        best: &mut Option<Best<P>>,
    ) {
        let mut settled = Vec::new();
        for (mask, cube, via) in level {
            if let Entry::Vacant(e) = investigated.entry(mask.clone()) {
                e.insert(Reached {
                    cost,
                    via,
                    cube: cube.clone(),
                });
                if let Some((tail_cost, tail)) = memo.settled(&mask) {
                    keep_cheaper(best, cost.plus(tail_cost), || {
                        let mut path = self.path_to(&cube, investigated);
                        path.extend(tail.iter().copied());
                        (path, cube.apply(tail))
                    });
                }
                settled.push(cube);
            }
        }

        let sequences_by_cost = [
            (cost.after_free(), self.free_search_algs.algs()),
            (cost.after_costly(), self.search_algs.algs()),
        ];
        for (next_cost, sequences) in sequences_by_cost {
            let known = &*investigated;
            let candidates: Vec<_> = settled
                .par_iter()
                .flat_map_iter(|p| {
                    sequences.iter().filter_map(move |s| {
                        let moved = p.apply(s);
                        let mask = self.mask(&moved);
                        (!known.contains_key(&mask)).then_some((p, mask, s, moved))
                    })
                })
                .collect();

            for (parent, mask, sequence, moved) in candidates {
                if let Some((tail_cost, tail)) = memo.settled(&mask) {
                    keep_cheaper(best, next_cost.plus(tail_cost), || {
                        let mut path = self.path_to(parent, investigated);
                        path.extend(sequence.iter().copied());
                        path.extend(tail.iter().copied());
                        (path, moved.apply(tail))
                    });
                }
                to_search
                    .entry(next_cost)
                    .or_default()
                    .push((mask, moved, Some(sequence)));
            }
        }
    }

    /// The moves from the searched state to `cube`, read back through `investigated`.
    fn path_to(&self, cube: &P, investigated: &FxMap<Mask<P>, Reached<'_, P>>) -> Algorithm<P> {
        let mut path = VecDeque::new();
        let mut cube = cube.clone();
        while let Some(sequence) = investigated
            .get(&self.mask(&cube))
            .expect("every state in a level was investigated")
            .via
        {
            cube = cube.apply(&sequence.clone().inv());
            path.push_front(sequence);
        }
        path.into_iter().flatten().copied().collect()
    }

    fn mask(&self, puzzle: &P) -> Mask<P> {
        Mask::<P>::filter_by_piece(puzzle, &self.after)
    }
}

impl<P: Puzzle> Step<P> for SearchStep<P> {
    fn name(&self) -> &str {
        &self.name
    }

    fn can_solve(&self, puzzle: &P) -> bool {
        self.before.applies_to(puzzle)
    }

    fn is_done(&self, puzzle: &P) -> bool {
        self.before.applies_to(puzzle) && self.after.applies_to(puzzle)
    }

    fn solve(&self, p: &mut P) -> Result<Solution<P>, StepError> {
        if !self.can_solve(p) {
            return Err(StepError::InvalidStartingState {
                state: p.to_string(),
            });
        }
        Ok(Solution::single_segment(self.name(), self.solve_bfs(p)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Corner, Cube3x3, Edge, Piece3x3};

    /// A memo grown until it has nothing left knows the true cost of every state, so the two-ended
    /// search must match it.
    #[test]
    fn meeting_in_the_middle_finds_the_minimal_cost() {
        let last_slot = vec![
            Piece3x3::Edge(Edge::Uf),
            Piece3x3::Edge(Edge::Ur),
            Piece3x3::Corner(Corner::Ufr),
        ];
        let last_six_edges = vec![
            Piece3x3::Edge(Edge::Uf),
            Piece3x3::Edge(Edge::Ub),
            Piece3x3::Edge(Edge::Df),
            Piece3x3::Corner(Corner::Ufr),
        ];
        let cases = [
            ("M", "U", last_six_edges.clone()),
            ("U", "M", last_six_edges.clone()),
            ("M U", "", last_six_edges),
            ("R2", "U", last_slot.clone()),
            ("R", "U", last_slot.clone()),
            ("U", "R", last_slot.clone()),
            ("R U", "F", last_slot.clone()),
            ("F2", "U R", last_slot),
        ];
        let mut random = fastrand::Rng::with_seed(1);
        for (costly, free, pieces) in cases {
            let costly_algs = AlgSet::from_static_parts(costly);
            let free_algs = AlgSet::from_static_parts(free);
            let goal = PieceSet::from_pieces(pieces);
            let mut exhausted = BfsMemo::<Cube3x3>::new(goal.clone());
            while exhausted.next_cost().is_some() {
                exhausted.deepen(&costly_algs, &free_algs, |_, _, _| {});
            }
            let moves: Vec<&str> = costly
                .split(' ')
                .chain(free.split(' '))
                .filter(|m| !m.is_empty())
                .collect();
            for _ in 0..60 {
                let scramble = (0..random.usize(0..14))
                    .map(|_| moves[random.usize(..moves.len())])
                    .collect::<Vec<_>>()
                    .join(" ");
                let start = Cube3x3::from_moves(&scramble).unwrap();
                let step = SearchStep::builder("cost", costly_algs.clone())
                    .free_algs(free_algs.clone())
                    .goal(goal.clone())
                    .build_in(&BuildContext::new())
                    .unwrap();
                let (minimal, _) = exhausted.settled(&step.mask(&start)).unwrap();
                let mut cube = start;

                let (cost, path) = step.search(&mut cube).unwrap();

                assert_eq!(cost, minimal, "costly {costly}, free {free}: {scramble}");
                assert!(
                    step.is_done(&cube),
                    "costly {costly}, free {free}: {scramble}"
                );
                assert!(
                    step.is_done(&start.apply(&path)),
                    "costly {costly}, free {free}: {scramble}"
                );
            }
        }
    }

    #[test]
    fn an_opaque_prerequisite_cannot_be_searched_from() {
        let result = SearchStep::<Cube3x3>::builder("Opaque", AlgSet::new()).build_wired(
            Some(PuzzleCondition::opaque()),
            None,
            &BuildContext::new(),
        );

        assert!(
            matches!(result, Err(BuildStepError::InvalidRequisite)),
            "{result:?}"
        );
    }

    #[test]
    fn an_opaque_goal_cannot_be_searched_for() {
        let result = SearchStep::<Cube3x3>::builder("Opaque", AlgSet::new()).build_wired(
            None,
            Some(PuzzleCondition::opaque()),
            &BuildContext::new(),
        );

        assert!(
            matches!(result, Err(BuildStepError::InvalidGoal)),
            "{result:?}"
        );
    }

    #[test]
    fn a_poisoned_memo_is_reported_as_memo_poisoned() {
        let u_turns = AlgSet::from_static_parts("U");
        let step = SearchStep::builder("UF", u_turns)
            .goal(PieceSet::from_pieces([Edge::Uf]))
            .build_in(&BuildContext::new())
            .unwrap();

        let memo = Arc::clone(&step.memo);
        let panicked = std::thread::spawn(move || {
            let _guard = memo.lock().unwrap();
            panic!("poisoning the memo on purpose");
        })
        .join();
        assert!(panicked.is_err(), "the helper thread should have panicked");
        assert!(step.memo.is_poisoned());

        let result = step.solve(&mut Cube3x3::default());

        assert!(
            matches!(&result, Err(StepError::Custom(e)) if e.downcast_ref::<MemoPoisoned>().is_some()),
            "{result:?}"
        );
    }
}
