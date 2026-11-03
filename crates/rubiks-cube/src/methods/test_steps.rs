//! Hand-written steps for tests of the solve loop and of steps that hold other steps, and a
//! hand-written builder for tests of how a technique wires its steps.

use std::{
    borrow::Cow,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use crate::{
    AlgSet, Algorithm, BuildContext, BuildStepError, Cube3x3, Puzzle, PuzzleCondition, Solution,
    Step, StepBuilder, StepError,
};

/// A step whose result is fixed in advance. `solve` either returns the error from `error`,
/// or applies `moves` to the cube and returns them as one segment named after the step.
/// `is_done` reports `done`, whatever the cube. It counts how many times `solve` ran; clones
/// share the count.
#[derive(Clone, Debug)]
pub struct FixedStep {
    name: &'static str,
    moves: &'static str,
    can_solve: bool,
    done: bool,
    error: Option<fn() -> StepError>,
    runs: Arc<AtomicUsize>,
}

impl FixedStep {
    /// A step that applies `moves` (notation text; `""` for none) and reports `done`.
    pub fn new(name: &'static str, moves: &'static str, can_solve: bool, done: bool) -> Self {
        Self {
            name,
            moves,
            can_solve,
            done,
            error: None,
            runs: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// A step whose `solve` always returns `error()` and leaves the cube alone.
    pub fn failing(name: &'static str, error: fn() -> StepError) -> Arc<Self> {
        Arc::new(Self {
            name,
            moves: "",
            can_solve: true,
            done: false,
            error: Some(error),
            runs: Arc::new(AtomicUsize::new(0)),
        })
    }

    /// How many times `solve` ran.
    pub fn runs(&self) -> usize {
        self.runs.load(Ordering::Relaxed)
    }
}

impl Step<Cube3x3> for FixedStep {
    fn name(&self) -> &str {
        self.name
    }

    fn can_solve(&self, _: &Cube3x3) -> bool {
        self.can_solve
    }

    fn is_done(&self, _: &Cube3x3) -> bool {
        self.done
    }

    fn solve(&self, puzzle: &mut Cube3x3) -> Result<Solution<Cube3x3>, StepError> {
        self.runs.fetch_add(1, Ordering::Relaxed);
        if let Some(error) = self.error {
            return Err(error());
        }
        let moves = self
            .moves
            .parse::<Algorithm<Cube3x3>>()
            .map_err(|e| StepError::Custom(Box::new(e)))?;
        *puzzle = puzzle.apply(&moves);
        Ok(Solution::single_segment(self.name, moves))
    }
}

/// What a [`RecordingBuilder`] was told to build with.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Wired {
    pub prerequisite: Option<PuzzleCondition<Cube3x3>>,
    pub goal: Option<PuzzleCondition<Cube3x3>>,
}

/// A builder that reports whatever it is given and records what `build_wired` receives. Clones
/// share the record, so a test can keep a clone and read it after the builder is consumed.
#[derive(Clone, Debug)]
pub struct RecordingBuilder {
    name: &'static str,
    prerequisite: Option<PuzzleCondition<Cube3x3>>,
    goal: Option<PuzzleCondition<Cube3x3>>,
    sequences: Option<AlgSet<Cube3x3>>,
    received: Arc<Mutex<Option<Wired>>>,
}

impl RecordingBuilder {
    /// A builder with no prerequisite, no goal, and no sequences to derive a prerequisite from.
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            prerequisite: None,
            goal: None,
            sequences: None,
            received: Arc::default(),
        }
    }

    pub fn prerequisite(mut self, prerequisite: PuzzleCondition<Cube3x3>) -> Self {
        self.prerequisite = Some(prerequisite);
        self
    }

    pub fn goal(mut self, goal: PuzzleCondition<Cube3x3>) -> Self {
        self.goal = Some(goal);
        self
    }

    pub fn sequences(mut self, sequences: AlgSet<Cube3x3>) -> Self {
        self.sequences = Some(sequences);
        self
    }

    /// What the last `build_wired` received. Panics if it never ran.
    pub fn received(&self) -> Wired {
        self.received
            .lock()
            .unwrap()
            .clone()
            .expect("`build_wired` should have run")
    }
}

impl StepBuilder<Cube3x3> for RecordingBuilder {
    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.name)
    }

    fn get_prereq(&self) -> Option<PuzzleCondition<Cube3x3>> {
        self.prerequisite.clone()
    }

    fn get_goal(&self) -> Option<PuzzleCondition<Cube3x3>> {
        self.goal.clone()
    }

    fn allowed_sequences(&self) -> Option<AlgSet<Cube3x3>> {
        self.sequences.clone()
    }

    fn build_wired(
        &self,
        prerequisite: Option<PuzzleCondition<Cube3x3>>,
        goal: Option<PuzzleCondition<Cube3x3>>,
        _context: &BuildContext,
    ) -> Result<Arc<dyn Step<Cube3x3>>, BuildStepError> {
        *self.received.lock().unwrap() = Some(Wired { prerequisite, goal });
        Ok(Arc::new(FixedStep::new(self.name, "", true, true)))
    }
}
