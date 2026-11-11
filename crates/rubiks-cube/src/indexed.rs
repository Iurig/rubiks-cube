//! The [`Indexed`] trait and its tools.

use std::fmt::Debug;

/// Trait for types with few variants that are all accessible from a comprehensive array. So they
/// can be iterated from, and a permutation can be expressed simply as another array.
pub trait Indexed: Clone + PartialEq + Debug {
    /// How many values the type can assume.
    const COUNT: usize;
    /// Where the value is indexed in the comprehensive array.
    fn index(&self) -> usize;
    /// What value lives in a given index of the array.
    ///
    /// # Panics
    /// Panics if the index is not in the range `0..COUNT`.
    fn from_index(index: usize) -> Self;

    /// Iterator over all values of the type.
    fn all() -> impl Iterator<Item = Self> + Clone {
        (0..Self::COUNT).map(Self::from_index)
    }
}

impl<A: Indexed, B: Indexed> Indexed for (A, B) {
    const COUNT: usize = A::COUNT * B::COUNT;
    fn index(&self) -> usize {
        self.0.index() * B::COUNT + self.1.index()
    }
    fn from_index(index: usize) -> Self {
        (
            A::from_index(index / B::COUNT),
            B::from_index(index % B::COUNT),
        )
    }
}

/// [usize]s in a range, as an [`Indexed`] type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangedInt<const BEGIN: usize, const END: usize>(usize);

impl<const BEGIN: usize, const END: usize> Indexed for RangedInt<BEGIN, END> {
    const COUNT: usize = END - BEGIN;
    fn index(&self) -> usize {
        self.0.strict_sub(BEGIN)
    }
    fn from_index(index: usize) -> Self {
        Self(BEGIN + index)
    }
}

/// Checks the round-trip law for every index of `T`: `from_index` and `index` undo each other.
#[cfg(test)]
pub(crate) fn assert_round_trips<T: Indexed + Eq + std::fmt::Debug>() {
    for i in 0..T::COUNT {
        let value = T::from_index(i);
        assert_eq!(
            value.clone().index(),
            i,
            "Total: {}\n Value: {value:?}",
            T::COUNT
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Three(usize);
    impl Indexed for Three {
        const COUNT: usize = 3;
        fn index(&self) -> usize {
            self.0
        }
        fn from_index(index: usize) -> Self {
            assert!(index < Self::COUNT, "{index} is not below {}", Self::COUNT);
            Self(index)
        }
    }

    #[test]
    fn a_pair_numbers_every_combination_once() {
        assert_eq!(<(Three, Three)>::COUNT, 9);
        assert_round_trips::<(Three, Three)>();
    }

    #[test]
    fn ranged_int_roundtrips() {
        assert_round_trips::<RangedInt<0, 40>>();
        assert_round_trips::<RangedInt<30, 40>>();
        assert_round_trips::<RangedInt<40, 40>>();
        assert_round_trips::<RangedInt<0, 0>>();
    }
}
