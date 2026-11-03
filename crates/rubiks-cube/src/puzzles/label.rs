use std::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::Mul,
};

use itertools::Itertools;

use crate::{AlgSet, Indexed, Puzzle, fast_hash::FxSet};

/// A puzzle with optional labels attached to its pieces, as well as optional orientations.
///
/// Generalizes the concept of a partial puzzle - in which the labels are the name of the pieces
/// themselves, and a puzzle that a set of pieces is tracked without regard for their identities -
/// where the labels are anything of unit type, and everything in between.
///
/// For partial puzzles, use [`Mask`](crate::Mask), for pieces tracked with a generic mark, use
/// [`PieceSet`](crate::PieceSet)
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Labeled<P: Puzzle, L: Marker<P>>(Box<[SlotCondition<P, L>]>);

/// A partly defined puzzle state, usually read as a condition: which pieces must sit in their
/// home slots, and which slots must hold an oriented piece.
///
/// [`SearchStep`](crate::SearchStep)s use masks for where they start and what they solve. A mask
/// ignores every piece it does not name.
///
/// ```
/// use rubiks_cube::{Cube3x3, Edge, Mask, Piece3x3};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let uf_solved = Mask::from_pieces([Edge::Uf]);
/// assert!(uf_solved.applies_to(&Cube3x3::from_moves("R")?));
/// assert!(!uf_solved.applies_to(&Cube3x3::from_moves("U")?));
/// # Ok(())
/// # }
/// ```
pub type Mask<P> = Labeled<P, ByIdentity>;

/// A puzzle state with optional orientations, and optional tracking of specific pieces.
///
/// Used for defining steps by marking positions to be solved, as well as to track cubes throughout
/// moves without distinction for its marked pieces.
pub type PieceSet<P> = Labeled<P, ByMembership>;

impl<P: Puzzle> Display for Labeled<P, ByMembership> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let permutation = self
            .0
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| {
                slot.label.map(|()| P::Piece::from_index(index).to_string())
            })
            .collect::<Vec<_>>()
            .join(", ");
        let orientation = self
            .0
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| {
                slot.orient.map(|_| P::Piece::from_index(index).to_string())
            })
            .join(", ");

        write!(f, "Placed: {permutation}\nOriented:{orientation}\n")
    }
}

/// What a [`Mask`] asks of one slot: which piece must sit there, and which orientation the piece
/// there must have. `None` asks nothing.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SlotCondition<P: Puzzle, L: Marker<P>> {
    pub(crate) label: Option<L::Label>,
    pub(crate) orient: Option<P::Orientation>,
}

/// What a [`Labeled`] writes in a slot.
///
/// Implemented by marker types instead of by the label types themselves, so the impls for
/// [`ByIdentity`] and [`ByMembership`] cannot overlap even if some puzzle's piece type is `()`.
pub trait Marker<P: Puzzle> {
    /// The label stored in each slot.
    type Label: Copy + Eq + Hash + Debug;
    /// The label meaning "this slot's own piece belongs here".
    fn home(slot: P::Piece) -> Self::Label;
}

/// Labels are pieces: each slot names the piece that must sit there. [`Mask`] uses it.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ByIdentity;

/// Labels are `()`: a labeled slot's own piece must end up home. Goals use it.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ByMembership;

impl<P: Puzzle> Marker<P> for ByIdentity {
    type Label = P::Piece;
    fn home(slot: P::Piece) -> Self::Label {
        slot
    }
}

impl<P: Puzzle> Marker<P> for ByMembership {
    type Label = ();
    fn home(_slot: P::Piece) -> Self::Label {}
}

impl<P: Puzzle, L: Marker<P>> Default for Labeled<P, L> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Puzzle, L: Marker<P>> Mul<P::Move> for Labeled<P, L> {
    type Output = Self;
    fn mul(self, rhs: P::Move) -> Self::Output {
        self.composed_with(&(P::default() * rhs))
    }
}

impl<P: Puzzle, L: Marker<P>> Labeled<P, L> {
    pub(crate) fn from_fn(condition: impl FnMut(P::Piece) -> SlotCondition<P, L>) -> Self {
        Self(P::Piece::all().map(condition).collect())
    }

    /// A mask that names no piece, so every puzzle meets it. The same as `Mask::default()`.
    #[must_use]
    pub fn new() -> Self {
        Self::from_fn(|_| SlotCondition {
            label: None,
            orient: None,
        })
    }

    /// A mask where each piece in `pieces` must be solved: home and oriented. Same as
    /// <code>[from_pieces_and_orientations](Self::from_pieces_and_orientations)(pieces.clone(),
    /// pieces)</code>.
    pub fn from_pieces<I>(pieces: I) -> Self
    where
        I: IntoIterator<Item: Into<P::Piece>> + Clone,
    {
        Self::from_pieces_and_orientations(pieces.clone(), pieces)
    }

    /// A mask where each piece in `permutations` must sit in its home slot, and the home slot of
    /// each piece in `orientations` must hold an oriented piece.
    ///
    /// The two lists are independent. A piece only in `orientations` asks that whatever piece
    /// sits in its slot be oriented, which is how Roux's corner orientation step (`CO`) says
    /// "orient the last-layer corners, in any order".
    pub fn from_pieces_and_orientations<I1, I2>(permutations: I1, orientations: I2) -> Self
    where
        I1: IntoIterator<Item: Into<P::Piece>>,
        I2: IntoIterator<Item: Into<P::Piece>>,
    {
        let perm_iter: FxSet<<P as Puzzle>::Piece> =
            permutations.into_iter().map(Into::into).collect();
        let orient_iter: FxSet<<P as Puzzle>::Piece> =
            orientations.into_iter().map(Into::into).collect();
        Self::from_fn(|slot| SlotCondition {
            label: perm_iter.contains(&slot).then_some(L::home(slot)),
            orient: orient_iter
                .contains(&slot)
                .then_some(P::default().orientation_at(slot)),
        })
    }

    #[must_use]
    pub(crate) fn filter_by_piece(puzzle: &P, goal: &PieceSet<P>) -> Self {
        Self::from_fn(|slot| {
            let piece = puzzle.piece_at(slot);
            let tracked = goal.condition(piece).label.is_some();
            SlotCondition {
                label: tracked.then_some(L::home(piece)),
                // A goal built by `from_algset` asks for orientation at a slot exactly when every
                // piece the moveset carries through it keeps its orientation, so the condition at
                // a piece's home says whether that piece's orientation matters, wherever it sits.
                // This holds as long as the search moveset never carries a piece whose
                // orientation is free into a slot that asks for one.
                orient: (tracked || goal.condition(piece).orient.is_some())
                    .then(|| puzzle.orientation_at(slot)),
            }
        })
    }

    /// What this mask asks of `slot`.
    #[expect(
        clippy::indexing_slicing,
        reason = "every constructor gives a mask one condition per index of `P::Piece`, and `Indexed::index` returns one of them"
    )]
    pub(crate) fn condition(&self, slot: P::Piece) -> &SlotCondition<P, L> {
        &self.0[slot.index()]
    }

    /// This mask carried along by `puzzle`: each slot takes the condition of the slot its piece
    /// came from, so labels and orientation requirements travel with the pieces. Applying moves
    /// one at a time with `*` gives the same result as composing once with their product.
    pub(crate) fn composed_with(&self, puzzle: &P) -> Self {
        Self::from_fn(|slot| {
            let source = self.condition(puzzle.piece_at(slot));
            SlotCondition {
                label: source.label,
                orient: source.orient.map(|o| o + puzzle.orientation_at(slot)),
            }
        })
    }

    /// A mask where each `(slot, label)` in `labels` asks for that label in that slot, and the
    /// home slot of each piece in `orientations` must hold an oriented piece.
    ///
    /// The two lists are independent. A piece only in `orientations` asks that whatever piece
    /// sits in its slot be oriented, which is how Roux's corner orientation step (`CO`) says
    /// "orient the last-layer corners, in any order".
    pub fn from_labels_and_orientations<I1, I2>(labels: I1, orientations: I2) -> Self
    where
        I1: IntoIterator<Item = (P::Piece, L::Label)>,
        I2: IntoIterator<Item = P::Piece>,
    {
        let perm_iter = FxSet::from_iter(labels);
        let orient_iter: FxSet<<P as Puzzle>::Piece> = FxSet::from_iter(orientations);
        Self::from_fn(|slot| SlotCondition {
            label: perm_iter
                .iter()
                .find_map(|(curr_slot, label)| (*curr_slot == slot).then_some(*label)),
            orient: orient_iter
                .contains(&slot)
                .then_some(P::default().orientation_at(slot)),
        })
    }
}

impl<P: Puzzle> Mask<P> {
    /// Whether `puzzle` meets every condition in this mask.
    #[must_use]
    pub fn applies_to(&self, puzzle: &P) -> bool {
        self.0.iter().zip(P::Piece::all()).all(|(condition, slot)| {
            condition
                .label
                .is_none_or(|piece| puzzle.piece_at(slot) == piece)
                && condition
                    .orient
                    .is_none_or(|orient| puzzle.orientation_at(slot) == orient)
        })
    }
    /// Composes two instances of [`Mask<P>`], marking pieces if they are tracked by both.
    ///
    /// Zeroes the orientations that are tracked - is meant to be used in goal-type values.
    #[must_use]
    pub fn and(&self, other: &Self) -> Self {
        Self::from_pieces_and_orientations(
            P::Piece::all()
                .zip(self.0.iter().zip(other.0.iter()))
                .filter(|&(_, (con_self, con_other))| {
                    con_self.label.is_some() && con_other.label.is_some()
                })
                .map(|(slot, _)| slot),
            P::Piece::all()
                .zip(self.0.iter().zip(other.0.iter()))
                .filter(|&(_, (con_self, con_other))| {
                    con_self.orient.is_some() && con_other.orient.is_some()
                })
                .map(|(slot, _)| slot),
        )
    }
}

impl<P: Puzzle> PieceSet<P> {
    /// Composes two instances of [`PieceSet<P>`], marking pieces if they are tracked by either one.
    ///
    /// Zeroes the orientations that are tracked - is meant to be used in goal-type values.
    #[must_use]
    pub fn or(&self, other: &Self) -> Self {
        Self::from_pieces_and_orientations(
            P::Piece::all()
                .zip(self.0.iter().zip(other.0.iter()))
                .filter(|&(_, (con_self, con_other))| {
                    con_self.label.is_some() || con_other.label.is_some()
                })
                .map(|(slot, _)| slot),
            P::Piece::all()
                .zip(self.0.iter().zip(other.0.iter()))
                .filter(|&(_, (con_self, con_other))| {
                    con_self.orient.is_some() || con_other.orient.is_some()
                })
                .map(|(slot, _)| slot),
        )
    }

    /// Whether every slot this marks, for its piece or its orientation, `other` marks too.
    #[must_use]
    pub fn is_subset(&self, other: &Self) -> bool {
        self.0
            .iter()
            .zip(other.0.iter())
            .all(|(con_self, con_other)| {
                (con_self.label.is_none() || con_other.label.is_some())
                    && (con_self.orient.is_none() || con_other.orient.is_some())
            })
    }

    /// Whether `puzzle` has the slots marked by tracked solved.
    #[must_use]
    pub fn applies_to(&self, puzzle: &P) -> bool {
        self.0.iter().zip(P::Piece::all()).all(|(condition, slot)| {
            condition
                .label
                .is_none_or(|()| puzzle.piece_at(slot) == slot)
                && condition
                    .orient
                    .is_none_or(|orient| puzzle.orientation_at(slot) == orient)
        })
    }

    /// Constructs a [`PieceSet Puzzle`](PieceSet<P>) by checking which pieces cannot be moved using
    /// only an algset.
    #[expect(
        clippy::indexing_slicing,
        reason = "Out of bounds slice is an unrecoverable state"
    )]
    #[must_use]
    pub fn from_algset(algset: &AlgSet<P>) -> Self {
        let mut mask = Mask::<P>::from_pieces(P::Piece::all());
        let mut changed = true;
        while changed {
            changed = false;
            for alg in algset.algs() {
                let moved = alg.iter().fold(mask.clone(), |p, m| p * *m);
                for p in P::Piece::all() {
                    if mask.condition(p).label != moved.condition(p).label {
                        mask.0[p.index()].label = None;
                        changed = true;
                    }
                    if mask.condition(p).orient != moved.condition(p).orient {
                        mask.0[p.index()].orient = None;
                        changed = true;
                    }
                }
            }
        }
        let marked = mask
            .0
            .iter()
            .map(|s| SlotCondition {
                label: s.label.is_some().then_some(()),
                orient: s.orient,
            })
            .collect();
        Self(marked)
    }
}

#[cfg(test)]
mod test {
    use crate::{Corner, Cube3x3, Piece3x3};

    use super::*;

    #[test]
    fn applies_to_composes_correctly_on_full_cube() {
        let cube = Cube3x3::scrambled_with_seed(0);
        let mask = Mask::from_pieces(Piece3x3::all());

        assert!(
            mask.composed_with(&cube).applies_to(&cube),
            "{}\n\n{}",
            mask.composed_with(&cube),
            cube
        );
    }

    fn random_states() -> impl Iterator<Item = Cube3x3> {
        (0..20).map(Cube3x3::scrambled_with_seed)
    }

    #[test]
    fn the_key_keeps_every_orientation_the_goal_moveset_preserves_wherever_the_piece_sits() {
        // `U R L` flips no edge, so every edge's flip decides whether the goal is met, including
        // a stray edge parked in DF or DB, the two edge slots where the goal names a piece.
        let goal = PieceSet::from_algset(&AlgSet::from_static_parts("U R L"));
        for cube in random_states() {
            let key = Mask::filter_by_piece(&cube, &goal);
            for slot in Piece3x3::all() {
                if matches!(slot, Piece3x3::Edge(_)) {
                    assert!(
                        key.condition(slot).orient.is_some(),
                        "{slot:?} left its flip out of the key:\n{cube}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_key_keeps_no_orientation_the_goal_moveset_changes() {
        // `U R M r` twists or flips every corner and edge it moves, so only the pieces it leaves
        // alone, which the key names, keep their orientation in it.
        let goal = PieceSet::from_algset(&AlgSet::from_static_parts("U R M r"));
        for cube in random_states() {
            let key = Mask::filter_by_piece(&cube, &goal);
            for slot in Piece3x3::all() {
                if matches!(slot, Piece3x3::Corner(_) | Piece3x3::Edge(_)) {
                    let condition = key.condition(slot);
                    assert_eq!(
                        condition.orient.is_some(),
                        condition.label.is_some(),
                        "{slot:?}:\n{cube}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_label_goes_on_its_own_slot_only() {
        let ufr = Piece3x3::Corner(Corner::Ufr);
        let dbr = Piece3x3::Corner(Corner::Dbr);
        let mask = Mask::<Cube3x3>::from_labels_and_orientations([(ufr, dbr)], []);

        assert_eq!(mask.condition(ufr).label, Some(dbr));
        assert_eq!(mask.condition(dbr).label, None);
    }

    #[test]
    fn a_set_is_a_subset_only_of_sets_marking_at_least_its_slots() {
        let ufr = Piece3x3::Corner(Corner::Ufr);
        let none: [Piece3x3; 0] = [];
        let solved = PieceSet::<Cube3x3>::from_pieces([ufr]);
        let home_only = PieceSet::<Cube3x3>::from_pieces_and_orientations([ufr], none);
        let oriented_only = PieceSet::<Cube3x3>::from_pieces_and_orientations(none, [ufr]);

        assert!(PieceSet::new().is_subset(&solved));
        assert!(home_only.is_subset(&solved));
        assert!(!solved.is_subset(&PieceSet::new()));
        assert!(!oriented_only.is_subset(&home_only));
    }

    #[test]
    fn fb_or_sb_is_f2b() {
        let fb = PieceSet::from_algset(&AlgSet::from_static_parts("R U M"));
        let sb = PieceSet::from_algset(&AlgSet::from_static_parts("L U M"));
        assert!(
            fb.or(&sb) == PieceSet::from_algset(&AlgSet::from_static("M\nU\nR U R' U R U2 R'")),
            "FB:\n{fb}\nSB:\n{sb}\nF2B:\n\tExpected:\n\t{}\n\tFound:\n\t{}",
            PieceSet::from_algset(&AlgSet::from_static_parts("M U"))
                .to_string()
                .lines()
                .join("\n\t"),
            fb.or(&sb).to_string().lines().join("\n\t")
        );
    }
}
