use std::{
    fmt::{self, Display},
    iter::successors,
    ops::Index,
    str::FromStr,
};

use itertools::Itertools;

use crate::{Cube3x3, Inv, Move3x3, MoveModifier, ParseSequenceError, Puzzle};

/// A sequence of moves applied in order, such as an algorithm or a step's solution.
///
/// It prints as its moves separated by spaces, in the crate's notation, so what it prints reads
/// back as the same moves. It implements [`FromStr`], which parses ignoring line comments written
/// with `//`, as well as empty lines. [`&str`](std::str)s that can't be parsed will return an error
/// carrying line number and position along the line, 1 indexed.
///
/// ```no_run
/// use rubiks_cube::{Algorithm, Cube3x3, Inv, Solution};
/// use rubiks_cube::{Kociemba, Method};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut cube = Cube3x3::from_moves("R U R' F2")?;
/// let solution: Solution<Cube3x3> = Kociemba.solve(&mut cube)?;
/// let moves: Algorithm<Cube3x3> = solution.iter().flat_map(|s| s.moves()).copied().collect();
/// // The inverse of a solution is a scramble that gives back the state it solved.
/// assert_eq!(
///     Cube3x3::from_moves(&moves.inv().to_string())?,
///     Cube3x3::from_moves("R U R' F2")?
/// );
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Algorithm<P: Puzzle>(Vec<P::Move>);

/// The moves in reverse order, each inverted: applying an algorithm and then its inverse leaves
/// any state unchanged.
impl<P: Puzzle> Inv for Algorithm<P> {
    fn inv(self) -> Self {
        self.into_iter().rev().map(Inv::inv).collect()
    }
}

impl<P: Puzzle> FromStr for Algorithm<P> {
    type Err = ParseSequenceError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.lines()
            .enumerate()
            .flat_map(|(line_index, line)| {
                let moves = line.split_once("//").map_or(line, |(moves, _)| moves);
                moves
                    .split_whitespace()
                    .enumerate()
                    .map(move |(move_index, m)| {
                        P::Move::from_str(m).map_err(|e| ParseSequenceError {
                            source: e,
                            line: line_index + 1,
                            position: move_index + 1,
                        })
                    })
            })
            .collect()
    }
}

impl<P: Puzzle> AsRef<[P::Move]> for Algorithm<P> {
    fn as_ref(&self) -> &[P::Move] {
        &self.0[..]
    }
}

/// The moves separated by spaces, such as `R U R' U'`. An empty algorithm prints nothing.
impl<P: Puzzle> Display for Algorithm<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut moves = self.0.iter();
        if let Some(first) = moves.next() {
            write!(f, "{first}")?;
            for m in moves {
                write!(f, " {m}")?;
            }
        }
        Ok(())
    }
}

impl<P: Puzzle> PartialOrd for Algorithm<P> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<P: Puzzle> Ord for Algorithm<P> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<P: Puzzle> FromIterator<P::Move> for Algorithm<P> {
    fn from_iter<I: IntoIterator<Item = P::Move>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<P: Puzzle> Extend<P::Move> for Algorithm<P> {
    fn extend<I: IntoIterator<Item = P::Move>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<P: Puzzle> IntoIterator for Algorithm<P> {
    type Item = P::Move;
    type IntoIter = std::vec::IntoIter<P::Move>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, P: Puzzle> IntoIterator for &'a Algorithm<P> {
    type Item = &'a P::Move;
    type IntoIter = std::slice::Iter<'a, P::Move>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<P: Puzzle> Index<usize> for Algorithm<P> {
    type Output = P::Move;
    #[expect(
        clippy::indexing_slicing,
        reason = "failing indexing out of range is the intended behaviour for the trait."
    )]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<P: Puzzle> Algorithm<P> {
    /// An algorithm with no moves.
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// The moves, in the order they are applied.
    pub fn iter(&self) -> std::slice::Iter<'_, P::Move> {
        self.0.iter()
    }

    /// The moves in a [`Vec`].
    pub fn to_vec(self) -> Vec<P::Move> {
        self.0
    }

    /// The number of moves.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the algorithm has no moves.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Adds the moves of `other` after the last move.
    pub fn extend_from(&mut self, other: &Self) {
        self.0.extend_from_slice(&other.0);
    }

    /// Same as [`FromStr`](core::str::FromStr), but for `'static` types.
    ///
    /// # Panics
    /// Panics when [`FromStr`](core::str::FromStr) would error.
    #[must_use]
    #[expect(
        clippy::panic,
        reason = "since the input is static, validation only needs to happen once."
    )]
    #[track_caller]
    pub fn from_static(s: &'static str) -> Self {
        s.parse()
            .unwrap_or_else(|e| panic!("invalid static &str: {e}"))
    }
}

impl Algorithm<Cube3x3> {
    /// Simplifies an algorithm not to include double prime moves.
    #[must_use]
    pub fn simplify_single_moves(self) -> Self {
        self.into_iter()
            .map(|m| {
                if m.modifier() == MoveModifier::DoublePrime {
                    Move3x3::new(m.part(), MoveModifier::Double)
                } else {
                    m
                }
            })
            .collect()
    }

    /// Simplifies an algorithm by canceling moves where possible.
    #[must_use]
    #[expect(
        clippy::missing_panics_doc,
        reason = "panicking breaks invariants - nothing useful to the final user"
    )]
    pub fn simplify_cancelations(self) -> Self {
        let cancel_once = |alg: Self| -> Self {
            alg.into_iter()
                .map(Some)
                .coalesce(|mov1, mov2| {
                    if let (Some(inner1), Some(inner2)) = (mov1, mov2) {
                        if inner1.part() == inner2.part() {
                            MoveModifier::from_number(
                                inner1.modifier().as_number() + inner2.modifier().as_number(),
                            )
                            .map_or(Ok(None), |modif| {
                                Ok(Some(Move3x3::new(inner1.part(), modif)))
                            })
                        } else {
                            Err((mov1, mov2))
                        }
                    } else {
                        Err((mov1, mov2))
                    }
                })
                .flatten()
                .collect()
        };

        successors(Some(self), |alg| {
            let next = cancel_once(alg.clone());
            (next != *alg).then_some(next)
        })
        .last()
        .expect("empy iterator")
    }
}

#[cfg(test)]
#[expect(clippy::panic_in_result_fn, reason = "Standardized across tests")]
mod tests {

    use crate::Cube3x3;

    use super::*;

    #[test]
    fn algorithm_into_iter_and_iter_agree() -> anyhow::Result<()> {
        let alg: Algorithm<Cube3x3> = "R U R' U' R' F R2 U' R' U' R U R' F'".parse()?;
        let alg_vec_2 = alg.iter().copied().collect::<Vec<_>>();
        let alg_vec_1 = alg.into_iter().collect::<Vec<_>>();
        assert_eq!(alg_vec_1, alg_vec_2);
        Ok(())
    }

    #[test]
    fn as_ref_is_the_moves_in_order() -> anyhow::Result<()> {
        let alg: Algorithm<Cube3x3> = "R U R' F2".parse()?;
        let expected = ["R", "U", "R'", "F2"]
            .map(<Cube3x3 as Puzzle>::Move::from_str)
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(alg.as_ref(), expected.as_slice());
        assert_eq!(Algorithm::<Cube3x3>::new().as_ref(), []);
        Ok(())
    }

    #[test]
    fn index_is_the_move_at_that_position() -> anyhow::Result<()> {
        let alg: Algorithm<Cube3x3> = "R U R' F2".parse()?;
        for (i, text) in ["R", "U", "R'", "F2"].into_iter().enumerate() {
            assert_eq!(
                alg[i],
                <Cube3x3 as Puzzle>::Move::from_str(text)?,
                "move {i}"
            );
        }
        Ok(())
    }

    fn cancels_to_equal(alg1: &str, alg2: &str) {
        let alg: Algorithm<Cube3x3> = alg1.parse().unwrap();
        let expected: Algorithm<Cube3x3> = alg2.parse().unwrap();
        assert_eq!(expected, alg.simplify_cancelations());
    }

    #[test]
    fn simplify_same_part_cancelations() {
        cancels_to_equal("R R", "R2");
        cancels_to_equal("R R'", "");
        cancels_to_equal("R R R R U U' R", "R");
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn indexing_past_the_last_move_panics() {
        let alg: Algorithm<Cube3x3> = Algorithm::new();
        let _ = alg[0];
    }
}
