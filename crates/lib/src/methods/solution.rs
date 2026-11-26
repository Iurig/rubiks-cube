use std::{
    fmt::Display,
    ops::{Index, Mul},
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
#[cfg_attr(
    feature = "serde",
    derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)
)]
pub struct MethodSolution<P: Puzzle> {
    step_solutions: Vec<StepSolution<P>>,
}

/// Segment of a solution: includes a name and the moves that make the solution up.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde_with::SerializeDisplay, serde_with::DeserializeFromStr)
)]
pub struct StepSolution<P: Puzzle> {
    name: String,
    moves: Algorithm<P>,
}

impl<'a, P: Puzzle> IntoIterator for &'a StepSolution<P> {
    type Item = &'a <P as Puzzle>::Move;
    type IntoIter = <&'a Algorithm<P> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.moves.iter()
    }
}

impl<P: Puzzle> IntoIterator for StepSolution<P> {
    type Item = <P as Puzzle>::Move;
    type IntoIter = <Algorithm<P> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.moves.into_iter()
    }
}

impl Mul<StepSolution<Self>> for Cube3x3 {
    type Output = Self;
    fn mul(self, rhs: StepSolution<Self>) -> Self::Output {
        rhs.into_iter().fold(self, |p, m| p * m)
    }
}

impl<P: Puzzle> Display for StepSolution<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}    // {}", self.moves, self.name)?;
        Ok(())
    }
}

impl<P: Puzzle> FromStr for MethodSolution<P> {
    type Err = ParseSequenceError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.lines()
            .enumerate()
            .filter_map(|(count, line)| {
                (!line.trim().is_empty()).then_some(
                    line.parse::<StepSolution<P>>()
                        .map_err(|e| e.on_line(count + 1)),
                )
            })
            .collect()
    }
}

impl<P: Puzzle> FromStr for StepSolution<P> {
    type Err = ParseSequenceError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split_comment = s.split_once("//").unwrap_or((s, ""));
        Ok(Self {
            name: split_comment.1.trim().to_owned(),
            moves: split_comment.0.parse()?,
        })
    }
}

impl<P: Puzzle> StepSolution<P> {
    /// Constructor for a [`StepSolution`] with moves.
    #[must_use]
    pub fn new(name: &str, moves: Algorithm<P>) -> Self {
        Self {
            name: String::from(name),
            moves,
        }
    }

    /// Constructor for an empty [`StepSolution`] with a name.
    #[must_use]
    pub(crate) fn empty(name: &str) -> Self {
        Self {
            name: String::from(name),
            moves: Algorithm::new(),
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

    /// Iterates on the [`StepSolution`]s moves.
    pub fn iter(&self) -> impl Iterator<Item = &P::Move> {
        self.moves().iter()
    }

    /// Append a move to the [`StepSolution`].
    pub(crate) fn push(&mut self, m: P::Move) {
        self.moves.push(m);
    }

    /// How many moves the segment has.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.moves.len()
    }

    /// If the segment has any moves.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<P: Puzzle> MethodSolution<P> {
    /// A solution with no segments.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Constructs a solution with only one segment.
    #[must_use]
    pub fn single_segment(name: impl Into<String>, moves: Algorithm<P>) -> Self {
        Self::from_iter([StepSolution {
            name: name.into(),
            moves,
        }])
    }

    /// Each segment's moves and step name, in solving order.
    pub fn iter(&self) -> std::slice::Iter<'_, StepSolution<P>> {
        self.step_solutions.iter()
    }

    /// Returns an [`Iterator`] overall moves in the [`MethodSolution`].
    pub fn moves(&self) -> impl Iterator<Item = &P::Move> {
        self.step_solutions.iter().flat_map(StepSolution::iter)
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

    /// Constructor for [`MethodSolution`] with named, but empty [`StepSolution`]s.
    pub(crate) fn with_empty_steps<'a>(steps: impl IntoIterator<Item = &'a str>) -> Self {
        Self {
            step_solutions: steps
                .into_iter()
                .map(|name| StepSolution::empty(name))
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

impl<P: Puzzle> Index<usize> for MethodSolution<P> {
    type Output = StepSolution<P>;
    #[expect(clippy::indexing_slicing, reason = "it defines the indexing API")]
    fn index(&self, index: usize) -> &Self::Output {
        &self.step_solutions[index]
    }
}

impl<P: Puzzle> MethodSolution<P> {
    #[expect(clippy::indexing_slicing, reason = "it defines the indexing API")]
    pub(crate) fn index_mut(&mut self, index: usize) -> &mut StepSolution<P> {
        &mut self.step_solutions[index]
    }
}

/// Builds a solution from [`StepSolution`]s, in order.
impl<P: Puzzle> FromIterator<StepSolution<P>> for MethodSolution<P> {
    fn from_iter<I: IntoIterator<Item = StepSolution<P>>>(iter: I) -> Self {
        Self {
            step_solutions: iter.into_iter().collect(),
        }
    }
}

/// Joins solutions into one, keeping every segment in order.
impl<P: Puzzle> FromIterator<Self> for MethodSolution<P> {
    fn from_iter<I: IntoIterator<Item = Self>>(iter: I) -> Self {
        Self {
            step_solutions: iter.into_iter().flatten().collect(),
        }
    }
}

impl<'a, P: Puzzle> IntoIterator for &'a MethodSolution<P> {
    type Item = &'a StepSolution<P>;
    type IntoIter = std::slice::Iter<'a, StepSolution<P>>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<P: Puzzle> Extend<StepSolution<P>> for MethodSolution<P> {
    fn extend<T: IntoIterator<Item = StepSolution<P>>>(&mut self, iter: T) {
        self.step_solutions.extend(iter);
    }
}
impl<P: Puzzle> IntoIterator for MethodSolution<P> {
    type Item = StepSolution<P>;
    type IntoIter = <Vec<Self::Item> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.step_solutions.into_iter()
    }
}
impl<P: Puzzle> Display for MethodSolution<P> {
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
        let s: MethodSolution<Cube3x3> = "y U2 r M' // Step 1".parse()?;
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
        let solution: MethodSolution<Cube3x3> = "R U // First \n M' // Second".parse()?;
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
        let mut solution: MethodSolution<Cube3x3> = "".parse()?;
        assert!(solution.is_empty());

        solution.extend("R // First".parse::<MethodSolution<Cube3x3>>()?);
        assert_eq!(solution.len(), 1);

        let cube1 = Cube3x3::default() * "R // First".parse::<StepSolution<Cube3x3>>()?;
        let extension = "U2 // Second\nF' L // Third".parse::<MethodSolution<Cube3x3>>()?;
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

        let names: Vec<&str> = solution.iter().map(StepSolution::name).collect();
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
            let Err(error) = text.parse::<MethodSolution<Cube3x3>>() else {
                panic!("`Q2` should not parse in {text:?}");
            };
            assert_eq!((error.line(), error.position()), expected, "{text:?}");
        }
    }
}
