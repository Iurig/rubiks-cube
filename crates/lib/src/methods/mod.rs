pub mod cube3x3;
//pub mod neutrality;
pub mod context;
mod solution;
pub mod step;
#[cfg(test)]
mod test_steps;

use dyn_clone::arc_make_mut;
use log::debug;
pub use solution::{Segment, Solution};

use std::{fmt::Debug, ops::Index, sync::Arc, time::Instant};

use crate::{
    PieceSet, SolveError, TechniqueError,
    methods::{
        context::{BuildContext, GLOBAL},
        step::{Step, StepBuilder},
    },
    puzzles::Puzzle,
};

/// The ordered list of [`Step`]s that a [`Method`] runs, built by
/// [`to_technique`](Method::to_technique) from the method's options.
///
/// [`Roux`](crate::Roux) gives the Roux steps for [`Cube3x3`](crate::Cube3x3). Any other list
/// of steps works too, and the steps can be of different types.
///
/// ```rust
/// use rubiks_cube::{Cube3x3, Method, Puzzle, Roux};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut cube = Cube3x3::from_moves("R U R' F' L2 D B'")?;
/// let solution = Roux::default().solve(&mut cube)?;
/// assert!(cube.is_solved());
/// println!("{solution}");
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct Technique<P: Puzzle> {
    steps: Vec<Arc<dyn Step<P>>>,
    // neutrality: Neutrality<P>,
}

/// Builder for [`Technique`].
///
/// Can be used to build a [`Technique`] from uncomplete [`StepBuilder`]s.
///
/// # Examples
///
/// ```ignore
/// use rubiks_cube::{AlgSet, SearchStep, Technique, PieceSet, Cube3x3};
///
/// let eolr = SearchStep::builder("EOLR", AlgSet::from_static_parts("M U"))
///     .goal(PieceSet::from_algset(&AlgSet::from_static("M\n U2")));
///
/// let advanced_roux = Technique::<Cube3x3>::start_with(SearchStep::builder(
///     "FB",
///     AlgSet::from_static_parts("R U F B D M r"),
/// ))
/// .then(SearchStep::builder(
///     "SB",
///     AlgSet::from_static_parts("R U r M"),
/// ))
/// .then(
///     SearchStep::builder(
///        "CMLL",
///         AlgSet::from_static_parts(concat!(include_str!("path/to/algs"))),
///     )
///     .free_algs(AlgSet::from_static_parts("U"))
///     .goal(PieceSet::from_algset(&AlgSet::from_static("U M' U' \n M"))),
/// )
/// .then(eolr)
/// .finish(SearchStep::builder("LSE", AlgSet::from_static("M U")));
/// ```
#[derive(Clone, Debug)]
pub struct TechniqueBuilder<P: Puzzle> {
    step_builders: Vec<Arc<dyn StepBuilder<P>>>,
}

impl<P: Puzzle> TechniqueBuilder<P> {
    /// Adds a [`Step`] to the final technique, constructed from a [`StepBuilder`].
    ///
    /// It constructs the step's goal to be the next step's
    /// [`assumed_solved`](StepBuilder::assumed_solved) and its own
    /// [`assumed_solved`](StepBuilder::assumed_solved) based on
    /// its [`allowed_sequences`](StepBuilder::allowed_sequences). If either can't be constructed
    /// because the [`StepBuilder`] doesn't provide one, it must be provided.
    pub fn then(&mut self, step: impl StepBuilder<P> + 'static) -> &mut Self {
        self.step_builders.push(Arc::new(step));
        self
    }

    /// Adds the final [`Step`] of a technique.
    ///
    /// It constructs its [`assumed_solved`](StepBuilder::assumed_solved) based on its
    /// [`allowed_sequences`](StepBuilder::allowed_sequences).
    ///
    /// # Errors
    ///
    /// Errors if the provided `step`, or any before it, can't [`build`](StepBuilder::build) based
    /// on the provided or constructed information. It also errors if the first step has a
    /// requisite, or the last a goal.
    #[expect(
        clippy::indexing_slicing,
        reason = "directly related to split points - if it fails, it should do so loudly."
    )]
    pub fn finish_in(
        &self,
        step: impl StepBuilder<P> + 'static,
        context: &BuildContext,
    ) -> Result<Technique<P>, TechniqueError> {
        let mut builders = self.step_builders.clone();
        builders.push(Arc::new(step));
        let last = builders.len() - 1;

        let mut steps = Vec::with_capacity(builders.len());
        for index in (0..builders.len()).rev() {
            let (builder_first, builder_second) = &mut builders.split_at_mut(index + 1);

            let builder = arc_make_mut(&mut builder_first[index]);

            if index != 0 && builder.assumed_solved().is_none() {
                *builder.assumed_solved() = builder
                    .allowed_sequences()
                    .map(|algs| PieceSet::from_algset(&algs));
            }

            if index == 0 && builder.assumed_solved().is_some() {
                return Err(TechniqueError::InvalidFirstStep);
            }

            if index == last {
                if builder.goal().is_some() {
                    return Err(TechniqueError::InvalidLastStep);
                }
            } else if builder.goal().is_none() {
                builder
                    .goal()
                    .clone_from(arc_make_mut(&mut builder_second[0]).assumed_solved());
            }

            steps.push(builder.build_in(context).map_err(|source| {
                TechniqueError::CantConstruct {
                    index: index + 1,
                    step: builder.name().to_string(),
                    source,
                }
            })?);
        }
        Ok(steps.into_iter().rev().collect())
    }

    /// Builds the technique using [`TechniqueBuilder::finish_in`] in the global context.
    ///
    /// # Errors
    /// Same as [`finish_in`](Self::finish_in)
    pub fn finish(
        &self,
        step: impl StepBuilder<P> + 'static,
    ) -> Result<Technique<P>, TechniqueError> {
        self.finish_in(step, &GLOBAL)
    }
}

impl<P: Puzzle> Index<usize> for Technique<P> {
    type Output = dyn Step<P>;
    #[expect(
        clippy::indexing_slicing,
        reason = "implementing indexing is supposed to be able to panic"
    )]
    fn index(&self, index: usize) -> &Self::Output {
        &*self.steps[index]
    }
}

impl<P: Puzzle> FromIterator<Arc<dyn Step<P>>> for Technique<P> {
    fn from_iter<T: IntoIterator<Item = Arc<dyn Step<P>>>>(iter: T) -> Self {
        Self {
            steps: iter.into_iter().collect(),
            // neutrality: Neutrality::default(),
        }
    }
}

impl<P: Puzzle> IntoIterator for Technique<P> {
    type IntoIter = <Vec<Arc<dyn Step<P>>> as IntoIterator>::IntoIter;
    type Item = Arc<dyn Step<P>>;
    fn into_iter(self) -> Self::IntoIter {
        self.steps.into_iter()
    }
}

impl<P: Puzzle> Technique<P> {
    /// Constructor for a technique: takes in any
    pub fn new(steps: impl IntoIterator<Item = Arc<dyn Step<P>>>) -> Self {
        Self {
            steps: steps.into_iter().collect(),
            // neutrality: Neutrality::default(),
        }
    }

    /// Creates a [`TechniqueBuilder`] with a starting given `step` given by [`StepBuilder`].
    ///
    /// It constructs [`assumed_solved`](StepBuilder::assumed_solved) to be empty. If
    /// [`assumed_solved`](StepBuilder::assumed_solved) is given in the builder, it will fail to
    /// construct the [`Technique`].
    pub fn start_with(step: impl StepBuilder<P> + 'static) -> TechniqueBuilder<P> {
        TechniqueBuilder {
            step_builders: vec![Arc::new(step)],
        }
    }

    /// Solves the provided [`Puzzle`], returning the solution.
    ///
    /// # Errors
    /// Errors if any step fails to start, to complete, or if it completes but isn't solved at the
    /// end.
    pub fn solve(&self, puzzle: &mut P) -> Result<Solution<P>, SolveError> {
        self.solve_steps(puzzle).collect()
    }

    /// Returns the steps of the Technique as an [`Iterator`](core::iter::Iterator).
    pub fn steps(&self) -> impl Iterator<Item = &dyn Step<P>> {
        self.steps.iter().map(|s| &**s)
    }

    /// Returns an [`Iterator`](core::iter::Iterator) over the solutions of every step which
    /// iterating over solves the [`puzzle`](crate::Puzzle).
    pub fn solve_steps(
        &self,
        puzzle: &mut P,
    ) -> impl Iterator<Item = Result<Solution<P>, SolveError>> {
        run_steps(self.steps.iter().cloned(), puzzle)
    }

    // Constructs the [`Technique`] with a given neutrality.
    // #[must_use]
    // fn with_neutrality(
    //     steps: impl IntoIterator<Item = Arc<dyn Step<P>>>,
    //     // neutrality: Neutrality<P>,
    // ) -> Self {
    //     Self {
    //         steps: steps.into_iter().collect(),
    //         // neutrality,
    //     }
    // }
    /*
    /// Sets the neutrality of the [`Technique`].{}
    #[must_use]
    pub fn neutrality(mut self, neutrality: Neutrality<P>) -> Self {
        self.neutrality = neutrality;
        self
    } */
}

/// Runs `steps` in order on `puzzle`, one per `next()`. Before each step it checks
/// [`can_solve`](Step::can_solve), after it [`is_done`](Step::is_done), and it yields nothing
/// after the first error.
fn run_steps<P: Puzzle>(
    steps: impl IntoIterator<Item = Arc<dyn Step<P>>>,
    puzzle: &mut P,
) -> impl Iterator<Item = Result<Solution<P>, SolveError>> {
    let mut failed = false;

    steps.into_iter().map_while(move |step| {
        if failed {
            return None;
        }
        if !step.can_solve(puzzle) {
            failed = true;
            return Some(Err(SolveError::requirements(
                step.name().to_string(),
                puzzle.to_string(),
            )));
        }

        let start = Instant::now();
        let result = match step.solve(puzzle) {
            Err(error) => Err(SolveError::couldnt_solve(step.name().to_string(), error)),
            Ok(_) if !step.is_done(puzzle) => Err(SolveError::not_done(step.name().to_string())),
            Ok(solution) => Ok(solution),
        };
        match &result {
            Ok(solution) => debug!(
                "{}: {} moves in {:.0?}: {}",
                step.name(),
                solution.move_count(),
                start.elapsed(),
                solution
                    .iter()
                    .map(|segment| segment
                        .moves()
                        .clone()
                        .iter()
                        .map(|m| m.to_string())
                        .collect::<Vec<_>>()
                        .join(" "))
                    .next()
                    .expect("solution has no segments")
            ),
            Err(e) => debug!("{}: {}", step.name(), e),
        }

        failed = result.is_err();
        Some(result)
    })
}

/// The main Method trait, implemented by specifying a conversion to [`Technique`] (a sequence of
/// steps) through [`to_technique`](Method::to_technique).
///
/// It can then be solved directly without conversion by using [`self.solve()`](Method::solve).
/// Methods in real life aren't a simple sequence of steps, but something that, depending on
/// parameters, collapses to a different [`Technique`]. Implementation of [`Method`] usually starts
/// by defining a type to hold such parameters, then how to get a sequence of steps from such
/// parameters.
pub trait Method<P: Puzzle>: Debug + Default {
    /// Converts a parametrized [`Method`] to a specific [`Technique`]
    fn to_technique(&self) -> Technique<P>;

    /// Constructs a default [`Method`].
    #[must_use]
    fn new() -> Self {
        Self::default()
    }

    /// Runs every step in order and joins their solutions: [`solve_steps`](Self::solve_steps),
    /// collected.
    ///
    /// # Errors
    /// The first [`SolveError`]. No later step runs, and `puzzle` is left as the failing step
    /// left it.
    fn solve(&self, puzzle: &mut P) -> Result<Solution<P>, SolveError>
    where
        Self: Sized,
    {
        self.solve_steps(puzzle).collect()
    }

    /// Solves `puzzle` one step at a time. Each call to `next()` runs the next step and yields
    /// its solution, so a caller can time a step or report progress before the next one runs.
    ///
    /// After each step the iterator checks the step's [`is_done`](Step::is_done). The first
    /// failure is yielded as an `Err`, and the iterator yields nothing after it. `puzzle` is
    /// left as the last step that ran left it.
    ///
    /// ```rust
    /// use rubiks_cube::{Cube3x3, Method, Roux};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let roux = Roux::default();
    /// let mut cube = Cube3x3::from_moves("R U R' F' L2 D B'")?;
    /// for segment in roux.solve_steps(&mut cube) {
    ///     print!("{}", segment?);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn solve_steps(&self, puzzle: &mut P) -> impl Iterator<Item = Result<Solution<P>, SolveError>>
    where
        Self: Sized,
    {
        run_steps(self.to_technique().steps, puzzle)
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::panic_in_result_fn,
        reason = "`?` reports setup failures; `assert!` reports the property under test failing"
    )]

    use std::{assert_matches, error::Error};

    use crate::{AlgSet, Cube3x3, Edge, PieceSet, SearchStep, SolveErrorKind, StepError};

    use super::{test_steps::FixedStep, *};

    fn fixed_failure() -> StepError {
        StepError::Custom("fixed failure".into())
    }

    #[test]
    fn a_method_step_that_returns_but_is_not_done_fails_the_solve_naming_it() {
        let never_done: Arc<dyn Step<Cube3x3>> =
            Arc::new(FixedStep::new("Never done", "", true, false));
        let method = Technique::new([never_done]);

        let result = method.solve(&mut Cube3x3::default());

        let Err(err) = result else {
            panic!("expected the solve to fail, got {result:?}");
        };
        assert_eq!(err.kind(), SolveErrorKind::NotDone);
        assert_eq!(err.step(), "Never done");
        assert!(err.source().is_none());
    }

    #[test]
    fn after_a_step_that_cannot_start_the_iterator_yields_nothing_more() {
        let later = FixedStep::new("Later", "", true, true);
        let method = Technique::new([
            Arc::new(FixedStep::new("Cannot start", "", false, true)) as Arc<dyn Step<Cube3x3>>,
            Arc::new(later.clone()),
        ]);
        let mut cube = Cube3x3::default();
        let mut steps = method.solve_steps(&mut cube);

        let Err(err) = steps.next().unwrap() else {
            panic!("there should be one step")
        };

        assert_eq!(err.kind(), SolveErrorKind::Requirements);
        assert_eq!(err.step(), "Cannot start");
        assert!(err.source().is_none());

        assert!(steps.next().is_none());
        assert_eq!(later.runs(), 0, "no step runs after an error");
    }

    #[test]
    fn search_step_rejects_a_cube_that_misses_its_before_without_searching() -> anyhow::Result<()> {
        let uf_solved = PieceSet::<Cube3x3>::from_pieces([Edge::Uf]);
        // No moves: if the step searched, it could not move and would return `UnreachableGoal`.
        let step = SearchStep::builder("UF then DF", AlgSet::new())
            .goal(PieceSet::<Cube3x3>::from_pieces([Edge::Uf, Edge::Df]))
            .expect_solved(uf_solved)
            .build()?;
        let scrambled = Cube3x3::from_moves("U")?;
        let mut cube = scrambled;

        let result = step.solve(&mut cube);

        assert!(
            matches!(result, Err(StepError::InvalidStartingState { .. })),
            "{result:?}"
        );
        assert_eq!(cube, scrambled, "a rejected step must not move the cube");
        Ok(())
    }

    #[test]
    fn a_step_that_returns_but_is_not_done_fails_the_solve_naming_it() {
        let never_done: Arc<dyn Step<Cube3x3>> =
            Arc::new(FixedStep::new("Never done", "", true, false));
        let method = Technique::new([never_done]);

        let result = method.solve(&mut Cube3x3::default());

        let Err(err) = result else {
            panic!("expected the solve to fail, got {result:?}");
        };
        assert_eq!(err.kind(), SolveErrorKind::NotDone);
        assert_eq!(err.step(), "Never done");
        assert!(err.source().is_none());
    }

    #[test]
    fn a_custom_step_error_comes_back_wrapped_and_naming_the_step() {
        let always_fails: Arc<dyn Step<Cube3x3>> =
            FixedStep::failing("Always fails", fixed_failure);
        let method = Technique::new([always_fails]);

        let result = method.solve(&mut Cube3x3::default());

        let Err(err) = result else {
            panic!("expected the solve to fail, got {result:?}");
        };
        assert_eq!(err.kind(), SolveErrorKind::CouldntSolve);
        assert_eq!(err.step(), "Always fails");
        let Some(StepError::Custom(custom)) = err.inner() else {
            panic!("expected a wrapped custom error, got {err:?}");
        };
        assert_eq!(custom.to_string(), "fixed failure");
    }

    #[test]
    fn each_next_runs_one_step_and_leaves_the_cube_after_it() -> anyhow::Result<()> {
        let uf_solved = PieceSet::<Cube3x3>::from_pieces([Edge::Uf]);
        let u_turns = AlgSet::from_static_parts("U");
        let solve_uf = SearchStep::builder("UF", u_turns)
            .goal(uf_solved.clone())
            .build()?;
        let later = FixedStep::new("Later", "", true, true);
        let method = Technique::new([
            Arc::new(solve_uf) as Arc<dyn Step<Cube3x3>>,
            Arc::new(later.clone()),
        ]);
        let mut cube = Cube3x3::from_moves("U")?;

        let first = method.solve_steps(&mut cube).next();

        assert!(matches!(first, Some(Ok(_))), "{first:?}");
        assert!(
            uf_solved.applies_to(&cube),
            "the first step's goal is met:\n{cube}"
        );
        assert_eq!(
            later.runs(),
            0,
            "taking one item must not run the next step"
        );
        Ok(())
    }

    #[test]
    fn after_an_error_the_iterator_yields_nothing_more() {
        let failing = FixedStep::failing("Fails", fixed_failure);
        let later = FixedStep::new("Later", "", true, true);
        let method = Technique::new([failing as Arc<dyn Step<Cube3x3>>, Arc::new(later.clone())]);
        let mut cube = Cube3x3::default();
        let mut steps = method.solve_steps(&mut cube);

        assert!(matches!(steps.next(), Some(Err(_))));
        assert!(steps.next().is_none());
        assert!(steps.next().is_none());
        assert_eq!(later.runs(), 0, "no step runs after an error");
    }

    #[test]
    fn final_step_with_goal_returns_error() {
        let technique =
            Technique::<Cube3x3>::start_with(SearchStep::builder("throwaway", AlgSet::new()))
                .finish(
                    SearchStep::builder("end", AlgSet::new())
                        .goal(PieceSet::from_algset(&AlgSet::new()))
                        .clone(),
                );
        assert_matches!(technique, Err(TechniqueError::InvalidLastStep));
    }

    #[test]
    fn starting_step_with_prereq_returns_error() {
        let technique = Technique::<Cube3x3>::start_with(
            SearchStep::builder("throwaway", AlgSet::new())
                .expect_solved(PieceSet::new())
                .clone(),
        )
        .finish(SearchStep::builder("end", AlgSet::new()));
        assert_matches!(technique, Err(TechniqueError::InvalidFirstStep));
    }

    #[test]
    fn invalid_intermediate_step_is_counted_correctly() {
        let technique =
            Technique::<Cube3x3>::start_with(SearchStep::builder("throwaway", AlgSet::new()))
                .then(
                    SearchStep::builder("2nd step", AlgSet::new())
                        .goal(PieceSet::new())
                        .clone(),
                )
                .finish(
                    SearchStep::builder("end", AlgSet::new())
                        .expect_solved(PieceSet::from_algset(&AlgSet::new()))
                        .clone(),
                );
        assert_matches!(
            technique,
            Err(TechniqueError::CantConstruct {
                index: 2,
                step: _,
                source: _
            })
        );
    }

    #[test]
    fn failed_to_start_returns_state() -> anyhow::Result<()> {
        let one_step_method = Technique::new(vec![Arc::new(
            SearchStep::builder("end", AlgSet::new())
                .expect_solved(PieceSet::from_algset(&AlgSet::new()))
                .clone()
                .build()?,
        ) as Arc<dyn Step<Cube3x3>>]);
        let mut cube = Cube3x3::scrambled_with_seed(1);
        let _cube_as_shown = format!("{cube}");
        println!("{cube}");
        assert_eq!(
            one_step_method
                .solve(&mut cube)
                .expect_err("purposefully erroed")
                .state(),
            Some(
                "    DRD\n    RUB\n    DBU\nFDB RLL FUF LBR\nRLF DFB DRL DBU\nRFB RRL DLL BLU\n    UFB\n    UDF\n    FUU"
            )
        );
        Ok(())
    }
}
