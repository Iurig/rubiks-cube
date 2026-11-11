use std::{fmt::Display, str::FromStr};

use crate::{Algorithm, ParseSequenceError, puzzles::Puzzle};

/// The moves a method found, as one segment per step, in solving order.
///
/// Each segment is the moves a step applied and the name of the step. `Display` prints one line
/// per segment: the moves in notation, then a tab, `//`, and the step name. That text is valid
/// notation, so a scramble followed by its printed solution replays to the solved state:
///
/// ```text
/// F' Uw2 Rw Fw M' E' F2    //FB
/// U Rw2 U M' U2 Rw' U Rw2 U R    //SB
/// ```
///
/// Build one with [`single_segment`](Self::single_segment), or collect solutions to join them.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Solution<P: Puzzle> {
    step_solutions: Vec<Segment<P>>,
}

/// Segment of a solution: includes a name and the moves that make the solution up.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Segment<P: Puzzle> {
    name: String,
    moves: Algorithm<P>,
}

impl<P: Puzzle> FromStr for Solution<P> {
    type Err = ParseSequenceError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.lines()
            .enumerate()
            .filter_map(|(count, line)| {
                (!line.trim().is_empty())
                    .then_some(line.parse::<Segment<P>>().map_err(|e| e.on_line(count + 1)))
            })
            .collect()
    }
}

impl<P: Puzzle> FromStr for Segment<P> {
    type Err = ParseSequenceError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split_comment = s.split_once("//").unwrap_or((s, ""));
        Ok(Self {
            name: split_comment.1.trim().to_owned(),
            moves: split_comment.0.parse()?,
        })
    }
}

impl<P: Puzzle> Segment<P> {
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
        Self::from_iter([Segment {
            name: name.into(),
            moves,
        }])
    }

    /// Each segment's moves and step name, in solving order.
    pub fn iter(&self) -> std::slice::Iter<'_, Segment<P>> {
        self.step_solutions.iter()
    }

    /// The number of moves in every segment, each move counting one whatever its part or
    /// modifier (slice turn metric). `M`, `Rw`, `y`, and `U2` each count one.
    #[must_use]
    pub fn move_count(&self) -> usize {
        self.step_solutions
            .iter()
            .map(|segment| segment.moves.len())
            .sum()
    }
}

/// Builds a solution from `(moves, step name)` segments, in order.
impl<P: Puzzle> FromIterator<Segment<P>> for Solution<P> {
    fn from_iter<I: IntoIterator<Item = Segment<P>>>(iter: I) -> Self {
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
    type Item = &'a Segment<P>;
    type IntoIter = std::slice::Iter<'a, Segment<P>>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<P: Puzzle> Extend<Segment<P>> for Solution<P> {
    fn extend<T: IntoIterator<Item = Segment<P>>>(&mut self, iter: T) {
        self.step_solutions.extend(iter);
    }
}
impl<P: Puzzle> IntoIterator for Solution<P> {
    type Item = Segment<P>;
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

    use crate::Cube3x3;

    use super::*;

    #[test]
    fn move_count_counts_correctly() -> anyhow::Result<()> {
        let s: Solution<Cube3x3> = "y U2 r M' // Step 1".parse()?;

        assert_eq!(s.move_count(), 4);
        Ok(())
    }

    #[test]
    fn borrowed_and_owned_iteration_yield_the_same_segments_in_order() -> anyhow::Result<()> {
        let solution: Solution<Cube3x3> = "R U // First \n M' // Second".parse()?;

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
        let mut solution: Solution<Cube3x3> = "R // First".parse()?;

        solution.extend("U2 // Second\nF' L // Third".parse::<Solution<Cube3x3>>()?);

        let names: Vec<&str> = solution.iter().map(Segment::name).collect();
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
