use std::{
    fmt::Display,
    ops::{Index, IndexMut, Mul},
    str::FromStr,
};

use crate::{Algorithm, Cube3x3, ParseSequenceError, puzzles::Puzzle};

/// The moves a method found, as one segment per step, in solving order.
///
/// Each segment is the moves a step applied and the name of the step. `Display` prints one line
/// per segment: the moves in notation, then four spaces, `//`, and the step name. That text is
/// valid notation, so a scramble followed by its printed solution replays to the solved state:
///
/// ```text
/// F' Uw2 Rw Fw M' E' F2    // FB
/// U Rw2 U M' U2 Rw' U Rw2 U R    // SB
/// ```
///
/// Build one with [`single_segment`](Self::single_segment), or collect solutions to join them.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Solution<P: Puzzle> {
    step_solutions: Vec<SolutionSegment<P>>,
}

/// Segment of a solution: includes a name and the moves that make the solution up.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SolutionSegment<P: Puzzle> {
    name: String,
    moves: Algorithm<P>,
}

impl<'a, P: Puzzle> IntoIterator for &'a SolutionSegment<P> {
    type Item = &'a <P as Puzzle>::Move;
    type IntoIter = <&'a Algorithm<P> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.moves.iter()
    }
}

impl<P: Puzzle> IntoIterator for SolutionSegment<P> {
    type Item = <P as Puzzle>::Move;
    type IntoIter = <Algorithm<P> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.moves.into_iter()
    }
}

impl Mul<SolutionSegment<Self>> for Cube3x3 {
    type Output = Self;
    fn mul(self, rhs: SolutionSegment<Self>) -> Self::Output {
        rhs.into_iter().fold(self, |p, m| p * m)
    }
}

impl<P: Puzzle> Display for SolutionSegment<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}    // {}", self.moves, self.name)?;
        Ok(())
    }
}

impl<P: Puzzle> FromStr for Solution<P> {
    type Err = ParseSequenceError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.lines()
            .enumerate()
            .filter_map(|(count, line)| {
                (!line.trim().is_empty()).then_some(
                    line.parse::<SolutionSegment<P>>()
                        .map_err(|e| e.on_line(count + 1)),
                )
            })
            .collect()
    }
}

impl<P: Puzzle> FromStr for SolutionSegment<P> {
    type Err = ParseSequenceError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split_comment = s.split_once("//").unwrap_or((s, ""));
        Ok(Self {
            name: split_comment.1.trim().to_owned(),
            moves: split_comment.0.parse()?,
        })
    }
}

impl<P: Puzzle> SolutionSegment<P> {
    /// Constructor for an empty [`SolutionSegment`] with a name.
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            moves: Algorithm::new(),
        }
    }

    /// Constructor for a [`SolutionSegment`] with moves.
    #[must_use]
    pub fn with_moves(name: &str, moves: Algorithm<P>) -> Self {
        Self {
            name: String::from(name),
            moves,
        }
    }

    /// Getter for the name of the step that generated the segment.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Getter for the moves that make the segment up.
    #[must_use]
    pub const fn moves(&self) -> &Algorithm<P> {
        &self.moves
    }

    /// Iterates on the [`SolutionSegment`]s moves.
    pub fn iter(&self) -> impl Iterator<Item = &P::Move> {
        self.moves().iter()
    }

    /// Append a move to the [`SolutionSegment`].
    pub fn push(&mut self, m: P::Move) {
        self.moves.push(m);
    }
}

impl<P: Puzzle> Solution<P> {
    /// A solution with no segments.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Constructs a solution with only one segment.
    #[must_use]
    pub fn single_segment(name: impl Into<String>, moves: Algorithm<P>) -> Self {
        Self::from_iter([SolutionSegment {
            name: name.into(),
            moves,
        }])
    }

    /// Each segment's moves and step name, in solving order.
    pub fn iter(&self) -> std::slice::Iter<'_, SolutionSegment<P>> {
        self.step_solutions.iter()
    }

    /// Returns an [`Iterator`] overall moves in the [`Solution`].
    pub fn moves(&self) -> impl Iterator<Item = &P::Move> {
        self.step_solutions.iter().flat_map(SolutionSegment::iter)
    }

    /// The number of moves in every segment, each move counting one whatever its part or
    /// modifier (ESTM). `M`, `Rw`, `y`, and `U2` each count one.
    #[must_use]
    pub fn move_count(&self) -> usize {
        self.step_solutions
            .iter()
            .map(|segment| segment.moves.len())
            .sum()
    }

    /// Constructor for [`Solution`] with named, but empty [`SolutionSegment`]s.
    pub fn with_empty_steps<'a>(steps: impl IntoIterator<Item = &'a str>) -> Self {
        Self {
            step_solutions: steps
                .into_iter()
                .map(|name| SolutionSegment::new(name))
                .collect(),
        }
    }

    /// How many steps the solution has.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.step_solutions.len()
    }

    /// If the solution is devoid of steps.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.step_solutions.is_empty()
    }
}

impl<P: Puzzle> Index<usize> for Solution<P> {
    type Output = SolutionSegment<P>;
    #[expect(clippy::indexing_slicing, reason = "it defines the indexing API")]
    fn index(&self, index: usize) -> &Self::Output {
        &self.step_solutions[index]
    }
}

impl<P: Puzzle> IndexMut<usize> for Solution<P> {
    #[expect(clippy::indexing_slicing, reason = "it defines the indexing API")]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.step_solutions[index]
    }
}

/// Builds a solution from [`SolutionSegment`]s, in order.
impl<P: Puzzle> FromIterator<SolutionSegment<P>> for Solution<P> {
    fn from_iter<I: IntoIterator<Item = SolutionSegment<P>>>(iter: I) -> Self {
        Self {
            step_solutions: iter.into_iter().collect(),
        }
    }
}

/// Joins solutions into one, keeping every segment in order.
impl<P: Puzzle> FromIterator<Self> for Solution<P> {
    fn from_iter<I: IntoIterator<Item = Self>>(iter: I) -> Self {
        Self {
            step_solutions: iter.into_iter().flatten().collect(),
        }
    }
}

impl<'a, P: Puzzle> IntoIterator for &'a Solution<P> {
    type Item = &'a SolutionSegment<P>;
    type IntoIter = std::slice::Iter<'a, SolutionSegment<P>>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<P: Puzzle> Extend<SolutionSegment<P>> for Solution<P> {
    fn extend<T: IntoIterator<Item = SolutionSegment<P>>>(&mut self, iter: T) {
        self.step_solutions.extend(iter);
    }
}
impl<P: Puzzle> IntoIterator for Solution<P> {
    type Item = SolutionSegment<P>;
    type IntoIter = <Vec<Self::Item> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.step_solutions.into_iter()
    }
}
impl<P: Puzzle> Display for Solution<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for alg in self {
            writeln!(f, "{}    // {}", alg.moves, alg.name)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::panic_in_result_fn,
        reason = "`?` reports setup failures; `assert!` reports the property under test failing"
    )]

    use itertools::Itertools;

    use crate::Cube3x3;

    use super::*;

    #[test]
    fn move_count_counts_correctly() -> anyhow::Result<()> {
        let s: Solution<Cube3x3> = "y U2 r M' // Step 1".parse()?;
        assert!(!s.is_empty());
        assert_eq!(s.move_count(), 4);

        let algorithm = Algorithm::<Cube3x3>::from_static("y U2 r M'");
        let section = s[0].clone();
        for (m1, m2) in (&section).into_iter().zip_eq(&algorithm) {
            assert_eq!(m1, m2);
        }
        for (m1, m2) in section.into_iter().zip_eq(algorithm) {
            assert_eq!(m1, m2);
        }
        Ok(())
    }

    #[test]
    fn borrowed_and_owned_iteration_yield_the_same_segments_in_order() -> anyhow::Result<()> {
        let solution: Solution<Cube3x3> = "R U // First \n M' // Second".parse()?;
        assert_eq!(solution.len(), 2);

        let mut borrowed = Vec::new();
        for segment in &solution {
            borrowed.push((segment.name.clone(), segment.moves.clone()));
        }
        let owned: Vec<(String, Algorithm<Cube3x3>)> = solution
            .into_iter()
            .map(|segment| (segment.name, segment.moves))
            .collect();

        let names: Vec<&str> = borrowed.iter().map(|(name, _)| name.as_str()).collect();
        assert_eq!(names, ["First", "Second"]);
        assert_eq!(borrowed, owned);
        Ok(())
    }

    #[test]
    fn extend_appends_segments_after_the_existing_ones() -> anyhow::Result<()> {
        let mut solution: Solution<Cube3x3> = "".parse()?;
        assert!(solution.is_empty());

        solution.extend("R // First".parse::<Solution<Cube3x3>>()?);
        assert_eq!(solution.len(), 1);

        let cube1 = Cube3x3::default() * "R // First".parse::<SolutionSegment<Cube3x3>>()?;
        let extension = "U2 // Second\nF' L // Third".parse::<Solution<Cube3x3>>()?;
        solution.extend(extension.clone());
        assert_eq!(
            cube1 * extension[0].clone() * extension[1].clone(),
            Cube3x3::default() * solution[0].clone() * solution[1].clone() * solution[2].clone()
        );
        assert_ne!(
            cube1 * extension[0].clone(),
            Cube3x3::default() * solution[0].clone() * solution[1].clone() * solution[2].clone()
        );
        assert_eq!(solution.len(), 3);

        let names: Vec<&str> = solution.iter().map(SolutionSegment::name).collect();
        assert_eq!(names, ["First", "Second", "Third"]);
        assert_eq!(solution.move_count(), 4);
        Ok(())
    }

    #[test]
    fn an_invalid_move_names_its_line_and_position() {
        for (text, expected) in [
            ("Q2 // First", (1, 1)),
            ("R // First\nM Q2 // Second", (2, 2)),
            ("R // First\n\nM Q2 // Second", (3, 2)),
        ] {
            let Err(error) = text.parse::<Solution<Cube3x3>>() else {
                panic!("`Q2` should not parse in {text:?}");
            };
            assert_eq!((error.line(), error.position()), expected, "{text:?}");
        }
    }
}
