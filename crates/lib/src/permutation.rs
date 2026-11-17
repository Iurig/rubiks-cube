//! The [`Permutation`] type and its tools.

use std::{hash::Hash, ops::Index, vec::IntoIter};

use crate::{Indexed, error::PermutationError};

/// Permutations of a [`Indexed`] type.
///
/// Like the type it permutes, it implements [`Indexed`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Permutation<T: Indexed>(Vec<T>);

impl<T: Indexed> Default for Permutation<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Indexed> Index<usize> for Permutation<T> {
    type Output = T;
    #[expect(
        clippy::indexing_slicing,
        reason = "an index out of range should panick"
    )]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T: Indexed> IntoIterator for Permutation<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: Indexed> Hash for Permutation<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Indexed::index(self).hash(state);
    }
}

impl<T: Indexed> Permutation<T> {
    /// Creates a permutation of the items ordered by index.
    #[must_use]
    pub fn new() -> Self {
        Self((0..T::COUNT).map(|index| T::from_index(index)).collect())
    }

    /// Constructor for [`Permutation<T>`] for iterators of `T` in order.
    pub fn try_from_iter(iter: impl IntoIterator<Item = T>) -> Result<Self, PermutationError> {
        let to_vec = iter.into_iter().collect::<Vec<_>>();
        let mut to_sorted_vec = to_vec.clone();
        to_sorted_vec.sort_by_key(<T as Indexed>::index);
        let mut repeats = false;
        for index in 0..(to_vec.len() - 1) {
            if to_sorted_vec.get(index).expect("index out of range")
                == to_sorted_vec.get(index + 1).expect("index out of range")
            {
                repeats = true;
                break;
            }
        }
        if to_vec.len() != T::COUNT {
            Err(PermutationError::WrongSize)
        } else if repeats {
            Err(PermutationError::RepeatedElement)
        } else {
            Ok(Self(to_vec))
        }
    }

    /// Swaps 2 positions in a permutation.
    pub fn swap_slots(&mut self, slot_1: usize, slot_2: usize) -> &mut Self {
        self.0.swap(slot_1, slot_2);
        self
    }

    /// Walks the given cycle.
    ///
    /// Puts each value in the position of the next in the iterator, finishing by putting the last
    /// where the first was.
    pub fn swap_cycle(&mut self, cycle: impl IntoIterator<Item = usize>) -> &mut Self {
        let mut walk = cycle.into_iter();
        if let Some(first) = walk.next() {
            for slot_2 in walk {
                self.swap_slots(first, slot_2);
            }
        }

        self
    }
}

pub(crate) const fn factorial(n: usize) -> usize {
    let mut decreasing = n;
    let mut resp = 1;
    while decreasing > 1 {
        resp *= decreasing;
        decreasing -= 1;
    }
    resp
}

impl<T: Indexed> Indexed for Permutation<T> {
    const COUNT: usize = factorial(T::COUNT);

    fn index(&self) -> usize {
        self.0
            .iter()
            .rev()
            .zip(1..=T::COUNT)
            .fold(0, |index, (value, base)| {
                index * base + value.index()
                    - self
                        .0
                        .iter()
                        .take(T::COUNT - base)
                        .filter(|previous_value| previous_value.index() < value.index())
                        .count()
            })
    }

    fn from_index(index: usize) -> Self {
        Self(
            (1..=T::COUNT)
                .rev()
                .fold(
                    (Vec::new(), index),
                    |(mut permutation, remaining_index), base| {
                        let value = T::all()
                            .filter(|v| !permutation.contains(v))
                            .nth(remaining_index % base)
                            .expect("invalid index");
                        permutation.push(value);
                        (permutation, remaining_index / base)
                    },
                )
                .0,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{
        indexed::{RangedInt, assert_round_trips},
        zn::Zn,
    };

    use super::*;

    #[test]
    fn factorial_test() {
        assert_eq!(factorial(1), 1);
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(4), 24);
    }

    #[test]
    fn permutations_of_ranged_round_trip() {
        assert_round_trips::<Permutation<RangedInt<0, 5>>>();
        assert_round_trips::<Permutation<RangedInt<30, 36>>>();
        assert_round_trips::<Permutation<RangedInt<40, 40>>>();
        assert_round_trips::<Permutation<RangedInt<0, 0>>>();
    }

    #[test]
    fn simple_cycles() {
        let mut perm = Permutation::<Zn<8>>::new();
        assert_eq!(
            *perm.swap_cycle([0, 1, 4]),
            Permutation::<Zn<8>>::try_from_iter(dbg!(
                [4, 0, 2, 3, 1, 5, 6, 7]
                    .iter()
                    .map(|&i| Zn::<8>::new(i))
                    .collect::<Vec<_>>()
            ))
            .unwrap()
        );
    }

    #[test]
    fn into_iter_and_try_from_iter_round_trops() -> Result<(), Box<dyn Error>> {
        let perm = Permutation::<Zn<8>>::try_from_iter(
            [4, 0, 2, 3, 1, 5, 6, 7].iter().map(|&i| Zn::<8>::new(i)),
        )?;
        assert_eq!(
            perm.clone(),
            Permutation::<Zn<8>>::try_from_iter(perm.into_iter())?
        );
        Ok(())
    }
}
