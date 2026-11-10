//! The [`Permutation`] type and its tools.

use crate::Indexed;

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

impl<T: Indexed> Permutation<T> {
    /// Creates a permutation of the items ordered by index.
    #[must_use]
    pub fn new() -> Self {
        Self((0..T::COUNT).map(|index| T::from_index(index)).collect())
    }

    /// Swaps 2 positions in a permutation.
    pub fn swap_slots(&mut self, slot_1: usize, slot_2: usize) -> Option<&mut Self> {
        self.0.swap(slot_1, slot_2);
        Some(self)
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

impl<T: Indexed> Indexed for Permutation<T> {
    const COUNT: usize = {
        let mut decreasing = T::COUNT;
        let mut resp = 1;
        while decreasing > 1 {
            resp *= decreasing;
            decreasing -= 1;
        }
        resp
    };

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
                            .unwrap_or_else(|| {
                                panic!(
                                    "invalid index, was {}, shouldn't surpass {}. vector:{:?}, {}",
                                    remaining_index % base,
                                    base,
                                    permutation,
                                    Self::COUNT
                                )
                            });
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
    use crate::indexed::{RangedInt, assert_round_trips};

    use super::*;

    #[test]
    fn permutations_of_ranged_round_trip() {
        assert_round_trips::<Permutation<RangedInt<0, 5>>>();
        assert_round_trips::<Permutation<RangedInt<30, 36>>>();
        assert_round_trips::<Permutation<RangedInt<40, 40>>>();
        assert_round_trips::<Permutation<RangedInt<0, 0>>>();
    }
}
