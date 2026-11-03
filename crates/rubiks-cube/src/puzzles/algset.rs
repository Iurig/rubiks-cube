use std::{str::FromStr, sync::Arc};

use itertools::Itertools;

use crate::{
    Algorithm, Cube3x3, ParseSequenceError, Puzzle,
    puzzles::cube3x3::moves::{Move3x3, MoveModifier},
};

/// A collection of move sequences, each applied as one unit: a single move, or a whole algorithm.
///
/// For the cube, build one from notation text with [`from_parts`](Self::from_parts),
/// [`from_str`](std::str::FromStr), or [`from_moves`](Self::from_moves), and join
/// several with [`combined_with`](Self::combined_with).
///
/// ```
/// use std::str::FromStr;
/// use rubiks_cube::{Cube3x3, AlgSet};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Two algorithms to search with, and the AUF to pass as the free sequences.
/// let algorithms = AlgSet::<Cube3x3>::from_str("R U R' U R U2 R'\nR U2 R' U' R U' R'")?;
/// let auf = AlgSet::<Cube3x3>::from_moves("U U2 U'")?;
/// # Ok(())
/// # }
/// ```
///
/// Two collections with the same sequences are equal and hash alike, whatever order the
/// sequences were given in and however often each was repeated.
// Every constructor goes through `FromIterator`, which sorts and dedupes, so the derived
// `PartialEq` and `Hash` compare the sequences as a set.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct AlgSet<P: Puzzle>(Arc<[Algorithm<P>]>);

impl<P: Puzzle> AlgSet<P> {
    /// Constructs an empty [`AlgSet`].
    #[must_use]
    pub fn new() -> Self {
        Self(Arc::new([]))
    }

    /// Every sequence of either collection, each listed once.
    ///
    /// The order of the sequences is not kept, and a sequence in both collections, or twice in
    /// one, appears once.
    #[must_use]
    pub fn combined_with(&self, other: &Self) -> Self {
        self.0.iter().chain(other.0.iter()).cloned().collect()
    }

    /// The sequences.
    pub(crate) fn algs(&self) -> &[Algorithm<P>] {
        &self.0
    }

    /// Returns an iterator over [`Algorithm`]s.
    pub fn iter(&self) -> std::slice::Iter<'_, Algorithm<P>> {
        self.0.iter()
    }

    /// Like [`from_str`](Self::from_str), for algorithms written in the source code.
    ///
    /// # Panics
    /// If `text` contains an invalid move.
    #[must_use]
    #[track_caller]
    #[expect(clippy::panic, reason = "only usable on statics")]
    pub fn from_static(text: &'static str) -> Self {
        text.parse()
            .unwrap_or_else(|e| panic!("invalid algorithms {text:?}: {e:?}"))
    }
}

impl<P: Puzzle> FromIterator<Algorithm<P>> for AlgSet<P> {
    fn from_iter<T: IntoIterator<Item = Algorithm<P>>>(iter: T) -> Self {
        let mut algs: Vec<Algorithm<P>> = iter.into_iter().collect();
        algs.sort_unstable();
        algs.dedup();
        Self(algs.into())
    }
}

impl<'a, P: Puzzle> IntoIterator for &'a AlgSet<P> {
    type Item = &'a Algorithm<P>;
    type IntoIter = std::slice::Iter<'a, Algorithm<P>>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// One sequence per line of `text`, such as an algorithm set. Lines with no moves, such as
/// blank or comment-only lines, are skipped. A line written twice is listed once, and the
/// order of the lines is not kept.
///
/// # Errors
/// If `text` contains an invalid move. The error's line counts lines of the whole `text`.
impl<P: Puzzle> FromStr for AlgSet<P> {
    type Err = ParseSequenceError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.lines()
            .enumerate()
            .filter_map(|(line_number, line)| {
                line.parse::<Algorithm<P>>()
                    .map(|moves| (!moves.is_empty()).then_some(moves))
                    .map_err(|e| e.on_line(line_number + 1))
                    .transpose()
            })
            .collect()
    }
}

impl AlgSet<Cube3x3> {
    /// Every modifier of each part named in `text`, one move per sequence.
    ///
    /// The modifier in the text does not matter: `"U R M r"` and `"U' R2 M r'"` both give the
    /// clockwise, counterclockwise, and double turn of `U`, `R`, `M`, and `r`, twelve moves.
    /// A part named twice is listed once, and the order of the parts is not kept.
    ///
    /// # Errors
    /// If `text` contains an invalid move.
    pub fn from_parts(moves: impl IntoIterator<Item = Move3x3>) -> Self {
        moves
            .into_iter()
            .map(Move3x3::part)
            .unique()
            .flat_map(|part| {
                [
                    MoveModifier::Clockwise,
                    MoveModifier::Prime,
                    MoveModifier::Double,
                ]
                .map(|modifier| Algorithm::from_iter([Move3x3::new(part, modifier)]))
            })
            .collect()
    }

    /// Creates an [`AlgSet`] from a `\n` separated `&'static str`.
    ///
    /// Same as `from_parts(Algorithm::<Cube3x3>::from_static(parts))`.
    #[must_use]
    #[track_caller]
    pub fn from_static_parts(parts: &'static str) -> Self {
        Self::from_parts(Algorithm::<Cube3x3>::from_static(parts))
    }

    /// Each move of `text` as its own sequence, keeping its modifier.
    ///
    /// A move written twice is listed once, and the order of the moves is not kept.
    ///
    /// # Errors
    /// If `text` contains an invalid move.
    pub fn from_moves(text: &str) -> Result<Self, ParseSequenceError> {
        Ok(text
            .parse::<Algorithm<Cube3x3>>()?
            .iter()
            .map(|&m| Algorithm::from_iter([m]))
            .collect())
    }

    /// Adds an empty [`Algorithm`] to an [`AlgSet`].
    #[must_use]
    pub fn or_skip(&self) -> Self {
        self.combined_with(&Self::from_iter([Algorithm::new()]))
    }
}

#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    reason = "tests should panic if failed, and return result for `?` convenience"
)]
mod tests {

    use crate::fast_hash::FxSet;

    use super::*;

    #[test]
    fn combined_with_reconstructs_an_algset() {
        let algset = AlgSet::from_static_parts("R U L F B D");

        for splitpoint in 0..algset.algs().len() {
            let (first, second) = algset.algs().split_at(splitpoint);
            assert_eq!(
                algset,
                AlgSet::from_iter(first.iter().cloned())
                    .combined_with(&AlgSet::from_iter(second.iter().cloned()))
            );
        }
    }

    #[test]
    fn algsets_are_equal_when_they_hold_the_same_sequences() -> anyhow::Result<()> {
        let algset = AlgSet::<Cube3x3>::from_str("R U\nF")?;

        // Order and repeats don't matter.
        for same in ["R U\nF", "F\nR U", "R U\nF\nR U"] {
            let same = AlgSet::from_str(same)?;
            assert_eq!(algset, same);
            assert_eq!(same, algset);
        }

        // Missing, extra, or different sequence matters.
        for different in ["", "R U", "R U\nF\nF'", "U R\nF"] {
            let different = AlgSet::from_str(different)?;
            assert_ne!(algset, different);
            assert_ne!(different, algset);
        }

        Ok(())
    }

    #[test]
    fn from_parts_gives_every_modifier() {
        let algset = AlgSet::from_static_parts("U R M r");

        assert_eq!(algset.algs().len(), 12);
        assert!(algset.algs().iter().all(|sequence| sequence.len() == 1));
    }

    #[test]
    fn from_str_gives_one_sequence_per_line() -> anyhow::Result<()> {
        let algset = AlgSet::<Cube3x3>::from_str("R U R' U'\n\nF R U R' U' F'\nM2 U2")?;

        let mut lengths: Vec<usize> = algset.algs().iter().map(Algorithm::len).collect();
        lengths.sort_unstable();
        assert_eq!(lengths, [2, 4, 6]);
        Ok(())
    }

    #[test]
    fn iterating_a_borrowed_algset_gives_its_algs() -> anyhow::Result<()> {
        let algset = AlgSet::<Cube3x3>::from_str("R U\nF2\nM' E")?;

        let iterated: Vec<&Algorithm<Cube3x3>> = (&algset).into_iter().collect();

        assert_eq!(iterated, algset.algs().iter().collect::<Vec<_>>());
        Ok(())
    }

    #[test]
    fn from_moves_removes_duplicates() -> anyhow::Result<()> {
        let expected: FxSet<Algorithm<Cube3x3>> = ["U", "U2", "U'"]
            .into_iter()
            .map(str::parse)
            .collect::<Result<_, _>>()?;

        let algset = AlgSet::<Cube3x3>::from_moves("U U2 U' U")?;

        let actual: FxSet<Algorithm<Cube3x3>> = algset.algs().iter().cloned().collect();
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn an_invalid_move_names_its_line_and_position() {
        for result in [
            Algorithm::<Cube3x3>::from_str("U R\nM Q2").map(AlgSet::from_parts),
            AlgSet::from_str("R U R'\nM Q2"),
            AlgSet::from_moves("U\nU2 Q2"),
        ] {
            let Err(error) = result else {
                panic!("`Q2` should not parse");
            };
            assert_eq!((error.line(), error.position()), (2, 2), "{error:?}");
        }
    }
}
