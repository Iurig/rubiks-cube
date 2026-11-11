use std::{borrow::Cow, sync::Arc};

use crate::{Puzzle, Solution, Step, StepError};

/// A step that tries several steps and keeps the shortest working solution.
///
/// Each alternative solves its own copy of the puzzle. Alternatives returning
/// [`StepError::InvalidStartingState`] are skipped, so `Shortest` also picks whichever
/// alternatives can start at all. Any other error ends the whole choice. On a tie, the earlier
/// alternative wins. The solution keeps the name of the alternative that ran, and the puzzle is
/// left as that alternative left it.
///
/// Roux uses it to build either the front or the back square of a block, and then the pair
/// that square leaves.
///
/// [`is_done`](Step::is_done) holds when any alternative's `is_done` holds.
#[derive(Debug)]
pub struct Shortest<P: Puzzle> {
    steps: Vec<Arc<dyn Step<P>>>,
    name: Cow<'static, str>,
}

impl<P: Puzzle> Shortest<P> {
    /// A choice between `steps`, its name is a simple direct reference to the steps it chooses,
    /// e.g. `"step1 or step2 or step3"`. The name appears in errors about the choice itself.
    /// When an alternative runs, the solution names that alternative instead.
    #[must_use]
    pub fn new(steps: impl IntoIterator<Item = Arc<dyn Step<P>>>) -> Self {
        let (steps_collected, name) = steps.into_iter().fold(
            (Vec::new(), Cow::<str>::Owned(String::new())),
            |(mut step_vector, mut name), s| {
                step_vector.push(s.clone());
                if !name.is_empty() {
                    name.to_mut().push_str(" or ");
                }
                name.to_mut().push_str(s.name());
                (step_vector, name)
            },
        );

        Self {
            steps: steps_collected,
            name,
        }
    }

    /// A choice between `steps`, explicitly named `name`.
    #[must_use]
    pub fn named(
        name: impl Into<Cow<'static, str>>,
        steps: impl IntoIterator<Item = Arc<dyn Step<P>>>,
    ) -> Self {
        Self {
            steps: steps.into_iter().collect(),
            name: name.into(),
        }
    }
}

impl<P: Puzzle> FromIterator<Arc<dyn Step<P>>> for Shortest<P> {
    fn from_iter<T: IntoIterator<Item = Arc<dyn Step<P>>>>(iter: T) -> Self {
        Self::new(iter)
    }
}

impl<P: Puzzle> Step<P> for Shortest<P> {
    fn name(&self) -> &str {
        &self.name
    }

    fn can_solve(&self, puzzle: &P) -> bool {
        self.steps.iter().any(|s| s.can_solve(puzzle))
    }

    fn solve(&self, puzzle: &mut P) -> Result<Solution<P>, StepError> {
        let best = self
            .steps
            .iter()
            .map(|s| {
                let mut candidate = puzzle.clone();
                s.solve(&mut candidate)
                    .map(|solution| (solution, candidate))
            })
            .try_fold(
                None,
                |best: Option<(Solution<P>, P)>, candidate| match candidate {
                    Err(StepError::InvalidStartingState { .. }) => Ok(best),
                    Err(e @ (StepError::UnreachableGoal | StepError::Custom(_))) => Err(e),
                    // On a tie the earlier alternative stays.
                    Ok(candidate) => Ok(match best {
                        Some(best) if best.0.move_count() <= candidate.0.move_count() => Some(best),
                        _ => Some(candidate),
                    }),
                },
            )?;
        let (solution, solved) = best.ok_or_else(|| StepError::InvalidStartingState {
            state: puzzle.to_string(),
        })?;
        *puzzle = solved;
        Ok(solution)
    }

    fn is_done(&self, puzzle: &P) -> bool {
        self.steps.iter().any(|s| s.is_done(puzzle))
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::panic_in_result_fn,
        reason = "`?` reports setup failures; `assert!` reports the property under test failing"
    )]

    use super::*;
    use crate::{Cube3x3, Segment, Technique, methods::test_steps::FixedStep};

    fn cannot_start() -> StepError {
        StepError::InvalidStartingState {
            state: String::new(),
        }
    }

    fn unreachable() -> StepError {
        StepError::UnreachableGoal
    }

    #[test]
    fn the_shorter_alternative_is_kept_and_the_longer_leaves_no_trace() -> anyhow::Result<()> {
        // The longer one comes first, so the win is not just list order.
        let steps = Technique::new([
            Arc::new(FixedStep::new("Long", "R U", true, true)) as Arc<dyn Step<Cube3x3>>,
            Arc::new(FixedStep::new("Short", "R", true, true)),
        ]);
        let choose = Shortest::named("Either", steps);
        let mut cube = Cube3x3::default();

        let solution = choose.solve(&mut cube)?;

        assert_eq!(
            cube,
            Cube3x3::from_moves("R")?,
            "only the short moves are applied"
        );
        assert_eq!(solution.move_count(), 1);
        let names: Vec<&str> = solution.iter().map(Segment::name).collect();
        assert_eq!(
            names,
            ["Short"],
            "the segment is named after the step that ran"
        );
        Ok(())
    }

    #[test]
    fn an_alternative_that_cannot_start_is_skipped() -> anyhow::Result<()> {
        let steps: Vec<Arc<dyn Step<Cube3x3>>> = vec![
            FixedStep::failing("Cannot start", cannot_start),
            Arc::new(FixedStep::new("Runs", "U", true, true)) as Arc<dyn Step<Cube3x3>>,
        ];
        let choose = Shortest::named("Either", steps);
        let mut cube = Cube3x3::default();

        let solution = choose.solve(&mut cube)?;

        assert_eq!(cube, Cube3x3::from_moves("U")?);
        let names: Vec<&str> = solution.iter().map(Segment::name).collect();
        assert_eq!(names, ["Runs"]);
        Ok(())
    }

    #[test]
    fn any_other_error_ends_the_choice_even_if_another_alternative_succeeds() {
        let steps: Vec<Arc<dyn Step<Cube3x3>>> = vec![
            FixedStep::failing("Broken", unreachable),
            Arc::new(FixedStep::new("Would run", "U", true, true)) as Arc<dyn Step<Cube3x3>>,
        ];
        let choose = Shortest::named("Either", steps);
        let mut cube = Cube3x3::default();

        let result = choose.solve(&mut cube);

        assert!(
            matches!(result, Err(StepError::UnreachableGoal)),
            "{result:?}"
        );
        assert_eq!(
            cube,
            Cube3x3::default(),
            "a failed choice leaves the cube alone"
        );
    }

    #[test]
    fn when_no_alternative_can_start_the_choice_cannot_start() {
        let steps: Vec<Arc<dyn Step<Cube3x3>>> = vec![
            FixedStep::failing("First", cannot_start),
            FixedStep::failing("Second", cannot_start),
        ];
        let choose = Shortest::named("Either", steps);
        let mut cube = Cube3x3::default();

        let result = choose.solve(&mut cube);

        assert!(
            matches!(result, Err(StepError::InvalidStartingState { .. })),
            "{result:?}"
        );
        assert_eq!(cube, Cube3x3::default());
    }

    #[test]
    fn the_choice_can_solve_only_when_an_alternative_can() {
        let cube = Cube3x3::default();
        let cannot: Arc<dyn Step<Cube3x3>> = Arc::new(FixedStep::new("Cannot", "", false, false));
        let can: Arc<dyn Step<Cube3x3>> = Arc::new(FixedStep::new("Can", "", true, true));

        let neither = Shortest::named("Neither", vec![Arc::clone(&cannot), Arc::clone(&cannot)]);
        let second = Shortest::named("Second", vec![cannot, can]);

        assert!(!neither.can_solve(&cube));
        assert!(second.can_solve(&cube));
    }

    #[test]
    fn the_shorter_alternative_wins_whichever_comes_first() -> anyhow::Result<()> {
        for short_first in [false, true] {
            let short: Arc<dyn Step<Cube3x3>> = Arc::new(FixedStep::new("Short", "R", true, true));
            let long: Arc<dyn Step<Cube3x3>> = Arc::new(FixedStep::new("Long", "R U", true, true));
            let alternatives = if short_first {
                vec![short, long]
            } else {
                vec![long, short]
            };
            let choose = Shortest::named("Either", alternatives);
            let mut cube = Cube3x3::default();

            let solution = choose.solve(&mut cube)?;

            let names: Vec<&str> = solution.iter().map(Segment::name).collect();
            assert_eq!(names, ["Short"], "short first: {short_first}");
            assert_eq!(
                cube,
                Cube3x3::from_moves("R")?,
                "short first: {short_first}"
            );
        }
        Ok(())
    }

    #[test]
    fn on_a_tie_the_earlier_alternative_is_kept() -> anyhow::Result<()> {
        let steps: Vec<Arc<dyn Step<Cube3x3>>> = vec![
            Arc::new(FixedStep::new("First", "R", true, true)),
            Arc::new(FixedStep::new("Second", "U", true, true)),
        ];
        let choose = Shortest::named("Either", steps);
        let mut cube = Cube3x3::default();

        let solution = choose.solve(&mut cube)?;

        let names: Vec<&str> = solution.iter().map(Segment::name).collect();
        assert_eq!(names, ["First"]);
        assert_eq!(cube, Cube3x3::from_moves("R")?);
        Ok(())
    }
}
