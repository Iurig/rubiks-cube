use std::str::FromStr;

use itertools::Itertools;

use crate::{Mask, ParseSequenceError, Puzzle};

#[derive(Debug, Clone)]
pub struct Neutrality<P: Puzzle> {
    puzzles: Vec<P>,
}

impl<P: Puzzle> Default for Neutrality<P> {
    fn default() -> Self {
        Self::new()
    }
}
impl<P: Puzzle> Neutrality<P> {
    pub fn new() -> Self {
        Self {
            puzzles: Vec::from([P::default()]),
        }
    }

    pub fn valid_neutralities(&self, solved: &Mask<P>) -> Vec<P> {
        self.puzzles
            .iter()
            .filter(|&puzzle| solved.applies_to(puzzle))
            .cloned()
            .collect()
    }

    pub fn from_moves(moves: AlgTree<P>) -> Self {
        Self {
            puzzles: moves
                .into_iter()
                .fold(Vec::from_iter([P::default()]), |s, f| {
                    s.into_iter()
                        .cartesian_product(f.iter())
                        .map(|(p, m)| p * *m)
                        .collect()
                })
                .into_iter()
                .collect(),
        }
    }
}

#[derive(Debug)]
pub struct AlgTree<P: Puzzle> {
    tree: Vec<Vec<P::Move>>,
}

impl<P: Puzzle> FromStr for AlgTree<P> {
    type Err = ParseSequenceError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}
impl<P: Puzzle> IntoIterator for AlgTree<P> {
    type Item = Vec<P::Move>;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.tree.into_iter()
    }
    // add code here
}
impl<P: Puzzle> AlgTree<P> {
    pub(crate) fn iter(&self) -> impl Iterator<Item = &Vec<P::Move>> {
        self.tree.iter()
    }
}
