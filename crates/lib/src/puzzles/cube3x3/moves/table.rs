//! The table of every implemented move, built once on first use.
//!
//! The nine face and slice moves are written out by hand as a `const`; the
//! rotations and wide moves are derived from them, and every move's inverse and
//! double are then generated, all inside the [`LazyLock`] behind [`cube_state`].

use std::sync::LazyLock;

use crate::{Inv, Piece};

#[allow(
    clippy::wildcard_imports,
    clippy::enum_glob_use,
    reason = "`allow`, not `expect`: the lint is skipped when the library is compiled with `cfg(test)`"
)]
use {
    super::{MovablePart, MoveModifier, MoveModifier::*},
    crate::{
        puzzles::cube3x3::{Cube3x3, pieces::*},
        zn::Zn,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MoveInformation {
    cube_state: Cube3x3,
    part: MovablePart,
    modifier: MoveModifier,
}
/// Position of this part in the clockwise move list.
const fn table_index_clockwise(part: MovablePart) -> usize {
    match part {
        MovablePart::Face(m) => m as usize,
        MovablePart::Slice(m) => Face::ALL.len() + m as usize,
        MovablePart::Rotation(m) => Face::ALL.len() + Slice::ALL.len() + m as usize,
        MovablePart::Wide(x) => {
            table_index_clockwise(MovablePart::Face(x))
                + Face::ALL.len()
                + Slice::ALL.len()
                + Rotation::ALL.len()
        }
    }
}

const fn table_index(part: MovablePart, modifier: MoveModifier) -> usize {
    3 * table_index_clockwise(part)
        + match modifier {
            Clockwise => 0,
            Prime => 1,
            DoublePrime | Double => 2,
        }
}
#[expect(
    clippy::indexing_slicing,
    reason = "building `ALL_MOVES` asserts every entry sits at its own `table_index`"
)]
pub fn cube_state(part: MovablePart, modifier: MoveModifier) -> Cube3x3 {
    ALL_MOVES[table_index(part, modifier)].cube_state
}

impl Inv for MoveInformation {
    fn inv(self) -> Self {
        Self {
            cube_state: self.cube_state.inv(),
            part: self.part,
            modifier: self.modifier.inverse(),
        }
    }
}

impl MoveInformation {
    /// Placeholder that fills the table arrays before every entry is placed.
    const IDENTITY: Self = Self {
        cube_state: Cube3x3::IDENTITY,
        part: MovablePart::Face(Face::R),
        modifier: Clockwise,
    };

    #[expect(
        clippy::unreachable,
        reason = "private, and only called on clockwise entries while building the table"
    )]
    fn double(&self) -> Self {
        Self {
            cube_state: self.cube_state * self.cube_state,
            part: self.part,
            modifier: {
                match self.modifier {
                    Clockwise | Prime => Double,
                    _ => unreachable!("double is only built from clockwise entries"),
                }
            },
        }
    }
}

impl Slice {
    const fn follows(self) -> Face {
        match self {
            Self::M => Face::L,
            Self::E => Face::D,
            Self::S => Face::F,
        }
    }
}
impl Rotation {
    const fn follows(self) -> Face {
        match self {
            Self::X => Face::R,
            Self::Y => Face::U,
            Self::Z => Face::F,
        }
    }
}
impl Face {
    const fn opposite(self) -> Self {
        match self {
            Self::R => Self::L,
            Self::L => Self::R,
            Self::U => Self::D,
            Self::D => Self::U,
            Self::F => Self::B,
            Self::B => Self::F,
        }
    }
}

const FACE_AND_SLICES_CLOCKWISE_MOVE_COUNT: usize = Face::ALL.len() + Slice::ALL.len();
const ALL_FACE_AND_SLICES_CLOCKWISE_MOVES: [MoveInformation; FACE_AND_SLICES_CLOCKWISE_MOVE_COUNT] = [
    MoveInformation {
        cube_state: Cube3x3 {
            centers: CenterConfiguration::IDENTITY,
            corners: {
                let mut corners = CornerConfiguration::cycle([[
                    Corner::Ufr,
                    Corner::Ubr,
                    Corner::Dbr,
                    Corner::Dfr,
                ]]);
                corners.orientation = Zn::array([0, 1, 2, 0, 0, 1, 2, 0]);
                corners
            },
            edges: EdgeConfiguration::cycle([[Edge::Fr, Edge::Ur, Edge::Br, Edge::Dr]]),
        },
        part: MovablePart::Face(Face::R),
        modifier: Clockwise,
    },
    MoveInformation {
        cube_state: Cube3x3 {
            centers: CenterConfiguration::IDENTITY,
            corners: {
                let mut corners = CornerConfiguration::cycle([[
                    Corner::Ubl,
                    Corner::Ufl,
                    Corner::Dfl,
                    Corner::Dbl,
                ]]);
                corners.orientation = Zn::array([2, 0, 0, 1, 2, 0, 0, 1]);
                corners
            },
            edges: EdgeConfiguration::cycle([[Edge::Fl, Edge::Dl, Edge::Bl, Edge::Ul]]),
        },
        part: MovablePart::Face(Face::L),
        modifier: Clockwise,
    },
    MoveInformation {
        cube_state: Cube3x3 {
            centers: CenterConfiguration::IDENTITY,
            corners: {
                let mut corners = CornerConfiguration::cycle([[
                    Corner::Ufr,
                    Corner::Ufl,
                    Corner::Ubl,
                    Corner::Ubr,
                ]]);
                corners.orientation = Zn::array([0, 0, 0, 0, 0, 0, 0, 0]);
                corners
            },
            edges: EdgeConfiguration::cycle([[Edge::Uf, Edge::Ul, Edge::Ub, Edge::Ur]]),
        },
        part: MovablePart::Face(Face::U),
        modifier: Clockwise,
    },
    MoveInformation {
        cube_state: Cube3x3 {
            centers: CenterConfiguration::IDENTITY,
            corners: {
                let mut corners = CornerConfiguration::cycle([[
                    Corner::Dfr,
                    Corner::Dbr,
                    Corner::Dbl,
                    Corner::Dfl,
                ]]);
                corners.orientation = Zn::array([0, 0, 0, 0, 0, 0, 0, 0]);
                corners
            },
            edges: EdgeConfiguration::cycle([[Edge::Df, Edge::Dr, Edge::Db, Edge::Dl]]),
        },
        part: MovablePart::Face(Face::D),
        modifier: Clockwise,
    },
    MoveInformation {
        cube_state: Cube3x3 {
            centers: CenterConfiguration::IDENTITY,
            corners: {
                let mut corners = CornerConfiguration::cycle([[
                    Corner::Dfr,
                    Corner::Dfl,
                    Corner::Ufl,
                    Corner::Ufr,
                ]]);
                corners.orientation = Zn::array([0, 0, 1, 2, 1, 2, 0, 0]);
                corners
            },
            edges: EdgeConfiguration {
                permutation: EdgeConfiguration::cycle([[Edge::Df, Edge::Fl, Edge::Uf, Edge::Fr]])
                    .permutation,
                orientation: Zn::array([0, 0, 1, 0, 1, 1, 0, 0, 1, 0, 0, 0]),
            },
        },
        part: MovablePart::Face(Face::F),
        modifier: Clockwise,
    },
    MoveInformation {
        cube_state: Cube3x3 {
            centers: CenterConfiguration::IDENTITY,
            corners: {
                let mut corners = CornerConfiguration::cycle([[
                    Corner::Ubr,
                    Corner::Ubl,
                    Corner::Dbl,
                    Corner::Dbr,
                ]]);
                corners.orientation = Zn::array([1, 2, 0, 0, 0, 0, 1, 2]);
                corners
            },
            edges: EdgeConfiguration {
                permutation: EdgeConfiguration::cycle([[Edge::Br, Edge::Ub, Edge::Bl, Edge::Db]])
                    .permutation,
                orientation: Zn::array([1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 0]),
            },
        },
        part: MovablePart::Face(Face::B),
        modifier: Clockwise,
    },
    MoveInformation {
        cube_state: Cube3x3 {
            centers: CenterConfiguration::cycle([[Center::F, Center::R, Center::B, Center::L]]),
            corners: CornerConfiguration::IDENTITY,
            edges: EdgeConfiguration {
                permutation: EdgeConfiguration::cycle([[Edge::Fr, Edge::Br, Edge::Bl, Edge::Fl]])
                    .permutation,
                orientation: Zn::array([0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0]),
            },
        },
        part: MovablePart::Slice(Slice::E),
        modifier: Clockwise,
    },
    MoveInformation {
        cube_state: Cube3x3 {
            centers: CenterConfiguration::cycle([[Center::F, Center::D, Center::B, Center::U]]),
            corners: CornerConfiguration::IDENTITY,
            edges: EdgeConfiguration {
                permutation: EdgeConfiguration::cycle([[Edge::Uf, Edge::Df, Edge::Db, Edge::Ub]])
                    .permutation,
                orientation: Zn::array([1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0]),
            },
        },
        part: MovablePart::Slice(Slice::M),
        modifier: Clockwise,
    },
    MoveInformation {
        cube_state: Cube3x3 {
            centers: CenterConfiguration::cycle([[Center::U, Center::R, Center::D, Center::L]]),
            corners: CornerConfiguration::IDENTITY,
            edges: EdgeConfiguration {
                permutation: EdgeConfiguration::cycle([[Edge::Ul, Edge::Ur, Edge::Dr, Edge::Dl]])
                    .permutation,
                orientation: Zn::array([0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1]),
            },
        },
        part: MovablePart::Slice(Slice::S),
        modifier: Clockwise,
    },
];

#[expect(
    clippy::panic,
    reason = "every face is followed or opposed by a slice, so the loop always returns"
)]
#[expect(
    clippy::indexing_slicing,
    reason = "`table_index_clockwise` is below `CLOCKWISE_MOVE_COUNT` for every part"
)]
fn slice_along(face: Face, placed: &[MoveInformation; CLOCKWISE_MOVE_COUNT]) -> Cube3x3 {
    for s in Slice::ALL {
        if s.follows() == face {
            return placed[table_index_clockwise(MovablePart::Slice(s))].cube_state;
        } else if s.follows().opposite() == face {
            return placed[table_index_clockwise(MovablePart::Slice(s))]
                .cube_state
                .inv();
        }
    }
    panic!("all faces must have a slice_along")
}

const CLOCKWISE_MOVE_COUNT: usize =
    FACE_AND_SLICES_CLOCKWISE_MOVE_COUNT + Face::ALL.len() + Rotation::ALL.len();
#[expect(
    clippy::indexing_slicing,
    reason = "`table_index_clockwise` is below `CLOCKWISE_MOVE_COUNT` for every part, and the counters stop at each array's length"
)]
fn all_clockwise_moves() -> [MoveInformation; CLOCKWISE_MOVE_COUNT] {
    let mut all_clockwise_moves = [MoveInformation::IDENTITY; CLOCKWISE_MOVE_COUNT];

    for i in 0..FACE_AND_SLICES_CLOCKWISE_MOVE_COUNT {
        all_clockwise_moves[table_index_clockwise(ALL_FACE_AND_SLICES_CLOCKWISE_MOVES[i].part)] =
            ALL_FACE_AND_SLICES_CLOCKWISE_MOVES[i];
    }

    for i in 0..Rotation::ALL.len() {
        all_clockwise_moves[table_index_clockwise(MovablePart::Rotation(Rotation::ALL[i]))] =
            MoveInformation {
                cube_state: all_clockwise_moves
                    [table_index_clockwise(MovablePart::Face(Rotation::ALL[i].follows()))]
                .cube_state
                    * (all_clockwise_moves[table_index_clockwise(MovablePart::Face(
                        Rotation::ALL[i].follows().opposite(),
                    ))]
                    .cube_state
                    .inv())
                    * (slice_along(Rotation::ALL[i].follows(), &all_clockwise_moves)),
                part: MovablePart::Rotation(Rotation::ALL[i]),
                modifier: Clockwise,
            };
    }
    for i in 0..Face::ALL.len() {
        let face = Face::ALL[i];
        all_clockwise_moves[table_index_clockwise(MovablePart::Wide(face))] = MoveInformation {
            cube_state: all_clockwise_moves[table_index_clockwise(MovablePart::Face(face))]
                .cube_state
                * (slice_along(face, &all_clockwise_moves)),
            part: MovablePart::Wide(face),
            modifier: Clockwise,
        };
    }

    all_clockwise_moves
}

static ALL_MOVES: LazyLock<[MoveInformation; 3 * CLOCKWISE_MOVE_COUNT]> = LazyLock::new(all_moves);

// A named function because a closure cannot carry `#[expect]`.
#[expect(
    clippy::indexing_slicing,
    reason = "`i` stops at `CLOCKWISE_MOVE_COUNT`, a third of the table's length"
)]
fn all_moves() -> [MoveInformation; 3 * CLOCKWISE_MOVE_COUNT] {
    let clockwise_moves = all_clockwise_moves();
    let mut all_moves = [MoveInformation::IDENTITY; 3 * CLOCKWISE_MOVE_COUNT];

    for i in 0..CLOCKWISE_MOVE_COUNT {
        all_moves[3 * i] = clockwise_moves[i];
        all_moves[3 * i + 1] = clockwise_moves[i].inv();
        all_moves[3 * i + 2] = clockwise_moves[i].double();
    }
    for (i, m) in all_moves.iter().enumerate() {
        assert_eq!(i, table_index(m.part, m.modifier));
    }
    all_moves
}

#[cfg(test)]
mod tests {
    //! Tests of the table's own construction: derivation identities and
    //! modifier consistency. Handedness pins, the facts from the physical cube
    //! that catch a mirrored base move, live in `tests/api/moves.rs` and go
    //! through the public queries.
    use super::*;

    fn clockwise(part: MovablePart) -> Cube3x3 {
        cube_state(part, Clockwise)
    }

    // Derivation identities: these pin the derivation, not the base moves.
    // A mirrored slice mirrors its rotation with it and still passes here.

    fn seq(parts: &[(MovablePart, MoveModifier)]) -> Cube3x3 {
        parts.iter().fold(Cube3x3::default(), |cube, &(p, m)| {
            cube * (cube_state(p, m))
        })
    }

    #[test]
    fn rotations_are_face_then_slice_then_opposite_face_undone() {
        assert_eq!(
            clockwise(MovablePart::Rotation(Rotation::X)),
            seq(&[
                (MovablePart::Face(Face::R), Clockwise),
                (MovablePart::Slice(Slice::M), Prime),
                (MovablePart::Face(Face::L), Prime),
            ])
        );
        assert_eq!(
            clockwise(MovablePart::Rotation(Rotation::Y)),
            seq(&[
                (MovablePart::Face(Face::U), Clockwise),
                (MovablePart::Slice(Slice::E), Prime),
                (MovablePart::Face(Face::D), Prime),
            ])
        );
        assert_eq!(
            clockwise(MovablePart::Rotation(Rotation::Z)),
            seq(&[
                (MovablePart::Face(Face::F), Clockwise),
                (MovablePart::Slice(Slice::S), Clockwise),
                (MovablePart::Face(Face::B), Prime),
            ])
        );
    }

    #[test]
    fn wide_moves_equal_opposite_face_plus_rotation() {
        // Rw = L x, Uw = D y, Fw = B z: reaches each wide move by a different
        // route than the table's own derivation (face then parallel slice).
        for (wide, opposite, rotation) in [
            (Face::R, Face::L, Rotation::X),
            (Face::U, Face::D, Rotation::Y),
            (Face::F, Face::B, Rotation::Z),
        ] {
            assert_eq!(
                clockwise(MovablePart::Wide(wide)),
                seq(&[
                    (MovablePart::Face(opposite), Clockwise),
                    (MovablePart::Rotation(rotation), Clockwise)
                ]),
                "{wide:?}w should equal {opposite:?} {rotation:?}"
            );
        }
    }

    // Modifiers: every clockwise entry generates its own inverse and double.

    #[test]
    fn every_part_has_consistent_modifiers() {
        for entry in all_clockwise_moves() {
            let part = entry.part;
            let cw = cube_state(part, Clockwise);
            let ccw = cube_state(part, Prime);
            assert_eq!(cw * ccw, Cube3x3::default(), "{part:?}' must undo {part:?}");
            assert_eq!(
                cube_state(part, Double),
                cw * cw,
                "{part:?}2 must be {part:?} twice"
            );
            assert_eq!(
                cube_state(part, Double),
                cube_state(part, DoublePrime),
                "{part:?}2 and {part:?}2' must share a cube state"
            );
        }
    }
}
