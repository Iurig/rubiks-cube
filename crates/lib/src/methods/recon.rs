use crate::{Algorithm, FromAlg, Method, MethodSolution, Puzzle, Step, Technique};

use std::{fmt::Display, hash::Hash};

/// A (possibly incomplete) reconstruction of a solve.
///
/// Designed so, once a method has been specified, moves can be pushed into it using
/// [`push`](Reconstruction::push), and step separation is done automatically.
#[derive(Debug)]
pub struct Reconstruction<P: Puzzle, M: Method<P>> {
    scramble: Algorithm<P>,
    solution: MethodSolution<P>,
    method: M,
    puzzle: P,
    technique: Technique<P>,
}

impl<P: Puzzle, M: Method<P> + PartialEq> PartialEq for Reconstruction<P, M> {
    fn eq(&self, other: &Self) -> bool {
        self.scramble.eq(&other.scramble)
            && self.solution.eq(&other.solution)
            && self.method.eq(&other.method)
    }
}

impl<P: Puzzle, M: Method<P> + Eq> Eq for Reconstruction<P, M> {}

impl<P: Puzzle, M: Method<P> + Hash> Hash for Reconstruction<P, M> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.scramble.hash(state);
        self.solution.hash(state);
        self.method.hash(state);
    }
}

impl<P: Puzzle, M: Method<P> + Clone> Clone for Reconstruction<P, M> {
    fn clone(&self) -> Self {
        Self {
            scramble: self.scramble().clone(),
            solution: self.solution().clone(),
            method: self.method.clone(),
            puzzle: self.puzzle.clone(),
            technique: self.technique.clone(),
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

/// Pushes all moves in the given iterator on the [`Reconstruction`], checking the appropriate
/// step for each one.
impl<P: Puzzle, M: Method<P>> Extend<P::Move> for Reconstruction<P, M> {
    fn extend<I: IntoIterator<Item = P::Move>>(&mut self, iter: I) {
        for m in iter {
            self.push(m);
        }
    }
}

impl<P: Puzzle, M: Method<P>> Reconstruction<P, M> {
    /// Creates a new [`Reconstruction`].
    #[must_use]
    pub fn new(scramble: Algorithm<P>, method: M) -> Self {
        let technique = method.to_technique();
        Self {
            puzzle: P::from_alg(&scramble),
            scramble,
            solution: MethodSolution::with_empty_steps(technique.steps().map(Step::name)),
            method,
            technique,
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
        let mut candidate_step_index = self
            .solution
            .iter()
            .enumerate()
            .rev()
            .find(|(_, s)| !s.is_empty())
            .map_or(0, |(i, _)| i);

        while self.technique[candidate_step_index].is_done(&self.puzzle) {
            candidate_step_index += 1;
        }

        self.solution.index_mut(candidate_step_index).push(m);
        self.puzzle = self.puzzle.clone() * m;
    }
}
