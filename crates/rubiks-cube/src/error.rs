use std::{error::Error, fmt::Display};

use thiserror::Error;

use crate::{PieceSet, Puzzle};

/// Why a [`Step`](crate::Step) could not solve.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum StepError {
    /// The step's moves cannot bring the puzzle to its goal.
    #[error("goal could not be reached with the given algset")]
    UnreachableGoal,
    /// The puzzle does not meet what the step needs before it starts, such as a
    /// [`SearchStep`](crate::SearchStep)'s `before` mask. [`Shortest`](crate::Shortest) skips an
    /// alternative that returns this.
    #[error("starting state doesn't fit expected properties:\n{state}")]
    InvalidStartingState {
        /// The puzzle the step was given, as its [`Display`] prints it.
        state: String,
    },
    #[error(transparent)]
    /// Any other error specific to the step type.
    Custom(Box<dyn Error + Send + Sync>),
}

/// Reasons a method's solve could've stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SolveErrorKind {
    /// The step's [`can_solve`](crate::Step::can_solve) returned false, so it never ran.
    Requirements,
    /// The step reported an error, available through [`inner`](SolveError::inner).
    CouldntSolve,
    /// The step returned a solution, but its [`is_done`](crate::Step::is_done) check failed.
    NotDone,
}

impl Display for SolveErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Requirements => {
                write!(f, "couldn't start because its requirements weren't met")
            }
            Self::CouldntSolve => write!(f, "couldn't be solved"),
            Self::NotDone => {
                write!(f, "finished, but its goal wasn't met")
            }
        }
    }
}

/// Why a method's solve stopped.
#[derive(Debug, Error)]
#[error(
    "step {step:?} {kind}{}",
    state.as_ref().map_or_else(String::new, |state| format!(":\n{state}"))
)]
pub struct SolveError {
    kind: SolveErrorKind,
    step: String,
    /// The puzzle the step was given, when [`kind`](Self::kind) is
    /// [`SolveErrorKind::Requirements`].
    state: Option<String>,
    source: Option<StepError>,
}

impl SolveError {
    /// The reason the solve stopped.
    #[must_use]
    pub const fn kind(&self) -> SolveErrorKind {
        self.kind
    }
    /// Which step stopped.
    #[must_use]
    pub fn step(&self) -> &str {
        &self.step
    }
    /// The puzzle the step was given, as its [`Display`] prints it, when [`kind`](Self::kind) is
    /// [`SolveErrorKind::Requirements`].
    #[must_use]
    pub fn state(&self) -> Option<&str> {
        self.state.as_deref()
    }
    /// What the step reported, when [`kind`](Self::kind) is [`SolveErrorKind::CouldntSolve`].
    #[must_use]
    pub const fn inner(&self) -> Option<&StepError> {
        self.source.as_ref()
    }
    /// Takes what the step reported, when [`kind`](Self::kind) is [`SolveErrorKind::CouldntSolve`].
    #[must_use]
    pub fn into_inner(self) -> Option<StepError> {
        self.source
    }

    /// The step's requirements weren't met, so it never ran.
    pub(crate) const fn requirements(step: String, state: String) -> Self {
        Self {
            kind: SolveErrorKind::Requirements,
            step,
            state: Some(state),
            source: None,
        }
    }
    /// The step ran and reported `source`.
    pub(crate) const fn couldnt_solve(step: String, source: StepError) -> Self {
        Self {
            kind: SolveErrorKind::CouldntSolve,
            step,
            state: None,
            source: Some(source),
        }
    }
    /// The step returned a solution, but its goal isn't met.
    pub(crate) const fn not_done(step: String) -> Self {
        Self {
            kind: SolveErrorKind::NotDone,
            step,
            state: None,
            source: None,
        }
    }
}

/// Why one move failed to parse.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ParseMoveError {
    /// Reachable only through `"".parse::<Move3x3>()`; a sequence parse never tries to parse an
    /// empty move, so it carries no offending text.
    #[error("empty string cannot be parsed into moves")]
    EmptyString,
    /// The text after the part is not a modifier.
    #[error("{modifier} is not a valid modifier in {invalid_move}")]
    BadModifier {
        /// The whole invalid move.
        invalid_move: String,
        /// The text that failed as a modifier.
        modifier: String,
    },
    /// The text before the modifier is not a part.
    #[error("{part} in {invalid_move} is not a valid part")]
    BadPart {
        /// The whole invalid move.
        invalid_move: String,
        /// The text that failed as a part.
        part: String,
    },
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
/// A move in a sequence failed to parse; `line` and `position` count from 1.
#[error("failed to parse move at line {line}, position {position}")]
pub struct ParseSequenceError {
    pub(crate) source: ParseMoveError,
    pub(crate) line: usize,
    pub(crate) position: usize,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
/// Attempted to construct a [`SearchStep`](crate::SearchStep) with invalid
/// [`SearchStepBuilder`](crate::SearchStepBuilder).
#[non_exhaustive]
pub enum SearchStepError<P: Puzzle> {
    /// The step prerequisite was not guaranteed by the goal.
    #[error(
        "expected {before} to be solved, but the previous goal only promisses {after} to be solved"
    )]
    InvalidPrerequisite {
        /// The prerequisite
        before: PieceSet<P>,
        /// The goal
        after: PieceSet<P>,
    },
    /// Prerequisite missing.
    #[error("the builder had no prerequisite")]
    MissingPrerequisite,
}
#[derive(Debug, Error)]
/// Attempted to construct [`Step`](crate::Step) failed.
#[non_exhaustive]
pub enum BuildStepError {
    /// The step being built required a goal to be defined, but either the
    /// [`StepBuilder`](crate::StepBuilder) or the [`TechniqueBuilder`](crate::TechniqueBuilder)
    /// didn't provide one.
    #[error("the step needs a goal to be built, but one wasn't available")]
    GoalNeeded,
    /// This step type cannot be constructed from the goal type provided.
    #[error("this step kind cannot receive goals of this shape")]
    InvalidGoal,
    /// The step being built required a prerequisite to be defined, but either the
    /// [`StepBuilder`](crate::StepBuilder) or the [`TechniqueBuilder`](crate::TechniqueBuilder)
    /// didn't provide one.
    #[error("the step needs a prerequisite to be built, but one wasn't available")]
    RequisiteNeeded,
    /// This step type cannot be constructed from the requisite type provided.
    #[error("this step kind cannot receive prerequisites of this shape")]
    InvalidRequisite,
    /// Error specific to this particular [`StepBuilder`](crate::StepBuilder).
    #[error(transparent)]
    Custom(Box<dyn Error + Send + Sync>),
}

#[derive(Debug, Error)]
/// Attempted to construct a [`Technique`](crate::Technique) with invalid
/// [`TechniqueBuilder`](crate::TechniqueBuilder).
#[non_exhaustive]
pub enum TechniqueError {
    /// The first step had a prerequisite.
    #[error("the first step should not have set requirements")]
    InvalidFirstStep,
    /// Any step failed to build.
    #[error(
        "{step}, the {} step in the technique, could not be built",
        crate::ordinal::ordinal(*index)
    )]
    CantConstruct {
        /// The failed step's position, 1-indexed.
        index: usize,
        /// The failed step's name.
        step: String,
        /// The reason it failed to build
        #[source]
        source: BuildStepError,
    },
    /// The last step had a goal.
    #[error("the last step should not have set goal")]
    InvalidLastStep,
}

impl ParseSequenceError {
    /// The 1-based line of the invalid move.
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }
    /// The 1-based position of the invalid move within its line.
    #[must_use]
    pub const fn position(&self) -> usize {
        self.position
    }
    /// The same error on `line`, for text parsed one line at a time.
    pub(crate) fn on_line(self, line: usize) -> Self {
        Self { line, ..self }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn solve_error_source_works() {
        let solve_error = SolveError::couldnt_solve("name".into(), StepError::UnreachableGoal);

        let source = solve_error.source().expect("a step error has a source");

        assert!(matches!(
            source.downcast_ref::<StepError>(),
            Some(StepError::UnreachableGoal)
        ));
    }

    #[test]
    fn solve_error_gives_back_the_step_error() {
        let solve_error = SolveError::couldnt_solve("name".into(), StepError::UnreachableGoal);

        assert!(matches!(
            solve_error.into_inner(),
            Some(StepError::UnreachableGoal)
        ));
    }
}
