use crate::{AlgSet, Indexed, Labeled, Marker, Puzzle, fast_hash::FxHasher};

use std::hash::{BuildHasherDefault, Hash};

use rayon::prelude::*;

#[derive(Clone, Debug)]
pub struct PruneTable<K: Eq + Hash> {
    table: papaya::HashMap<K, u8, BuildHasherDefault<FxHasher>>,
}

impl<K: Eq + Hash> FromIterator<(K, u8)> for PruneTable<K> {
    fn from_iter<T: IntoIterator<Item = (K, u8)>>(iter: T) -> Self {
        let table = papaya::HashMap::default();
        for (state, distance) in iter {
            table.pin().insert(state, distance);
        }
        Self { table }
    }
}

impl<P: Puzzle, L: Marker<P, Label: Send + Sync> + Clone + Eq + Hash + Send + Sync>
    PruneTable<Labeled<P, L>>
{
    /// The distance from `goal` of every state `moveset` can reach, with `goal` at 0.
    pub(crate) fn from_goal(goal: &Labeled<P, L>, moveset: &AlgSet<P>) -> Self {
        let table = Self::from_iter([(goal.clone(), 0)]);
        table.populate(moveset);
        table
    }
    /// Fills the table outward from the keys already in it. Each sequence of `moveset` is one
    /// step of distance, the same unit [`PrunedCombine`](super::PrunedCombine) searches in.
    #[expect(
        clippy::indexing_slicing,
        reason = "vector made to always comport the length: breaking this invariant should fail loudly"
    )]
    pub(crate) fn populate(&self, algset: &AlgSet<P>) {
        // Every key in the frontier is at `depth`, so the depth lives once, outside the loop.
        let Some(&first_depth) = self.table.pin().values().next() else {
            return;
        };
        debug_assert!(
            self.table.pin().values().all(|&d| d == first_depth),
            "the keys already in the table must all be at the same distance"
        );
        let mut depth = first_depth;
        let mut frontier: Vec<Labeled<P, L>> = self.table.pin().keys().cloned().collect();
        let moves_as_puzzles = <P::Move as Indexed>::all()
            .map(|m| P::default() * m)
            .collect::<Vec<_>>();

        while !frontier.is_empty() {
            log::debug!("depth = {depth}, frontier = {}", frontier.len());
            depth += 1;

            frontier = frontier
                .par_iter()
                .flat_map_iter(|representation| {
                    algset
                        .algs()
                        .iter()
                        .map(|alg| {
                            alg.iter().fold(representation.clone(), |rep, m| {
                                rep.composed_with(&moves_as_puzzles[m.index()])
                            })
                        })
                        .filter(|state| !self.table.pin().contains_key(state))
                })
                .filter_map(|state| {
                    self.table
                        .pin()
                        .try_insert(state.clone(), depth)
                        .is_ok()
                        .then_some(state)
                })
                .collect();
        }
    }
}

impl<K: Clone + Eq + Hash> PruneTable<K> {
    pub(crate) fn get_distance(&self, state: &K) -> Option<u8> {
        self.table.pin().get(state).copied()
    }
}
