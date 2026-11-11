use std::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::{Add, Mul, MulAssign},
    str::FromStr,
};

use crate::{Algorithm, Cube3x3, Move3x3, ParseMoveError, SolveError, indexed::Indexed};

pub mod algorithm;
pub mod algset;
pub mod cube3x3;
pub mod label;

/// A twisty puzzle the solver can work on. [`Cube3x3`](crate::Cube3x3) is the only one so far.
///
/// Import this trait to call its methods on a cube, such as [`is_solved`](Puzzle::is_solved).
/// A puzzle state is a value; applying a move with `*` returns the new state.
///
/// # Laws
///
/// Generic code, such as a [`Mask`](crate::Mask), relies on every implementation keeping these:
///
/// 1. `Self::default()` is the solved state, and the identity for moves.
/// 2. A move acts the same way on every state. For any state `p`, move `m`, and slot `s`, let
///    `action = Self::default() * m` and `src = action.piece_at(s)`, the slot `m` moves into `s`.
///    Then `(p * m).piece_at(s) == p.piece_at(src)`, and `(p * m).orientation_at(s) ==
///    p.orientation_at(src) + action.orientation_at(s)`.
///
/// So reading `piece_at` and `orientation_at` at every slot of `Self::default() * m` gives
/// everything `m` does to any state.
pub trait Puzzle:
    Default
    + Mul<Self::Move, Output = Self>
    + Debug
    + Display
    + Clone
    + Eq
    + Hash
    + Send
    + Sync
    + 'static
{
    /// One piece of the puzzle. A piece also names its home slot.
    type Piece: Copy + Eq + Debug + Hash + 'static + Send + Sync + Indexed + Display;
    /// How a piece is oriented in its slot. One type covers every kind of piece, so a puzzle with
    /// several kinds, such as the cube's corners and edges, uses an enum with one variant per kind.
    type Orientation: Debug
        + Eq
        + Hash
        + Clone
        + Copy
        + Send
        + Sync
        + Add<Output = Self::Orientation>;
    /// One move of the puzzle, such as `R'` on the cube.
    type Move: crate::Inv
        + Eq
        + Ord
        + Copy
        + 'static
        + Display
        + Debug
        + Send
        + Sync
        + Hash
        + Indexed
        + FromStr<Err = ParseMoveError>;

    /// The slot where `piece` sits now.
    fn slot_of(&self, piece: Self::Piece) -> Self::Piece;

    /// Whether the puzzle is solved, ignoring how the whole puzzle is rotated.
    #[must_use]
    fn is_solved(&self) -> bool;

    /// The piece sitting in `slot` now. `piece_at(slot) == slot` when that piece is home.
    fn piece_at(&self, slot: Self::Piece) -> Self::Piece;

    /// The orientation of the piece in `slot`. It is oriented when this equals the solved
    /// state's orientation at `slot`.
    fn orientation_at(&self, slot: Self::Piece) -> Self::Orientation;

    /// A random scrambled state. The same seed always gives the same state.
    #[must_use]
    fn scrambled_with_seed(seed: u64) -> Self;

    /// A random scrambled state from a random seed.
    #[must_use]
    fn scrambled() -> Self {
        Self::scrambled_with_seed(fastrand::u64(..))
    }

    /// The scramble itself in algorithm form. The same seed always returns the same scramble.
    ///
    /// # Errors
    /// Errors if the method for finding a scramble from a random state fails.
    fn scramble_with_seed(seed: u64) -> Result<Algorithm<Self>, SolveError>;

    /// The scramble itself in algorithm form from a random seed.
    ///
    /// # Errors
    /// Errors if the method for finding a scramble from a random state fails.
    fn scramble() -> Result<Algorithm<Self>, SolveError> {
        Self::scramble_with_seed(fastrand::u64(..))
    }

    /// The state after applying the moves of `alg` to this one, in order.
    #[must_use]
    fn apply(&self, alg: &Algorithm<Self>) -> Self {
        alg.iter().fold(self.clone(), |p, &m| p * m)
    }
}

impl Mul<Algorithm<Self>> for Cube3x3 {
    type Output = Self;
    fn mul(self, rhs: Algorithm<Self>) -> Self::Output {
        rhs.iter().fold(self, |p, &m| p * m)
    }
}

impl MulAssign<Move3x3> for Cube3x3 {
    fn mul_assign(&mut self, rhs: Move3x3) {
        *self = *self * rhs;
    }
}
