use crate::{Algorithm, FromAlg, Method, MethodSolution, Puzzle, Step, Technique};

use std::fmt::Display;

/// A (possibly incomplete) reconstruction of a solve.
///
/// Designed so, once a method has been specified, moves can be pushed into it using
/// [`push`](Reconstruction::push), and step separation is done automatically.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Reconstruction<P: Puzzle, M: Method<P>> {
    scramble: Algorithm<P>,
    solution: MethodSolution<P>,
    method: M,
    puzzle: P,
}

impl<P: Puzzle, M: Method<P> + Clone> Clone for Reconstruction<P, M> {
    fn clone(&self) -> Self {
        Self {
            scramble: self.scramble().clone(),
            solution: self.solution().clone(),
            method: self.method.clone(),
            puzzle: self.puzzle.clone(),
        }
    }
}

impl<P: Puzzle, M: Method<P>> From<Reconstruction<P, M>> for MethodSolution<P> {
    fn from(value: Reconstruction<P, M>) -> Self {
        value.solution
    }
}

impl<P: Puzzle, M: Method<P>> Display for Reconstruction<P, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}", self.scramble(), self.solution())
    }
}

impl<P: Puzzle, M: Method<P>> Reconstruction<P, M> {
    /// Creates a new [`Reconstruction`].
    #[must_use]
    pub fn new(scramble: Algorithm<P>, method: M) -> Self {
        Self {
            puzzle: P::from_alg(&scramble),
            scramble,
            solution: MethodSolution::with_empty_steps(
                method.to_technique().steps().map(Step::name),
            ),
            method,
        }
    }

    /// Getter for the scramble of the [`Reconstruction`].
    pub const fn scramble(&self) -> &Algorithm<P> {
        &self.scramble
    }

    /// Getter for the (possibly partial) solution of the [`Reconstruction`].
    pub const fn solution(&self) -> &MethodSolution<P> {
        &self.solution
    }

    /// Getter for the method used in the [`Reconstruction`].
    pub const fn method(&self) -> &M {
        &self.method
    }

    /// Pushes a [`Puzzle`]s move at the end of the reconstruction, checking which step that applies
    /// to automatically using [`Reconstruction::method`].
    pub fn push(&mut self, m: P::Move) {
        let technique = self.method.to_technique();
        self.push_with_technique(m, &technique);
    }

    fn push_with_technique(&mut self, m: P::Move, technique: &Technique<P>) {
        let mut candidate_step_index = self
            .solution
            .iter()
            .enumerate()
            .rev()
            .find(|(_, s)| !s.is_empty())
            .map_or(0, |(i, _)| i);

        while technique[candidate_step_index].is_done(&self.puzzle) {
            candidate_step_index += 1;
        }

        self.solution.index_mut(candidate_step_index).push(m);
        self.puzzle = self.puzzle.clone() * m;
    }

    /// Pushes all moves in the given iterator on the [`Reconstruction`], checking the appropriate
    /// step for each one.
    ///
    /// More performant than repeated [`push`](Reconstruction::push)ing.
    pub fn extend(&mut self, moves: impl IntoIterator<Item = P::Move>) {
        let technique = self.method.to_technique();
        for m in moves {
            self.push_with_technique(m, &technique);
        }
    }
}
