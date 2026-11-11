use std::collections::BTreeMap;

use crate::{AlgSet, Algorithm, Inv, Mask, PieceSet, Puzzle, fast_hash::FxMap};

/// What a path costs: compared by costly sequences first, then by free sequences.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Cost {
    costly: u32,
    free: u32,
}

impl Cost {
    pub(crate) const ZERO: Self = Self { costly: 0, free: 0 };

    pub(crate) const fn after_costly(self) -> Self {
        Self {
            costly: self.costly + 1,
            free: self.free,
        }
    }

    pub(crate) const fn after_free(self) -> Self {
        Self {
            costly: self.costly,
            free: self.free + 1,
        }
    }

    pub(crate) const fn plus(self, other: Self) -> Self {
        Self {
            costly: self.costly + other.costly,
            free: self.free + other.free,
        }
    }
}

/// A state reached but not settled: its mask, the state, and its way back to the goal.
type Pending<P> = (Mask<P>, P, Algorithm<P>);

#[derive(Debug, Eq, PartialEq)]
pub struct BfsMemo<P: Puzzle> {
    /// Every state is filtered through this, so memo keys match the forward search's masks.
    goal: PieceSet<P>,
    /// The settled states: each cost here is final.
    memorization: FxMap<Mask<P>, (Cost, Algorithm<P>)>,
    /// By cost: a cheaper path to one of these may still turn up.
    to_deepen: BTreeMap<Cost, Vec<Pending<P>>>,
}

impl<P: Puzzle> BfsMemo<P> {
    pub fn new(goal: PieceSet<P>) -> Self {
        let mask = Mask::filter_by_piece(&P::default(), &goal);
        Self {
            goal,
            memorization: FxMap::default(),
            to_deepen: BTreeMap::from([(Cost::ZERO, vec![(mask, P::default(), Algorithm::new())])]),
        }
    }

    #[cfg(test)]
    pub(crate) fn solution(&self, mask: &Mask<P>) -> Option<&Algorithm<P>> {
        self.memorization.get(mask).map(|(_, solution)| solution)
    }

    /// The settled cost and solution of `mask`.
    pub(crate) fn settled(&self, mask: &Mask<P>) -> Option<(Cost, &Algorithm<P>)> {
        self.memorization
            .get(mask)
            .map(|(cost, solution)| (*cost, solution))
    }

    /// Number of states the next [`Self::deepen`] starts from; 0 once every reachable state is
    /// memorized.
    pub(crate) fn frontier_len(&self) -> usize {
        self.to_deepen
            .first_key_value()
            .map_or(0, |(_, states)| states.len())
    }

    /// The cost the next [`Self::deepen`] settles; `None` once every reachable state is memorized.
    pub(crate) fn next_cost(&self) -> Option<Cost> {
        self.to_deepen.first_key_value().map(|(&cost, _)| cost)
    }

    /// Deepens until every state of at most `depth + 1` costly sequences is settled.
    #[cfg(test)]
    pub(crate) fn search_to(
        &mut self,
        depth: u32,
        possible_sequences: &AlgSet<P>,
        free_possible_sequences: &AlgSet<P>,
    ) {
        while self
            .next_cost()
            .is_some_and(|cost| cost.costly <= depth + 1)
        {
            self.deepen(possible_sequences, free_possible_sequences, |_, _, _| {});
        }
    }

    /// Settles the cheapest layer and reaches out from it by one sequence. `seen` gets every state
    /// settled or reached, so the forward search can check each for a meeting.
    pub(crate) fn deepen(
        &mut self,
        possible_sequences: &AlgSet<P>,
        free_possible_sequences: &AlgSet<P>,
        mut seen: impl FnMut(&Mask<P>, Cost, &Algorithm<P>),
    ) {
        let Some((cost, layer)) = self.to_deepen.pop_first() else {
            return;
        };
        for (mask, p, solution) in layer {
            if self.memorization.contains_key(&mask) {
                continue;
            }
            seen(&mask, cost, &solution);
            self.memorization.insert(mask.clone(), (cost, solution));
            for sequence in free_possible_sequences.algs() {
                if let Some(state) = self.memorize(&mask, &p, sequence) {
                    seen(&state.0, cost.after_free(), &state.2);
                    self.to_deepen
                        .entry(cost.after_free())
                        .or_default()
                        .push(state);
                }
            }
            for sequence in possible_sequences.algs() {
                if let Some(state) = self.memorize(&mask, &p, sequence) {
                    seen(&state.0, cost.after_costly(), &state.2);
                    self.to_deepen
                        .entry(cost.after_costly())
                        .or_default()
                        .push(state);
                }
            }
        }
    }

    /// The state `sequence` leads back from, with its mask and solution, if it is not settled yet.
    fn memorize(
        &self,
        parent_mask: &Mask<P>,
        parent: &P,
        sequence: &Algorithm<P>,
    ) -> Option<Pending<P>> {
        // Walk away from the goal by undoing `sequence`, so the way back applies `sequence` as
        // written, then the parent's path.
        let moved = sequence
            .iter()
            .rev()
            .fold(parent.clone(), |puzzle, m| puzzle * m.inv());
        let mask = Mask::filter_by_piece(&moved, &self.goal);
        if self.memorization.contains_key(&mask) {
            return None;
        }
        let solution = sequence
            .iter()
            .chain(
                &self
                    .memorization
                    .get(parent_mask)
                    .expect("a parent is memorized before its children")
                    .1,
            )
            .copied()
            .collect();
        Some((mask, moved, solution))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::*;
    use fastrand::Rng;

    #[test]
    fn each_kind_of_sequence_adds_to_its_own_count() {
        let cost = Cost { costly: 2, free: 3 };
        assert_eq!(cost.after_free(), Cost { costly: 2, free: 4 });
        assert_eq!(cost.after_costly(), Cost { costly: 3, free: 3 });
    }

    #[test]
    fn one_costly_sequence_outweighs_any_number_of_free_ones() {
        assert!(
            Cost {
                costly: 0,
                free: 100
            } < Cost { costly: 1, free: 0 }
        );
    }

    #[test]
    fn applying_mask_matches_filtering_through() {
        let mut random = Rng::with_seed(3);
        for _ in 0..100 {
            let pieces = Piece3x3::all()
                .filter(|_| random.bool())
                .collect::<Vec<Piece3x3>>();
            assert_eq!(
                Mask::<Cube3x3>::filter_by_piece(
                    &Cube3x3::default(),
                    &PieceSet::<Cube3x3>::from_pieces_and_orientations(
                        pieces.clone(),
                        pieces.clone()
                    )
                ),
                Mask::from_pieces_and_orientations(pieces.clone(), pieces)
            );
        }
    }

    /// A memo over the whole cube, searched with R and U moves only.
    fn r_u_memo(depth: u32) -> (BfsMemo<Cube3x3>, PieceSet<Cube3x3>) {
        let goal = PieceSet::<Cube3x3>::from_pieces(Piece3x3::all());
        let allowed = AlgSet::from_moves("R R' R2 U U' U2").unwrap();
        let mut memo = BfsMemo::new(goal.clone());
        memo.search_to(depth, &allowed, &AlgSet::new());
        (memo, goal)
    }

    #[test]
    fn memo_solution_undoes_one_move() {
        let (memo, goal) = r_u_memo(1);
        let r = Cube3x3::from_moves("R").unwrap();
        assert_eq!(
            memo.solution(&Mask::filter_by_piece(&r, &goal)),
            Some(&"R'".parse().unwrap())
        );
    }

    #[test]
    fn memo_solution_undoes_the_whole_path() {
        let (memo, goal) = r_u_memo(2);
        let r_u = Cube3x3::from_moves("R U").unwrap();
        assert_eq!(
            memo.solution(&Mask::filter_by_piece(&r_u, &goal)),
            Some(&"U' R'".parse().unwrap())
        );
    }

    /// The FR edge passes through UR on the way; the memo key must not keep UR's orientation.
    #[test]
    fn memo_keys_match_forward_masks_after_a_piece_leaves_home() {
        let goal = PieceSet::<Cube3x3>::from_pieces([Edge::Fr]);
        let allowed = AlgSet::from_moves("R' U'").unwrap();
        let mut memo = BfsMemo::new(goal.clone());
        memo.search_to(1, &allowed, &AlgSet::new());
        let r_u = Cube3x3::from_moves("R U").unwrap();
        assert_eq!(
            memo.solution(&Mask::filter_by_piece(&r_u, &goal)),
            Some(&"U' R'".parse().unwrap())
        );
    }

    /// Sune's inverse is not an allowed sequence, so the memo must solve with Sune as written.
    #[test]
    fn memo_solutions_use_the_allowed_sequences_as_written() {
        let goal = PieceSet::<Cube3x3>::from_pieces(Piece3x3::all());
        let sune = "R U R' U R U2 R'";
        let allowed = AlgSet::from_str(sune).unwrap();
        let mut memo = BfsMemo::new(goal.clone());
        memo.search_to(0, &allowed, &AlgSet::new());
        let before_sune = Cube3x3::from_moves(sune).unwrap().inv();
        assert_eq!(
            memo.solution(&Mask::filter_by_piece(&before_sune, &goal)),
            Some(&sune.parse().unwrap())
        );
    }

    /// The `U'` state is reachable by the costly `U` and by the free `U`; the costly one is tried
    /// first. It still costs nothing, so what `R` reaches from it costs 1 and is in the memo after
    /// level 0 is expanded.
    #[test]
    fn free_sequences_are_closed_before_costly_ones_claim_their_states() {
        let goal = PieceSet::<Cube3x3>::from_pieces(Piece3x3::all());
        let allowed = AlgSet::from_moves("U R").unwrap();
        let free = AlgSet::from_moves("U").unwrap();
        let mut memo = BfsMemo::new(goal.clone());
        memo.search_to(0, &allowed, &free);
        let before_r_u = Cube3x3::from_moves("R U").unwrap().inv();
        assert_eq!(
            memo.solution(&Mask::filter_by_piece(&before_r_u, &goal)),
            Some(&"R U".parse().unwrap())
        );
    }

    /// The goal tracks the FR edge wherever it goes, orientation included. After `F` another
    /// edge sits in the FR slot, and its orientation is not part of the goal.
    #[test]
    fn mask_ignores_orientation_of_other_pieces_in_a_goal_pieces_home_slot() {
        let fr = Piece3x3::Edge(Edge::Fr);
        let goal = PieceSet::<Cube3x3>::from_pieces([fr]);
        let f = Cube3x3::from_moves("F").unwrap();
        assert_ne!(f.piece_at(fr), fr);
        assert_eq!(Mask::filter_by_piece(&f, &goal).condition(fr).orient, None);
    }

    #[test]
    fn free_sequences_are_memorized_at_depth_zero() {
        let goal = PieceSet::<Cube3x3>::from_pieces(Piece3x3::all());
        let allowed = AlgSet::from_moves("R").unwrap();
        let free = AlgSet::from_moves("U'").unwrap();
        let mut memo = BfsMemo::new(goal.clone());
        memo.search_to(0, &allowed, &free);
        let u = Cube3x3::from_moves("U").unwrap();
        assert_eq!(
            memo.solution(&Mask::filter_by_piece(&u, &goal)),
            Some(&"U'".parse().unwrap())
        );
    }

    #[test]
    fn fb_is_unchanged_by_sb_moves_when_filtered() {
        let mut cube = Cube3x3::default();
        let fb_pieces = [
            Piece3x3::Center(crate::Center::L),
            Piece3x3::Corner(crate::Corner::Dfl),
            Piece3x3::Corner(crate::Corner::Dbl),
            Piece3x3::Edge(crate::Edge::Fl),
            Piece3x3::Edge(crate::Edge::Dl),
            Piece3x3::Edge(crate::Edge::Bl),
        ];
        let sb_moves = [
            puzzles::cube3x3::moves::Move3x3::new(
                puzzles::cube3x3::moves::MovablePart::Face(puzzles::cube3x3::pieces::Face::U),
                puzzles::cube3x3::moves::MoveModifier::Clockwise,
            ),
            puzzles::cube3x3::moves::Move3x3::new(
                puzzles::cube3x3::moves::MovablePart::Face(puzzles::cube3x3::pieces::Face::R),
                puzzles::cube3x3::moves::MoveModifier::Clockwise,
            ),
            puzzles::cube3x3::moves::Move3x3::new(
                puzzles::cube3x3::moves::MovablePart::Slice(puzzles::cube3x3::pieces::Slice::M),
                puzzles::cube3x3::moves::MoveModifier::Clockwise,
            ),
            puzzles::cube3x3::moves::Move3x3::new(
                puzzles::cube3x3::moves::MovablePart::Wide(puzzles::cube3x3::pieces::Face::R),
                puzzles::cube3x3::moves::MoveModifier::Clockwise,
            ),
        ];

        let fb_filter = PieceSet::<Cube3x3>::from_pieces_and_orientations(fb_pieces, fb_pieces);
        let mut rng = Rng::with_seed(0);

        for _ in 0..100 {
            cube *= sb_moves[rng.usize(0..4)];
            assert_eq!(
                fb_filter,
                PieceSet::<Cube3x3>::filter_by_piece(&cube, &fb_filter)
            );
        }
        cube = cube.apply_moves("L").unwrap();

        assert_ne!(
            fb_filter,
            PieceSet::<Cube3x3>::filter_by_piece(&cube, &fb_filter)
        );
    }
}
