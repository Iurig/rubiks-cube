#[cfg(feature = "anstyle")]
pub mod color_facelets;
pub mod facelets;
pub mod moves;
pub mod pieces;
#[cfg(feature = "ratatui")]
pub mod widget;

#[allow(
    clippy::wildcard_imports,
    reason = "`allow`, not `expect`: the lint is skipped when the library is compiled with `cfg(test)`"
)]
use self::{moves::*, pieces::*};

use crate::{
    Algorithm, Indexed, Method, ParseSequenceError, Piece, SolveError,
    methods::cube3x3::kociemba::Kociemba,
    ops::{Inv, Pow},
    puzzles::Puzzle,
    zn::Zn,
};

/// A 3x3x3 cube state; `a * b` applies `a` then `b`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Cube3x3 {
    /// `CENTER_ORIENTATION_COUNT` is 1, centers are considered without orientation
    centers: CenterConfiguration,
    /// Corner orientation is done with the convention of clockwise rotations from white/yellow
    /// sticker being in the faces U and D
    corners: CornerConfiguration,
    /// 0 is oriented, 1 is misoriented
    edges: EdgeConfiguration,
}
macro_rules! unify_pieces {
    ($($piece_type:ident: [$($p:ident),+]),+ $(,)?) => {
        [
            $($(Piece3x3::$piece_type($piece_type::$p)),+),+
        ]
    };
}

impl Puzzle for Cube3x3 {
    type Piece = Piece3x3;
    type Orientation = Orientation3x3;
    type Move = Move3x3;

    /// Whether this state is a (possibly rotated) solved cube.
    fn is_solved(&self) -> bool {
        self.rotated_until_solved_centers() == Some(Self::default())
    }

    fn slot_of(&self, piece: Self::Piece) -> Self::Piece {
        Piece3x3::all()
            .find(|&slot| self.piece_at(slot) == piece)
            .expect("All Cubes should have all pieces somewhere")
    }

    fn piece_at(&self, slot: Self::Piece) -> Self::Piece {
        match slot {
            Self::Piece::Corner(co) => Piece3x3::Corner(self.corners.piece_at(co)),
            Self::Piece::Edge(ed) => Piece3x3::Edge(self.edges.piece_at(ed)),
            Self::Piece::Center(ce) => Piece3x3::Center(self.centers.piece_at(ce)),
        }
    }

    fn orientation_at(&self, slot: Self::Piece) -> Self::Orientation {
        match slot {
            Self::Piece::Corner(co) => Self::Orientation::Twist(self.corners.orientation_at(co)),
            Self::Piece::Edge(ed) => Self::Orientation::Flip(self.edges.orientation_at(ed)),
            Self::Piece::Center(_) => Self::Orientation::Fixed,
        }
    }

    fn scrambled_with_seed(seed: u64) -> Self {
        let mut rng = fastrand::Rng::with_seed(seed);
        let mut attempt = Self {
            corners: CornerConfiguration::random_state(&mut rng),
            edges: EdgeConfiguration::random_state(&mut rng),
            ..Default::default()
        };
        attempt.corners.orientation[0] -= attempt.corners.orientation_sum();
        attempt.edges.orientation[0] -= attempt.edges.orientation_sum();
        if !(attempt.is_reachable()) {
            attempt.edges.permutation.swap(0, 1);
        }
        attempt
    }

    fn scramble_with_seed(seed: u64) -> Result<Algorithm<Self>, SolveError> {
        let cube = Self::scrambled_with_seed(seed);
        cube.get_scramble()
    }
}

impl Indexed for Piece3x3 {
    const COUNT: usize = Cube3x3::ALL_PIECES.len();
    fn index(&self) -> usize {
        match self {
            Self::Center(c) => *c as usize,
            Self::Corner(c) => Center::ALL.len() + *c as usize,
            Self::Edge(e) => Center::ALL.len() + Corner::ALL.len() + *e as usize,
        }
    }
    fn from_index(index: usize) -> Self {
        *Cube3x3::ALL_PIECES.get(index).expect("index in 0..COUNT")
    }
}

impl std::ops::Mul for Cube3x3 {
    type Output = Self;
    /// Applies the state (permutations and orientations) that is the second argument to the first
    /// argument, which is a cube.
    ///
    /// IMPORTANT: associative, but non-commutative
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            centers: self.centers.then(&rhs.centers),
            corners: self.corners.then(&rhs.corners),
            edges: self.edges.then(&rhs.edges),
        }
    }
}

impl std::ops::Mul<Move3x3> for Cube3x3 {
    type Output = Self;
    fn mul(self, m: Move3x3) -> Self::Output {
        self * Self::from(m)
    }
}

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

impl Cube3x3 {
    /// The multiplicative identity of the cube group: the solved cube
    pub const IDENTITY: Self = Self {
        centers: CenterConfiguration::IDENTITY,
        corners: CornerConfiguration::IDENTITY,
        edges: EdgeConfiguration::IDENTITY,
    };

    const ALL_PIECES: &'static [Piece3x3] = unify_pieces!(
        Center: [U, F, R, B, L, D],
        Corner: [Ubl, Ubr, Ufr, Ufl, Dfl, Dfr, Dbr, Dbl],
        Edge: [Ub, Ur, Uf, Ul, Fl, Fr, Br, Bl, Df, Dr, Db, Dl],
    )
    .as_slice();

    /// The corner permutation and twists.
    #[must_use]
    pub const fn corners(&self) -> &CornerConfiguration {
        &self.corners
    }
    /// The edge permutation and flips.
    #[must_use]
    pub const fn edges(&self) -> &EdgeConfiguration {
        &self.edges
    }
    /// The center permutation. For consistency, accompanied by a `Zn::ZERO` orientation
    #[must_use]
    pub const fn centers(&self) -> &CenterConfiguration {
        &self.centers
    }

    /// Applies a move sequence to this cube state, in order, from a `&str`
    ///
    /// # Errors
    ///
    /// Errors when a whitespace separated &str outside of comments is not parseable
    /// as a move; the error names that &str and its line and position, both counted
    /// from 1, using the type [`ParseSequenceError`]
    pub fn apply_moves(&self, moves: &str) -> Result<Self, ParseSequenceError> {
        Ok(self.apply(&moves.parse()?))
    }

    /// Applies a move sequence to the solved cube.
    ///
    /// # Errors
    ///
    /// Same as [`Self::apply_moves`].
    pub fn from_moves(m: &str) -> Result<Self, ParseSequenceError> {
        Self::default().apply_moves(m)
    }

    fn rotated_until_solved_centers(&self) -> Option<Self> {
        let mut rotated_self = *self;
        if ![
            rotated_self.centers().piece_at(Center::F),
            rotated_self.centers().piece_at(Center::U),
            rotated_self.centers().piece_at(Center::B),
            rotated_self.centers().piece_at(Center::D),
        ]
        .contains(&Face::F)
        {
            rotated_self *=
                Move3x3::new(MovablePart::Rotation(Rotation::Y), MoveModifier::Clockwise);
        }
        for _ in 0..4 {
            if rotated_self.centers().piece_at(Face::F) != Face::F {
                rotated_self *=
                    Move3x3::new(MovablePart::Rotation(Rotation::X), MoveModifier::Clockwise);
            }
        }
        for _ in 0..4 {
            if rotated_self.centers().piece_at(Face::U) != Face::U {
                rotated_self *=
                    Move3x3::new(MovablePart::Rotation(Rotation::Z), MoveModifier::Clockwise);
            }
        }

        (rotated_self.centers == CenterConfiguration::default()).then_some(rotated_self)
    }

    /// Whether some move sequence produces this state from the solved cube.
    ///
    /// Checks for four invariants: that centers are solved with respect to each other, that edge
    /// flips are even, that corner twists are divisible by 3, and that an even number of
    /// 2-swaps reaches the permutation of the pieces.
    #[must_use]
    pub fn is_reachable(&self) -> bool {
        self.twists_cancel()
            && self.flips_cancel()
            && self.parities_cancel()
            && self.centers_form_a_rotation()
    }

    /// Corner twists sum to zero mod 3: a face turn twists corners by amounts
    /// that cancel, so a lone twisted corner is unreachable.
    fn twists_cancel(&self) -> bool {
        self.corners().orientation_sum() == Zn::ZERO
    }

    /// Edge flips sum to zero mod 2: a face turn flips an even number of
    /// edges, so a lone flipped edge is unreachable.
    fn flips_cancel(&self) -> bool {
        self.edges().orientation_sum() == Zn::ZERO
    }

    /// The permutation parities of corners, edges, and centers sum to zero
    /// mod 2. A face turn is odd on corners and edges; a slice turn is odd on
    /// edges and centers. Every move flips exactly two of the three, so the
    /// sum stays zero. A two-way corner/edge check would reject a lone `M`.
    fn parities_cancel(&self) -> bool {
        self.corners().parity() + self.edges().parity() + self.centers().parity() == Zn::ZERO
    }

    /// The centers sit as one of the 24 whole-cube rotations. A 3-cycle of
    /// centers is an even permutation, so the parity sum accepts it, yet no
    /// move sequence produces it.
    fn centers_form_a_rotation(&self) -> bool {
        self.rotated_until_solved_centers().is_some()
    }

    /// The scramble that gets the cube to the state provided.
    ///
    /// # Errors
    /// Errors if the solving process to produce the scramble errors. See [`solve`](Method::solve)
    /// for more details.
    pub fn get_scramble(&self) -> Result<Algorithm<Cube3x3>, SolveError> {
        let mut cube_copy = self.clone();
        Ok(Kociemba
            .solve(&mut cube_copy)?
            .iter()
            .flat_map(|s| s.moves().iter().copied())
            .collect::<Algorithm<Self>>()
            .inv()
            .simplify_single_moves())
    }
}
#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    reason = "tests should panic if failed, and return result for `?` convenience"
)]
mod tests {

    use crate::{Piece, indexed::assert_round_trips, piece::index};

    use super::*;

    #[test]
    fn index_is_the_position_in_all_pieces() {
        assert_round_trips::<Piece3x3>();
        for (i, &piece) in Cube3x3::ALL_PIECES.iter().enumerate() {
            assert_eq!(piece.index(), i, "{piece:?}");
        }
    }

    #[test]
    fn center_3_cyle_isnt_reachable_even_if_respects_parity() {
        assert!(
            !Cube3x3 {
                centers: CenterConfiguration::cycle([[Center::F, Center::R, Center::U]]),
                ..Default::default()
            }
            .is_reachable()
        );
    }

    #[test]
    fn corner_twist_isnt_reachable() {
        assert!(
            !Cube3x3 {
                corners: CornerConfiguration {
                    orientation: Zn::array([1, 0, 0, 0, 0, 0, 0, 0,]),
                    ..Default::default()
                },
                ..Default::default()
            }
            .is_reachable()
        );
    }

    #[test]
    fn edge_flip_isnt_reachable() {
        assert!(
            !Cube3x3 {
                edges: EdgeConfiguration {
                    orientation: Zn::array([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,]),
                    ..Default::default()
                },
                ..Default::default()
            }
            .is_reachable()
        );
    }

    #[test]
    fn single_swap_isnt_reachable() {
        assert!(
            !Cube3x3 {
                corners: CornerConfiguration {
                    permutation: {
                        let mut p = Corner::ALL;
                        p.swap(index(Corner::Ufr), index(Corner::Ubr));
                        p
                    },
                    ..Default::default()
                },
                ..Default::default()
            }
            .is_reachable()
        );
    }

    #[test]
    fn mul_carries_orientation_along_with_the_piece() -> anyhow::Result<()> {
        // Pre-twist the piece at UFR, then apply R: that piece lands at UBR and
        // its twist is added to the twist R gives the UBR slot.
        let r = Cube3x3::from_moves("R")?;
        let mut twisted = Cube3x3::default();
        twisted.corners.orientation[Corner::Ufr as usize] = Zn::new(1);
        let after = twisted * r;
        let mut expected = r;
        expected.corners.orientation[Corner::Ubr as usize] += Zn::new(1);
        assert_eq!(after, expected);
        Ok(())
    }

    #[test]
    fn multiplying_by_an_algorithm_applies_its_moves_in_order() -> anyhow::Result<()> {
        let alg: Algorithm<Cube3x3> = "R U R' F2".parse()?;
        let start = Cube3x3::scrambled_with_seed(0);
        assert_eq!(start * alg, start * Cube3x3::from_moves("R U R' F2")?);
        Ok(())
    }

    #[test]
    fn u_perm_repeats_after_3_applications() {
        use Edge::{Uf, Ul, Ur};
        let u_perm = Cube3x3 {
            edges: EdgeConfiguration::cycle([[Ur, Uf, Ul]]),
            ..Default::default()
        };
        assert_eq!(u_perm.pow(3), Cube3x3::default());
    }

    #[test]
    fn default_is_reachable() {
        assert!(Cube3x3::default().is_reachable());
    }

    #[test]
    fn r_is_reachable() -> anyhow::Result<()> {
        assert!(Cube3x3::from_moves("R")?.is_reachable());
        Ok(())
    }

    #[test]
    fn m_is_reachable() -> anyhow::Result<()> {
        assert!(Cube3x3::from_moves("M")?.is_reachable());
        Ok(())
    }

    #[test]
    fn y_is_reachable() -> anyhow::Result<()> {
        assert!(Cube3x3::from_moves("y")?.is_reachable());
        Ok(())
    }

    #[test]
    fn hundred_random_states_are_reachable() {
        for seed in 0..100 {
            assert!(
                Cube3x3::scrambled_with_seed(seed).is_reachable(),
                "the state from seed {seed} is not reachable"
            );
        }
    }

    /// A random state is solved about once in 4.3 × 10¹⁹ draws.
    #[test]
    fn a_random_scramble_is_not_solved() {
        assert!(!Cube3x3::scrambled().is_solved());
    }

    #[test]
    fn default_is_solved() {
        assert!(Cube3x3::default().is_solved());
    }

    #[test]
    fn y_rotated_solved_is_solved() -> anyhow::Result<()> {
        let rotated_def = Cube3x3::from_moves("y")?;
        assert!(rotated_def.is_solved());
        Ok(())
    }

    #[test]
    fn rotated_solved_is_solved() -> anyhow::Result<()> {
        let rotated_def = Cube3x3::from_moves("y z y z x2 z2")?;
        assert!(rotated_def.is_solved());
        Ok(())
    }

    #[test]
    fn all_axes_rotated_solved_is_solved_but_not_after_a_face_turn() -> anyhow::Result<()> {
        assert!(Cube3x3::from_moves("x y2 z'")?.is_solved());
        assert!(!Cube3x3::from_moves("x y2 z' R")?.is_solved());
        Ok(())
    }
}
