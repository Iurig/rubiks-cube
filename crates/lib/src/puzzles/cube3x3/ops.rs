use std::ops::{Mul, MulAssign};

use crate::{Algorithm, Cube3x3, Inv, MethodSolution, Move3x3, Pow};

impl Mul for Cube3x3 {
    type Output = Self;
    /// Applies the state (permutations and orientations) that is the second argument to the first
    /// argument, which is a cube.
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            centers: self.centers.then(&rhs.centers),
            corners: self.corners.then(&rhs.corners),
            edges: self.edges.then(&rhs.edges),
        }
    }
}

impl Mul<&MethodSolution<Self>> for Cube3x3 {
    type Output = Self;
    fn mul(self, rhs: &MethodSolution<Self>) -> Self::Output {
        rhs.moves().fold(self, |p, m| p * m)
    }
}

impl Mul<Algorithm<Self>> for Cube3x3 {
    type Output = Self;
    fn mul(self, rhs: Algorithm<Self>) -> Self::Output {
        self * &rhs
    }
}

macro_rules! impl_mul_assign {
    ($type:ty) => {
        impl MulAssign<$type> for Cube3x3 {
            fn mul_assign(&mut self, rhs: $type) {
                *self = *self * rhs;
            }
        }
    };
}

impl_mul_assign!(Cube3x3);
impl_mul_assign!(Move3x3);
impl_mul_assign!(Algorithm<Cube3x3>);

impl Pow for Cube3x3 {
    fn identity() -> Self {
        Self::default()
    }
}

impl Inv for Cube3x3 {
    fn inv(self) -> Self {
        Self {
            centers: self.centers.inv(),
            corners: self.corners.inv(),
            edges: self.edges.inv(),
        }
    }
}
