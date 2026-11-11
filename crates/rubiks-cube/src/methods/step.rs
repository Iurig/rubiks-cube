use crate::{
    AlgSet, PieceSet, Puzzle, Solution, StepError,
    error::BuildStepError,
    methods::context::{BuildContext, GLOBAL},
};

use dyn_clone::DynClone;

use std::{borrow::Cow, fmt::Debug, sync::Arc};

pub mod choose;
pub mod combine_pruned;
pub mod search_step;

/// One stage of a solving method, such as building the first block in Roux.
///
/// A [`Technique`](crate::Technique) runs its steps in order. Before each step,
/// [`can_solve`](Step::can_solve) is called to check that the requirements for the step are met,
/// and after each step [`is_done`](Step::is_done) is called to check if the step delivered what it
/// promised, and the solve stops with a [`SolveError`](crate::SolveError) if the step couldn't
/// start, failed, or its goal is not met.
///
/// The crate has two public step types: [`SearchStep`](crate::SearchStep) searches for its goal,
/// and [`Shortest`](crate::Shortest) runs several steps and keeps the shortest result. Implement
/// this trait to write another kind, such as a step that follows hand-written rules. A step is
/// `Send + Sync` so that one built step can be shared by many methods through an `Arc`.
pub trait Step<P: Puzzle>: Send + Sync + Debug {
    /// The step's name, as it appears in errors and in the [`Solution`] it returns.
    fn name(&self) -> &str;

    /// Whether `puzzle` meets this step's goal.
    fn is_done(&self, puzzle: &P) -> bool;

    /// Moves `puzzle` to a state that meets this step's goal and returns the moves it applied.
    ///
    /// The returned moves, applied to `puzzle` as it was, must give `puzzle` as this function
    /// leaves it. A printed [`Solution`] replays only if every step keeps this promise.
    ///
    /// # Errors
    /// A [`StepError`] when the step cannot start on `puzzle` or cannot reach its goal. The
    /// step may leave `puzzle` changed when it fails.
    fn solve(&self, puzzle: &mut P) -> Result<Solution<P>, StepError>;

    /// Whether `puzzle` meets this step's pre-requisites. Expensive for expensive-to-solve steps -
    /// override recommended for those.
    fn can_solve(&self, puzzle: &P) -> bool {
        let mut puzzle_copy = puzzle.clone();
        match self.solve(&mut puzzle_copy) {
            Ok(_) => self.is_done(&puzzle_copy),
            Err(_) => false,
        }
    }
}

/// A type that can build a [`Step`].
///
/// This trait is also used to build techniques with [`TechniqueBuilder`](crate::TechniqueBuilder)
/// on insufficient information that is deduceable in context of the build.
pub trait StepBuilder<P: Puzzle>: DynClone + Debug + Send + Sync {
    /// Used to read or write to a [`StepBuilder`]'s goal.
    ///
    /// Builders for [`Step`]s for which the goal cannot be represented as a [`PieceSet`] should
    /// always return [`None`].
    fn goal(&mut self) -> &mut Option<PieceSet<P>>;

    /// The name of the [`Step`] being built
    fn name(&self) -> Cow<'_, str>;

    /// Setter for the assumptions of the [`StepBuilder`].
    fn assumed_solved(&mut self) -> &mut Option<PieceSet<P>>;

    /// Getter for the move sequences a built step might use.
    fn allowed_sequences(&mut self) -> Option<AlgSet<P>>;

    /// Builds the step within a given [`BuildContext`].
    ///
    /// # Errors
    /// Errors if there is not enough information in the builder to build the step. The required
    /// information depends on the step and its [`StepBuilder`]'s implementation.
    fn build_in(&self, context: &BuildContext) -> Result<Arc<dyn Step<P>>, BuildStepError>;

    /// Builds the step in a global context.
    ///
    /// Steps built this way always share a cache.
    ///
    /// # Errors
    /// Same as [`build_in`](Self::build_in).
    fn build(&self) -> Result<Arc<dyn Step<P>>, BuildStepError> {
        self.build_in(&GLOBAL)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cube3x3, methods::test_steps::FixedStep};

    /// Forwards everything but `can_solve`, so the trait's default runs.
    #[derive(Debug)]
    struct DefaultCanSolve(FixedStep);

    impl Step<Cube3x3> for DefaultCanSolve {
        fn name(&self) -> &str {
            self.0.name()
        }
        fn is_done(&self, puzzle: &Cube3x3) -> bool {
            self.0.is_done(puzzle)
        }
        fn solve(&self, puzzle: &mut Cube3x3) -> Result<Solution<Cube3x3>, StepError> {
            self.0.solve(puzzle)
        }
    }

    #[test]
    fn by_default_a_step_can_solve_when_solving_leaves_it_done() {
        let cube = Cube3x3::default();
        let done = DefaultCanSolve(FixedStep::new("Done", "U", true, true));
        let not_done = DefaultCanSolve(FixedStep::new("Not done", "U", true, false));
        let failing = DefaultCanSolve(Arc::unwrap_or_clone(FixedStep::failing("Failing", || {
            StepError::UnreachableGoal
        })));

        assert!(done.can_solve(&cube));
        assert!(!not_done.can_solve(&cube));
        assert!(!failing.can_solve(&cube));
    }
}
