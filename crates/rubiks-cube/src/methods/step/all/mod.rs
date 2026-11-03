use std::{
    borrow::Cow,
    sync::{Arc, Mutex},
};

use crate::{Puzzle, Step, StepBuilder};

#[derive(Debug, Clone)]
struct All<P: Puzzle> {
    steps: Box<[Box<dyn StepBuilder<P>>]>,
    name: Cow<'static, str>,
}

impl<P: Puzzle> Step<P> for All<P> {
    fn can_solve(&self, puzzle: &P) -> bool {
        let mut mutable_self = self.clone();
        mutable_self.steps.iter_mut().any(|s| {
            s.assumed_solved()
                .clone()
                .is_some_and(|goal| goal.applies_to(puzzle))
        })
    }

    fn is_done(&self, puzzle: &P) -> bool {
        self.steps.iter().all(|s| s.is_done(puzzle))
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn solve(&self, puzzle: &mut P) -> Result<crate::Solution<P>, crate::StepError> {}
}
