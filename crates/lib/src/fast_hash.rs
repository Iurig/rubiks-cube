use std::hash::{BuildHasherDefault, Hasher};

#[derive(Default)]
pub struct FxHasher {
    hash: u64,
}

impl FxHasher {
    const fn step(&mut self, word: u64) {
        self.hash = (self.hash.rotate_left(5) ^ word).wrapping_mul(0x517c_c1b7_2722_0a95);
    }
}

impl Hasher for FxHasher {
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.step(u64::from(b));
        }
    }
    fn finish(&self) -> u64 {
        self.hash
    }
    fn write_u8(&mut self, i: u8) {
        self.step(u64::from(i));
    }
    fn write_u16(&mut self, i: u16) {
        self.step(u64::from(i));
    }
    fn write_u32(&mut self, i: u32) {
        self.step(u64::from(i));
    }
    fn write_u64(&mut self, i: u64) {
        self.step(i);
    }
    #[expect(clippy::cast_possible_truncation, reason = "it should truncate")]
    fn write_u128(&mut self, i: u128) {
        self.step(i as u64);
        self.step((i >> 64) as u64);
    }
    fn write_usize(&mut self, i: usize) {
        self.step(i as u64);
    }
}

pub type FxMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<FxHasher>>;
pub type FxSet<K> = std::collections::HashSet<K, BuildHasherDefault<FxHasher>>;

/// Equal keys must hash equally. Hash values are not portable, so these compare two hashes
/// with each other and look keys up in a map, never against a constant.
#[cfg(test)]
mod tests {
    use std::hash::BuildHasher;

    use super::*;
    use crate::{Center, Corner, Cube3x3, Edge, Piece3x3, PieceSet};
    use statrs::{
        distribution::{Binomial, DiscreteCDF},
        statistics::Distribution,
    };

    const FIRST_BLOCK: [Piece3x3; 6] = [
        Piece3x3::Center(Center::L),
        Piece3x3::Corner(Corner::Dfl),
        Piece3x3::Corner(Corner::Dbl),
        Piece3x3::Edge(Edge::Fl),
        Piece3x3::Edge(Edge::Dl),
        Piece3x3::Edge(Edge::Bl),
    ];

    fn hash(mask: &PieceSet<Cube3x3>) -> u64 {
        BuildHasherDefault::<FxHasher>::default().hash_one(mask)
    }

    const RUNS: u64 = 1_000_000;
    const P_VALUE: f64 = 1e-4;

    macro_rules! test {
        ($u_n:ident, $write_u_n:ident) => {
            fastrand::seed(7);
            let mut hasher = FxHasher { hash: 0 };
            let runs = std::cmp::min(($u_n::MAX / 2) as u64, RUNS);
            {
                let mut key_set = FxSet::default();
                for _ in 0..runs {
                    hasher.$write_u_n(fastrand::$u_n(..));
                    key_set.insert(hasher.finish());
                    hasher.hash = 0;
                }
                let distr = Binomial::new(
                    1.0 - (1.0 - 1.0 / ($u_n::MAX as f64)).powi(runs as i32),
                    $u_n::MAX as u64,
                )
                .unwrap();
                let rarity = distr.cdf(key_set.iter().len() as u64);
                dbg!(
                    1.0 - (1.0 - 1.0 / ($u_n::MAX as f64)).powi(runs as i32),
                    runs,
                    key_set.iter().len() as u64,
                    rarity,
                    distr.mean()
                );
                assert!(rarity > P_VALUE);
                assert!(key_set.iter().len() > (runs / 3) as usize);
            };
        };
    }

    #[expect(
        clippy::cast_lossless,
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        reason = "tests multiple types"
    )]
    #[test]
    fn collisions_match_pre_defined_p_value_on_specific_writes() {
        test!(u8, write_u8);
        test!(u16, write_u16);
        test!(u32, write_u32);
        test!(u64, write_u64);
        test!(u128, write_u128);
        test!(usize, write_usize);
    }

    #[expect(
        clippy::cast_lossless,
        clippy::cast_possible_truncation,
        reason = "tests multiple types"
    )]
    #[test]
    fn collisions_match_pre_defined_p_value_on_generic_write() {
        fastrand::seed(7);
        let mut hasher = FxHasher { hash: 0 };
        let runs = RUNS;
        {
            let mut key_set = FxSet::default();
            for _ in 0..runs {
                hasher.write(&fastrand::u32(..).to_be_bytes());
                key_set.insert(hasher.finish());
                hasher.hash = 0;
            }
            let distr = Binomial::new(
                1.0 - (1.0 - 1.0 / (u32::MAX as f64)).powi(runs as i32),
                u32::MAX as u64,
            )
            .unwrap();
            let rarity = distr.cdf(key_set.iter().len() as u64);
            dbg!(
                1.0 - (1.0 - 1.0 / (u32::MAX as f64)).powi(runs as i32),
                key_set.iter().len() as u32,
                rarity,
                distr.mean()
            );
            assert!(rarity > P_VALUE);
        };
    }

    #[test]
    fn masks_built_from_pieces_in_any_order_find_each_other() {
        let forward = PieceSet::<Cube3x3>::from_pieces(FIRST_BLOCK);
        let mut reversed_pieces = FIRST_BLOCK;
        reversed_pieces.reverse();
        let reversed = PieceSet::<Cube3x3>::from_pieces(reversed_pieces);

        assert_eq!(forward, reversed);
        assert_eq!(hash(&forward), hash(&reversed));
        let map = FxMap::from_iter([(forward, "first block")]);
        assert_eq!(map.get(&reversed), Some(&"first block"));
    }

    /// `U` and `R M'` leave the first block alone, so two different cubes filter to one key.
    #[test]
    fn different_cubes_with_the_same_masked_pieces_find_each_other() {
        let goal = PieceSet::<Cube3x3>::from_pieces(FIRST_BLOCK);
        let after_u = Cube3x3::from_moves("U").unwrap();
        let after_r_m = Cube3x3::from_moves("R M'").unwrap();
        assert_ne!(after_u, after_r_m);

        let from_u = PieceSet::<Cube3x3>::filter_by_piece(&after_u, &goal);
        let from_r_m = PieceSet::<Cube3x3>::filter_by_piece(&after_r_m, &goal);

        assert_eq!(from_u, from_r_m);
        assert_eq!(hash(&from_u), hash(&from_r_m));
        let map = FxMap::from_iter([(from_u, "U")]);
        assert_eq!(map.get(&from_r_m), Some(&"U"));
    }
}
