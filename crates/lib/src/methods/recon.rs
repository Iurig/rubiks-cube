use crate::{Algorithm, Method, MethodSolution, Puzzle, Step};

/// A (possibly incomplete) reconstruction of a solve.
///
/// Designed so, once a method has been specified, moves can be pushed into it using
/// [`push`](Reconstruction::push), and step separation is done automatically.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Reconstruction<P: Puzzle, M: Method<P>> {
    scramble: Algorithm<P>,
    solution: MethodSolution<P>,
    method: M,
}

impl<P: Puzzle, M: Method<P> + Clone> Clone for Reconstruction<P, M> {
    fn clone(&self) -> Self {
        Self {
            scramble: self.scramble().clone(),
            solution: self.solution().clone(),
            method: self.method.clone(),
        }
    }
}

impl<P: Puzzle, M: Method<P>> From<Reconstruction<P, M>> for MethodSolution<P> {
    fn from(value: Reconstruction<P, M>) -> Self {
        value.solution
    }
}

impl<P: Puzzle, M: Method<P>> Reconstruction<P, M> {
    /// Creates a new [`Reconstruction`].
    #[must_use]
    pub fn new(scramble: Algorithm<P>, method: M) -> Self {
        Self {
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
    #[expect(
        clippy::indexing_slicing,
        clippy::missing_panics_doc,
        reason = "panics on broken invariant only"
    )]
    pub fn push(&mut self, m: P::Move) {
        let mut candidate_step_index = self.solution.len();
        let cubes = (0..self.solution().len())
            .map(|index| {
                let mut cube = P::default().apply(&self.scramble);
                for i in 0..index {
                    cube = cube.apply(
                        &self
                            .solution()
                            .iter()
                            .nth(i)
                            .expect("solution doesn't have enough steps")
                            .clone()
                            .into(),
                    );
                }
                cube
            })
            .collect::<Vec<_>>();
        let step_to_push = self.solution.iter().enumerate().rev();
        for (index, solution) in step_to_push {
            let cube = cubes[index].apply(solution.moves());
            if !self.method.to_technique()[index].is_done(&cube) {
                candidate_step_index = index;
            }
            if !solution.is_empty() {
                break;
            }
        }
        self.solution[candidate_step_index].push(m);
    }

    /// Pushes all moves in the given iterator on the [`Reconstruction`], checking the appropriate
    /// step for each one.
    pub fn extend(&mut self, moves: impl IntoIterator<Item = P::Move>) {
        for m in moves {
            self.push(m);
        }
    }
}
