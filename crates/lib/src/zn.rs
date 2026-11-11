//! Modular integers for piece orientation.
use std::{
    fmt::Debug,
    iter::{Product, Sum},
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use crate::Indexed;

/// Integers mod `N`, stored as the representative in `0..N`.
///
/// The representative fits a byte: a twist is at most 2 and a flip at most
/// 1, and `N` is capped at 256 so any modulus this crate could want still
/// fits. The public face works in `usize`; the byte never leaks.
#[derive(Clone, Copy, Default, Eq, Hash, PartialEq)]
pub struct Zn<const N: usize>(u8);

impl<const N: usize> Debug for Zn<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl<const N: usize> Neg for Zn<N> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(N - self.value())
    }
}
impl<const N: usize> Add for Zn<N> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.value() + rhs.value())
    }
}
impl<const N: usize> Sub for Zn<N> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(N + self.value() - rhs.value())
    }
}
impl<const N: usize> Mul for Zn<N> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.value() * rhs.value())
    }
}
impl<const N: usize> AddAssign for Zn<N> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
impl<const N: usize> SubAssign for Zn<N> {
    fn sub_assign(&mut self, rhs: Self) {
        *self += -rhs;
    }
}
impl<const N: usize> MulAssign for Zn<N> {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}
impl<const N: usize> Sum for Zn<N> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |prev, next| prev + next)
    }
}
impl<const N: usize> Product for Zn<N> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::new(1), |prev, next| prev * next)
    }
}

impl<const N: usize> Indexed for Zn<N> {
    const COUNT: usize = N;
    fn index(&self) -> usize {
        self.value()
    }
    fn from_index(index: usize) -> Self {
        Self::new(index)
    }
}

impl<const N: usize> Zn<N> {
    const CHECK: () = assert!(N >= 1 && N <= 256, "N must be in 1..=256");
    /// The additive identity.
    pub const ZERO: Self = Self(0);

    /// Reduces `value` mod `N`.
    #[must_use = "the new value is returned"]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "value % N < N <= 256, so the representative fits a byte"
    )]
    pub const fn new(value: usize) -> Self {
        () = Self::CHECK;
        Self((value % N) as u8)
    }

    /// The representative in `0..N`.
    #[must_use]
    pub const fn value(self) -> usize {
        self.0 as usize
    }

    /// Reduces every element of `values` mod `N`.
    #[must_use = "the array is returned"]
    pub const fn array<const L: usize>(values: [usize; L]) -> [Self; L] {
        let mut final_array = [Self(0); L];
        let mut i = 0;
        while i < L {
            final_array[i] = Self::new(values[i]);
            i += 1;
        }
        final_array
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zn_public_arithmetic() {
        assert_eq!(Zn::<5>::new(7), Zn::new(2));
        assert_eq!(Zn::<5>::new(3) + Zn::new(4), Zn::new(2));
        assert_eq!(-Zn::<5>::new(2), Zn::new(3));
        assert_eq!(-Zn::<5>::new(0), Zn::new(0));
    }

    #[test]
    fn value_is_the_representative_in_range() {
        assert_eq!(Zn::<3>::ZERO.value(), 0);
        assert_eq!(Zn::<3>::new(7).value(), 1);
        for x in 0..3 {
            assert_eq!(Zn::<3>::new(x).value(), x);
        }
        assert_eq!((Zn::<3>::new(2) + Zn::new(2)).value(), 1);
    }

    #[test]
    fn zn_is_one_byte_and_the_largest_modulus_still_adds_and_negates() {
        assert_eq!(std::mem::size_of::<Zn<3>>(), 1);
        let top = Zn::<256>::new(255);
        assert_eq!((top + top).value(), 254);
        assert_eq!((-top).value(), 1);
        assert_eq!((-Zn::<256>::ZERO).value(), 0);
    }

    #[test]
    fn zn_reduces_wraps_and_negates() {
        assert_eq!(Zn::<3>::new(7), Zn::new(1));
        assert_eq!(Zn::<3>::new(2) + Zn::new(2), Zn::new(1));
        for x in 0..3 {
            let v = Zn::<3>::new(x);
            assert_eq!(v + (-v), Zn::new(0));
        }
    }

    #[test]
    fn sub_and_mul_agree_with_integer_arithmetic_mod_n() {
        for a in 0..5 {
            for b in 0..5 {
                let (x, y) = (Zn::<5>::new(a), Zn::<5>::new(b));
                assert_eq!(x - y, Zn::new(a + 5 - b), "{a} - {b}");
                assert_eq!(x * y, Zn::new(a * b), "{a} * {b}");
            }
        }
    }

    #[test]
    fn sub_is_adding_the_negation() {
        for a in 0..3 {
            for b in 0..3 {
                let (x, y) = (Zn::<3>::new(a), Zn::<3>::new(b));
                assert_eq!(x - y, x + (-y), "{a} - {b}");
            }
        }
    }

    #[test]
    fn mul_assign_matches_mul() {
        for a in 0..3 {
            for b in 0..3 {
                let (x, y) = (Zn::<3>::new(a), Zn::<3>::new(b));
                let mut z = x;
                z *= y;
                assert_eq!(z, x * y, "{a} * {b}");
            }
        }
    }

    #[test]
    fn product_multiplies_and_the_empty_product_is_one() {
        assert_eq!(std::iter::empty::<Zn<3>>().product::<Zn<3>>(), Zn::new(1));
        let factors = [2, 2, 2].map(Zn::<5>::new);
        assert_eq!(factors.into_iter().product::<Zn<5>>(), Zn::new(8));
    }

    #[test]
    fn the_largest_modulus_still_subtracts_and_multiplies() {
        let top = Zn::<256>::new(255);
        assert_eq!((Zn::<256>::ZERO - top).value(), 1);
        assert_eq!((top - Zn::ZERO).value(), 255);
        assert_eq!((top * top).value(), 1);
    }
}
