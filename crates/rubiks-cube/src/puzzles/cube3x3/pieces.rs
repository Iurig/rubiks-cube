use std::fmt::Display;

use crate::{
    Indexed, Piece,
    piece::PieceConfiguration,
    puzzles::cube3x3::pieces::Orientation3x3::{Fixed, Flip, Twist},
    zn::Zn,
};

const CENTERS_COUNT: usize = 6;
const CORNERS_COUNT: usize = 8;
const EDGES_COUNT: usize = 12;
const CENTER_ORIENTATION_COUNT: usize = 1;
const CO_COUNT: usize = 3;
const EO_COUNT: usize = 2;

macro_rules! new_piece {
    ($(#[$attr:meta])* $type_name:ident, $amount:ident, [$($p:ident),+]) => {
        $(#[$attr])*
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[repr(u8)]
        pub enum $type_name {
            $(
                #[doc = concat!("The `", stringify!($p), "` piece, and the slot it is solved in.")]
                $p
            ),+
        }
        impl Display for $type_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(Self::$p => stringify!($p).fmt(f)),+
                }
            }
        }
        impl Indexed for $type_name {
            const COUNT: usize = $amount;
            fn index(&self) -> usize {
                const { assert!(std::mem::size_of::<Self>() == 1) }
                *self as usize
            }
            fn from_index(index: usize) -> Self {
                *Self::ALL.get(index).expect("index in 0..COUNT")
            }
        }
        impl Piece<$amount> for $type_name {
            const ALL: [Self; $amount] = [$($type_name::$p),+];
        }
        impl crate::piece::private::Sealed for $type_name {}
        const _: () = {
            let mut i = 0;
            while i < $amount {
                assert!($type_name::ALL[i] as usize == i);
                i += 1;
            }
        };
        impl From<$type_name> for Piece3x3 {
            fn from(piece: $type_name) -> Self {
                Self::$type_name(piece)
            }
        }
    };
}
// Center slots are in blind standard order, i.e. `[U, F, R, B, L, D]`
new_piece!(
    /// The six center pieces, named by face.
    Center,
    CENTERS_COUNT,
    [U, F, R, B, L, D]
);
// Corner slots are in blind standard order, i.e. `[UBL, UBR, UFR, UFL, DFL, DFR, DBR, DBL]`
new_piece!(
    /// The eight corner pieces, named by their three faces.
    Corner,
    CORNERS_COUNT,
    [Ubl, Ubr, Ufr, Ufl, Dfl, Dfr, Dbr, Dbl]
);
// Edge slots are clockwise per layer, i.e. `[UB, UR, UF, UL, FL, FR, BR, BL, DF, DR, DB, DL]`
new_piece!(
    /// The twelve edge pieces, named by their two faces.
    Edge,
    EDGES_COUNT,
    [Ub, Ur, Uf, Ul, Fl, Fr, Br, Bl, Df, Dr, Db, Dl]
);

pub type CenterConfiguration = PieceConfiguration<Center, CENTERS_COUNT, CENTER_ORIENTATION_COUNT>;
pub type CornerConfiguration = PieceConfiguration<Corner, CORNERS_COUNT, CO_COUNT>;
pub type EdgeConfiguration = PieceConfiguration<Edge, EDGES_COUNT, EO_COUNT>;
/// A [`Cube3x3`](crate::Cube3x3) face.
///
/// An alias for [`Center`].
pub type Face = Center;

/// Any piece of the 3×3 cube: the cube's [`Puzzle::Piece`](crate::Puzzle::Piece) type. Masks and
/// piece queries on a [`Cube3x3`](crate::Cube3x3) take this type.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Piece3x3 {
    /// A center piece.
    Center(Center),
    /// An edge piece.
    Edge(Edge),
    /// A corner piece.
    Corner(Corner),
}

impl Display for Piece3x3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Center(piece) => piece.fmt(f),
            Self::Edge(piece) => piece.fmt(f),
            Self::Corner(piece) => piece.fmt(f),
        }
    }
}

/// The orientation of one piece of the 3x3, with one variant per kind of piece.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Orientation3x3 {
    /// A center, which has no orientation.
    Fixed,
    /// An edge's flip: 0 is oriented, 1 is flipped.
    Flip(Zn<2>),
    /// A corner's twist: 0 is oriented, 1 or 2 is twisted.
    Twist(Zn<3>),
}

#[expect(
    unnameable_types,
    reason = "reachable through `MovablePart::Slice`; exported together with `MovablePart`"
)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Slice {
    M,
    S,
    E,
}
impl Slice {
    pub const ALL: [Self; 3] = [Self::M, Self::S, Self::E];
}
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[expect(
    unnameable_types,
    reason = "reachable through `MovablePart::Rotation`; exported together with `MovablePart`"
)]
pub enum Rotation {
    X,
    Y,
    Z,
}
impl Rotation {
    pub const ALL: [Self; 3] = [Self::X, Self::Y, Self::Z];
}

impl std::ops::Add for Orientation3x3 {
    type Output = Self;
    #[expect(
        clippy::unreachable,
        reason = "if this arm is ever reached, the state of puzzle is absolutely unrecoverable"
    )]
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Fixed, Fixed) => Fixed,
            (Twist(lhs), Twist(rhs)) => Twist(lhs + rhs),
            (Flip(lhs), Flip(rhs)) => Flip(lhs + rhs),
            (_, _) => unreachable!("You should never add the orientation of different piece types"),
        }
    }
}
