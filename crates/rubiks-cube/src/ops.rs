/// A value with a multiplicative inverse.
pub trait Inv: Sized {
    /// Inverts a state multiplicatively.
    ///
    /// # Examples
    ///
    /// ```
    /// use rubiks_cube::Inv;
    /// #[derive(Clone, Debug, PartialEq)]
    /// enum FieldZ2 {
    ///     Zero,
    ///     One,
    /// }
    /// impl std::ops::Mul for FieldZ2 {
    ///     type Output = Self;
    ///     fn mul(self, rhs: Self) -> Self {
    ///         let product_table = [[FieldZ2::Zero, FieldZ2::One], [FieldZ2::One, FieldZ2::Zero]];
    ///         product_table[self as usize][rhs as usize].clone()
    ///     }
    /// }
    /// impl Inv for FieldZ2 {
    ///     fn inv(self) -> Self {
    ///         self.clone()
    ///     }
    /// }
    ///
    /// assert_eq!(FieldZ2::Zero, FieldZ2::Zero * FieldZ2::Zero.inv());
    /// assert_eq!(FieldZ2::Zero, FieldZ2::One * FieldZ2::One.inv());
    /// ```
    #[must_use = "this returns the inverse of a state, without modifying the original state"]
    fn inv(self) -> Self;
}

/// Repeated multiplication by fast exponentiation.
pub trait Pow: std::ops::Mul<Self, Output = Self> + Clone {
    /// The result of an empty product.
    fn identity() -> Self;

    /// Performs the power operation based on `std::ops::Mul` assuming an empty multiplication
    /// returns [`Self::identity`].
    ///
    /// # Examples
    ///
    /// ```
    /// use rubiks_cube::Pow;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct TurnCount(u32);
    ///
    /// impl std::ops::Mul for TurnCount {
    ///     type Output = Self;
    ///     fn mul(self, rhs: Self) -> Self::Output {
    ///         TurnCount((self.0 + rhs.0) % 4)
    ///     }
    /// }
    ///
    /// impl Pow for TurnCount {
    ///     fn identity() -> Self {
    ///         TurnCount(0)
    ///     }
    /// }
    ///
    /// assert_eq!(TurnCount(2).pow(3), TurnCount(2));
    /// ```
    #[must_use = "returns the power instead of applying it in place"]
    fn pow(self, exponent: u32) -> Self {
        // Values 0, 1 and 2 are needed for recursion on `_` branch
        match exponent {
            0 => Self::identity(),
            1 => self,
            2 => self.clone() * self,
            _ => self.clone().pow(exponent % 2) * self.pow(exponent / 2).pow(2),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    reason = "tests should panic if failed, and return result for `?` convenience"
)]
mod tests {
    use super::*;
    use crate::{Algorithm, Cube3x3, Puzzle};

    #[test]
    fn algorithm_inverse_reverses_and_inverts_each_move() -> anyhow::Result<()> {
        let alg: Algorithm<Cube3x3> = "R U R' F2".parse()?;
        assert_eq!(alg.inv(), "F2' R U' R'".parse()?);
        Ok(())
    }

    #[test]
    fn empty_algorithm_is_its_own_inverse() {
        assert_eq!(Algorithm::<Cube3x3>::new().inv(), Algorithm::new());
    }

    #[test]
    fn inverting_an_algorithm_twice_gives_it_back() -> anyhow::Result<()> {
        let alg: Algorithm<Cube3x3> = "r U' M2 x E' Fw2 D".parse()?;
        assert_eq!(alg.clone().inv().inv(), alg);
        Ok(())
    }

    #[test]
    fn algorithm_then_its_inverse_returns_to_the_start() -> anyhow::Result<()> {
        let alg: Algorithm<Cube3x3> = "R U R' U' R' F R2 U' R' U' R U R' F'".parse()?;
        for seed in 0..20 {
            let start = Cube3x3::scrambled_with_seed(seed);
            let end = alg
                .iter()
                .chain(&alg.clone().inv())
                .fold(start, |cube, &m| cube * m);
            assert_eq!(end, start, "seed {seed}");
        }
        Ok(())
    }

    #[test]
    fn clockwise_moves_have_order_exactly_4() -> anyhow::Result<()> {
        for m in &["R", "U", "D", "L", "F", "B", "E", "S", "M"] {
            let cube = Cube3x3::from_moves(m)?;
            for k in 1..4 {
                assert!(!cube.pow(k).is_solved(), "{m}^{k} should not be solved");
            }
            assert!(cube.pow(4).is_solved(), "{m}^4 should be solved");
        }
        Ok(())
    }

    #[test]
    fn pow_splits_an_odd_exponent_correctly() -> anyhow::Result<()> {
        let r = Cube3x3::from_moves("R")?;
        assert_eq!(r.pow(5), r);
        Ok(())
    }
}
