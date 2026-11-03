pub mod context;
pub mod cube3x3;
// pub mod neutrality;
mod solution;
pub mod step;
#[cfg(test)]
mod test_steps;

pub use solution::{Segment, Solution};

use std::{fmt::Debug, ops::Index, sync::Arc};

use crate::{
    PieceSet, SolveError, TechniqueError,
    methods::{
        context::{BuildContext, GLOBAL},
        // neutrality::Neutrality,
        step::{Step, StepBuilder},
    },
    puzzles::Puzzle,
};

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
enum ConditionStates<P: Puzzle> {
    Pieces(PieceSet<P>),
    Opaque,
}

/// The types of puzzle conditions steps either require or guarantee - usually a set of pieces and
/// orientations to be solved, but possibly an opaque, more complex state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PuzzleCondition<P: Puzzle>(ConditionStates<P>);

impl<P: Puzzle> From<PieceSet<P>> for PuzzleCondition<P> {
    fn from(value: PieceSet<P>) -> Self {
        Self::pieces(value)
    }
}

impl<P: Puzzle> PuzzleCondition<P> {
    /// Creates a [`PuzzleCondition`] that describes the pieces in `pieces` as solved.
    #[must_use]
    pub const fn pieces(pieces: PieceSet<P>) -> Self {
        Self(ConditionStates::Pieces(pieces))
    }

    /// Creates a [`PuzzleCondition`] that doesn't describe the state directly. Used on
    /// [`StepBuilder`]s that define their puzzle conditions some other way than by pieces.
    #[must_use]
    pub const fn opaque() -> Self {
        Self(ConditionStates::Opaque)
    }

    /// Returns the pieces that describe the [`PuzzleCondition`], if it can be defined as a
    /// [`PieceSet`].
    #[must_use]
    pub fn as_pieces(&self) -> Option<PieceSet<P>> {
        match self.0.clone() {
            ConditionStates::Pieces(pieces) => Some(pieces),
            _ => None,
        }
    }
}

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
/// Can be used to build a [`Technique`] from incomplete [`StepBuilder`]s.
///
/// # Examples
///
/// ```ignore
/// use rubiks_cube::{AlgSet, SearchStep, Technique, PieceSet, Cube3x3};
///
/// let eolr = *SearchStep::builder("EOLR", AlgSet::from_static_parts("M U"))
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
///     *SearchStep::builder(
///        "CMLL",
///         AlgSet::from_static_parts(concat!(include_str!("path/to/algs/"))),
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
    // neutrality: Neutrality<P>,
}

impl<P: Puzzle> TechniqueBuilder<P> {
    /// Adds a [`Step`] to the final technique, constructed from a [`StepBuilder`].
    ///
    /// It constructs the step's goal to be the next step's
    /// [`get_prereq`](StepBuilder::get_prereq) and its own
    /// [`get_prereq`](StepBuilder::get_prereq) based on
    /// its [`allowed_sequences`](StepBuilder::allowed_sequences). If either can't be constructed
    /// because the [`StepBuilder`] doesn't provide one, it must be provided.
    pub fn then(&mut self, step: impl StepBuilder<P> + 'static) -> &mut Self {
        self.step_builders.push(Arc::new(step));
        self
    }

    /// Adds a given [`Neutrality`] to a [`TechniqueBuilder`].
    /*pub fn neutrality(&mut self, neutrality: Neutrality<P>) -> &mut Self {
        self.neutrality = neutrality;
        self
    }*/

    /// Adds the final [`Step`] of a technique.
    ///
    /// It constructs its [`get_prereq`](StepBuilder::get_prereq) based on its
    /// [`allowed_sequences`](StepBuilder::allowed_sequences).
    ///
    /// # Errors
    ///
    /// Errors if the provided `step`, or any before it, can't [`build`](StepBuilder::build) based
    /// on the provided or constructed information. It also errors if the first step has a
    /// requisite, or the last a goal.
    pub fn finish_in(
        &self,
        step: impl StepBuilder<P> + 'static,
        context: &BuildContext,
    ) -> Result<Technique<P>, TechniqueError> {
        let mut builders = self.step_builders.clone();
        builders.push(Arc::new(step));
        let last = builders.len() - 1;

        let mut steps = Vec::with_capacity(builders.len());
        let mut next_prereq = None;
        for (index, builder) in builders.iter().enumerate().rev() {
            if index == 0 && builder.get_prereq().is_some() {
                return Err(TechniqueError::InvalidFirstStep);
            }
            if index == last && builder.get_goal().is_some() {
                return Err(TechniqueError::InvalidLastStep);
            }

            let prereq = (index != 0)
                .then_some(builder.get_prereq().or_else(|| {
                    builder
                        .allowed_sequences()
                        .map(|algs| PieceSet::from_algset(&algs).into())
                }))
                .flatten();

            let goal = builder.get_goal().or(next_prereq);

            steps.push(
                builder
                    .build_wired(prereq.clone(), goal, context)
                    .map_err(|source| TechniqueError::CantConstruct {
                        index: index + 1,
                        step: builder.name().to_string(),
                        source,
                    })?,
            );
            next_prereq = prereq;
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
    /// It constructs [`get_prereq`](StepBuilder::get_prereq) to be empty. If
    /// [`get_prereq`](StepBuilder::get_prereq) is given in the builder, it will fail to
    /// construct the [`Technique`].
    pub fn start_with(step: impl StepBuilder<P> + 'static) -> TechniqueBuilder<P> {
        TechniqueBuilder {
            step_builders: vec![Arc::new(step)],
            // neutrality: Neutrality::default(),
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
    /*#[must_use]
    pub fn with_neutrality(
        steps: impl IntoIterator<Item = Arc<dyn Step<P>>>,
        neutrality: Neutrality<P>,
    ) -> Self {
        Self {
            steps: steps.into_iter().collect(),
            neutrality,
        }
    }

    /// Sets the neutrality of the [`Technique`].
    #[must_use]
    pub fn neutrality(mut self, neutrality: Neutrality<P>) -> Self {
        self.neutrality = neutrality;
        self
    }*/
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
        let result = match step.solve(puzzle) {
            Err(error) => Err(SolveError::couldnt_solve(step.name().to_string(), error)),
            Ok(_) if !step.is_done(puzzle) => Err(SolveError::not_done(step.name().to_string())),
            Ok(solution) => Ok(solution),
        };
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
pub trait Method<P: Puzzle>: Debug {
    /// Converts a parametrized [`Method`] to a specific [`Technique`] in the given
    /// [`BuildContext`].
    fn to_technique_in(&self, context: &BuildContext) -> Technique<P>;

    /// Converts a parametrized [`Method`] to a specific [`Technique`] in the global
    /// [`BuildContext`].
    fn to_technique(&self) -> Technique<P> {
        self.to_technique_in(&GLOBAL)
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

    use super::{
        test_steps::{FixedStep, RecordingBuilder, Wired},
        *,
    };

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
            .prerequisite(uf_solved)
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
                .prerequisite(PieceSet::new())
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
                        .prerequisite(PieceSet::from_algset(&AlgSet::new()))
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
                .prerequisite(PieceSet::from_algset(&AlgSet::new()))
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

    /// What `parts` cannot touch, as a condition: the prerequisite `finish` derives from them.
    fn untouched_by(parts: &'static str) -> PuzzleCondition<Cube3x3> {
        PieceSet::from_algset(&AlgSet::from_static_parts(parts)).into()
    }

    fn wired(
        prerequisite: Option<PuzzleCondition<Cube3x3>>,
        goal: Option<PuzzleCondition<Cube3x3>>,
    ) -> Wired {
        Wired { prerequisite, goal }
    }

    #[test]
    fn each_step_is_wired_to_keep_what_the_next_step_cannot_touch() {
        let first = RecordingBuilder::new("First").sequences(AlgSet::from_static_parts("F U"));
        let second = RecordingBuilder::new("Second").sequences(AlgSet::from_static_parts("R U"));
        let third = RecordingBuilder::new("Third").sequences(AlgSet::from_static_parts("U"));

        let technique = Technique::start_with(first.clone())
            .then(second.clone())
            .finish(third.clone())
            .unwrap();

        assert_eq!(first.received(), wired(None, Some(untouched_by("R U"))));
        assert_eq!(
            second.received(),
            wired(Some(untouched_by("R U")), Some(untouched_by("U")))
        );
        assert_eq!(third.received(), wired(Some(untouched_by("U")), None));
        let names: Vec<&str> = technique.steps().map(Step::name).collect();
        assert_eq!(names, ["First", "Second", "Third"]);
    }

    #[test]
    fn explicit_conditions_win_over_derived_ones() {
        let prerequisite = untouched_by("F");
        let goal = untouched_by("B");
        let first = RecordingBuilder::new("First").sequences(AlgSet::from_static_parts("F U"));
        let second = RecordingBuilder::new("Second")
            .sequences(AlgSet::from_static_parts("R U"))
            .prerequisite(prerequisite.clone())
            .goal(goal.clone());
        let third = RecordingBuilder::new("Third").sequences(AlgSet::from_static_parts("U"));

        Technique::start_with(first.clone())
            .then(second.clone())
            .finish(third)
            .unwrap();

        assert_eq!(first.received(), wired(None, Some(prerequisite.clone())));
        assert_eq!(second.received(), wired(Some(prerequisite), Some(goal)));
    }

    #[test]
    fn opaque_conditions_are_kept_and_reach_the_step_before() {
        let first = RecordingBuilder::new("First").sequences(AlgSet::from_static_parts("F U"));
        let second = RecordingBuilder::new("Second")
            .sequences(AlgSet::from_static_parts("R U"))
            .prerequisite(PuzzleCondition::opaque())
            .goal(PuzzleCondition::opaque());
        let third = RecordingBuilder::new("Third").sequences(AlgSet::from_static_parts("U"));

        Technique::start_with(first.clone())
            .then(second.clone())
            .finish(third)
            .unwrap();

        assert_eq!(
            first.received(),
            wired(None, Some(PuzzleCondition::opaque()))
        );
        assert_eq!(
            second.received(),
            wired(
                Some(PuzzleCondition::opaque()),
                Some(PuzzleCondition::opaque())
            )
        );
    }

    #[test]
    fn a_step_with_no_sequences_has_nothing_to_derive_a_prerequisite_from() {
        let first = RecordingBuilder::new("First").sequences(AlgSet::from_static_parts("F U"));
        let second = RecordingBuilder::new("Second");
        let third = RecordingBuilder::new("Third").sequences(AlgSet::from_static_parts("U"));

        Technique::start_with(first.clone())
            .then(second.clone())
            .finish(third)
            .unwrap();

        assert_eq!(first.received(), wired(None, None));
        assert_eq!(second.received(), wired(None, Some(untouched_by("U"))));
    }
}
