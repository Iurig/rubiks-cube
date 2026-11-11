mod table;

use std::str::FromStr;

use crate::ParseMoveError;
#[allow(
    clippy::wildcard_imports,
    reason = "`allow`, not `expect`: the lint is skipped when the library is compiled with `cfg(test)`"
)]
use crate::{
    Indexed, Piece, ops,
    puzzles::cube3x3::{Cube3x3, pieces::*},
};

use table::cube_state;

/// A part of a [`Cube3x3`] that can be moved (e.g. F, M, y).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MovablePart {
    /// A 3x3 face.
    Face(Face),
    /// A 3x3 slice.
    Slice(Slice),
    /// A 3x3 rotation axis.
    Rotation(Rotation),
    /// A 3x3 wide move type.
    Wide(Face),
}

/// Types of qualifiers that moves can have (e.g. `2` or `'`).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MoveModifier {
    /// The qualifier of moves without written qualifiers, usually associated with clockwise
    /// movement.
    Clockwise,
    /// The qualifier of moves qualified by `'`, usually associated with counterclockwise movement.
    Prime,
    /// A double move, when executed without a specified direction, or in the direction defined by
    /// [`Clockwise`](MoveModifier::Clockwise).
    Double,
    /// A double move executed in the [`Prime`](MoveModifier::Prime) direction.
    DoublePrime,
}

/// A move on [`Cube3x3`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Move3x3 {
    part: MovablePart,
    modifier: MoveModifier,
}

/// The axis moves and parts can have one for each cartesian axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Axis3x3(Slice);

impl Move3x3 {
    /// The Constructor for [`Move3x3`].
    #[must_use]
    pub const fn new(part: MovablePart, modifier: MoveModifier) -> Self {
        Self { part, modifier }
    }

    /// The part this move turns.
    #[must_use]
    pub const fn part(self) -> MovablePart {
        self.part
    }

    /// How far and which way the part turns.
    #[must_use]
    pub const fn modifier(self) -> MoveModifier {
        self.modifier
    }

    /// The axis of a move, so parallel moves can be differentiated.
    #[must_use]
    pub fn axis(self) -> Axis3x3 {
        self.part().axis()
    }
}

impl MovablePart {
    const ALL: [Self; 2 * Face::ALL.len() + Slice::ALL.len() + Rotation::ALL.len()] = {
        let mut all =
            [Self::Face(Face::R); 2 * Face::ALL.len() + Slice::ALL.len() + Rotation::ALL.len()];
        let mut next = 0;

        let mut i = 0;
        while i < Face::ALL.len() {
            all[next] = Self::Face(Face::ALL[i]);
            next += 1;
            i += 1;
        }
        let mut i = 0;
        while i < Slice::ALL.len() {
            all[next] = Self::Slice(Slice::ALL[i]);
            next += 1;
            i += 1;
        }
        let mut i = 0;
        while i < Rotation::ALL.len() {
            all[next] = Self::Rotation(Rotation::ALL[i]);
            next += 1;
            i += 1;
        }
        let mut i = 0;
        while i < Face::ALL.len() {
            all[next] = Self::Wide(Face::ALL[i]);
            next += 1;
            i += 1;
        }
        assert!(next == all.len());
        all
    };

    /// The axis of a part, so parallel parts can be differentiated.
    #[must_use]
    pub fn axis(self) -> Axis3x3 {
        match self {
            Self::Slice(s) => Axis3x3(s),
            Self::Wide(w) => Self::Face(w).axis(),
            Self::Rotation(Rotation::X) | Self::Face(Face::R | Face::L) => Axis3x3(Slice::M),
            Self::Rotation(Rotation::Y) | Self::Face(Face::U | Face::D) => Axis3x3(Slice::E),
            Self::Rotation(Rotation::Z) | Self::Face(Face::F | Face::B) => Axis3x3(Slice::S),
        }
    }
}

impl MoveModifier {
    const ALL: [Self; 4] = [
        Self::Clockwise,
        Self::Prime,
        Self::Double,
        Self::DoublePrime,
    ];

    /// The inverse of a move direction.
    #[must_use]
    pub const fn inverse(self) -> Self {
        match self {
            Self::Clockwise => Self::Prime,
            Self::Prime => Self::Clockwise,
            Self::Double => Self::DoublePrime,
            Self::DoublePrime => Self::Double,
        }
    }

    /// Converts a modifier to a number.
    ///
    /// Here positive numbers mean it is equivalent to that number of clockwise moves, negatives
    /// mean the same for counterclockwise.
    #[must_use]
    pub const fn as_number(self) -> i32 {
        match self {
            Self::Clockwise => 1,
            Self::Double => 2,
            Self::DoublePrime => -2,
            Self::Prime => -1,
        }
    }

    /// Converts a number to a modifier.
    ///
    /// Here, the simplest form that has an equivalent effect on the cube is returned, so double
    /// counterclockwise moves are never returned over double moves. Follows the same convention as
    /// [`as_number`](MoveModifier::as_number).
    #[must_use]
    #[expect(
        clippy::unreachable,
        reason = "type sytem doesn't support range bounds, manually verified"
    )]
    pub fn from_number(n: i32) -> Option<Self> {
        match n.rem_euclid(4) {
            0 => None,
            1 => Some(Self::Clockwise),
            2 => Some(Self::Double),
            3 => Some(Self::Prime),
            _ => unreachable!(),
        }
    }
}

impl Indexed for MovablePart {
    const COUNT: usize = Self::ALL.len();
    fn index(&self) -> usize {
        match self {
            Self::Face(f) => *f as usize,
            Self::Slice(s) => Face::ALL.len() + *s as usize,
            Self::Rotation(r) => Face::ALL.len() + Slice::ALL.len() + *r as usize,
            Self::Wide(f) => Face::ALL.len() + Slice::ALL.len() + Rotation::ALL.len() + *f as usize,
        }
    }
    fn from_index(index: usize) -> Self {
        *Self::ALL.get(index).expect("index in 0..COUNT")
    }
}

impl Indexed for MoveModifier {
    const COUNT: usize = Self::ALL.len();
    fn index(&self) -> usize {
        *self as usize
    }
    fn from_index(index: usize) -> Self {
        *Self::ALL.get(index).expect("index in 0..COUNT")
    }
}

impl Indexed for Move3x3 {
    const COUNT: usize = <(MovablePart, MoveModifier)>::COUNT;
    fn index(&self) -> usize {
        (self.part, self.modifier).index()
    }
    fn from_index(index: usize) -> Self {
        let (part, modifier) = <(MovablePart, MoveModifier)>::from_index(index);
        Self { part, modifier }
    }
}

impl std::fmt::Display for MovablePart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Face(Face::R) => write!(f, "R"),
            Self::Face(Face::F) => write!(f, "F"),
            Self::Face(Face::U) => write!(f, "U"),
            Self::Face(Face::L) => write!(f, "L"),
            Self::Face(Face::D) => write!(f, "D"),
            Self::Face(Face::B) => write!(f, "B"),
            Self::Rotation(Rotation::X) => write!(f, "x"),
            Self::Rotation(Rotation::Y) => write!(f, "y"),
            Self::Rotation(Rotation::Z) => write!(f, "z"),
            Self::Wide(x) => write!(f, "{}w", Self::Face(*x)),
            Self::Slice(Slice::E) => write!(f, "E"),
            Self::Slice(Slice::M) => write!(f, "M"),
            Self::Slice(Slice::S) => write!(f, "S"),
        }
    }
}

impl std::fmt::Display for MoveModifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Clockwise => write!(f, ""),
            Self::Prime => write!(f, "'"),
            Self::DoublePrime => write!(f, "2'"),
            Self::Double => write!(f, "2"),
        }
    }
}

impl std::fmt::Display for Move3x3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.part, self.modifier)
    }
}

impl ops::Inv for Move3x3 {
    fn inv(self) -> Self {
        Self {
            part: self.part,
            modifier: self.modifier.inverse(),
        }
    }
}

impl FromStr for Move3x3 {
    type Err = ParseMoveError;
    /// One move in notation: a part, then a modifier.
    ///
    /// The part is an uppercase face letter (`R`), a face letter followed by
    /// `w` for the wide move (`Rw`), a lowercase face letter meaning the same
    /// wide move (`r`), a slice (`M E S`), or a rotation (`x y z`). Whatever
    /// follows the part must be a modifier: nothing, `'`, `2`, or `2'`.
    fn from_str(token: &str) -> Result<Self, Self::Err> {
        let mut chars = token.chars();
        let first = chars.next().ok_or(ParseMoveError::EmptyString)?;
        let after_first = chars.as_str();
        let face = |letter: char| match letter {
            'R' => Some(Face::R),
            'L' => Some(Face::L),
            'U' => Some(Face::U),
            'D' => Some(Face::D),
            'F' => Some(Face::F),
            'B' => Some(Face::B),
            _ => None,
        };
        let (part, modifier_text) = match first {
            'x' => (MovablePart::Rotation(Rotation::X), after_first),
            'y' => (MovablePart::Rotation(Rotation::Y), after_first),
            'z' => (MovablePart::Rotation(Rotation::Z), after_first),
            'M' => (MovablePart::Slice(Slice::M), after_first),
            'E' => (MovablePart::Slice(Slice::E), after_first),
            'S' => (MovablePart::Slice(Slice::S), after_first),
            _ => match (face(first), face(first.to_ascii_uppercase())) {
                (Some(f), _) => after_first
                    .strip_prefix('w')
                    .map_or((MovablePart::Face(f), after_first), |after_w| {
                        (MovablePart::Wide(f), after_w)
                    }),
                (None, Some(f)) => (MovablePart::Wide(f), after_first),
                (None, None) => {
                    return Err(ParseMoveError::BadPart {
                        invalid_move: token.to_string(),
                        part: first.to_string(),
                    });
                }
            },
        };
        let modifier = match modifier_text {
            "" => MoveModifier::Clockwise,
            "'" => MoveModifier::Prime,
            "2" => MoveModifier::Double,
            "2'" | "'2" => MoveModifier::DoublePrime,
            other => {
                return Err(ParseMoveError::BadModifier {
                    invalid_move: token.to_string(),
                    modifier: other.to_string(),
                });
            }
        };
        Ok(Self::new(part, modifier))
    }
}

impl From<Move3x3> for Cube3x3 {
    fn from(m: Move3x3) -> Self {
        cube_state(m.part, m.modifier)
    }
}

#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    reason = "tests should panic if failed, and return result for `?` convenience"
)]
mod tests {
    //! `Move3x3::ALL` lists every move independently of `Indexed`, so tests can check one
    //! against the other.
    use super::*;
    use crate::{Algorithm, ParseSequenceError, indexed::assert_round_trips};

    impl Move3x3 {
        const ALL: [Self; MovablePart::ALL.len() * MoveModifier::ALL.len()] = {
            let mut all: [Self; MovablePart::ALL.len() * MoveModifier::ALL.len()] = [Self {
                part: MovablePart::Face(Face::R),
                modifier: MoveModifier::Clockwise,
            };
                MovablePart::ALL.len() * MoveModifier::ALL.len()];
            let mut i = 0;
            while i < MovablePart::ALL.len() {
                let mut j = 0;
                while j < MoveModifier::ALL.len() {
                    all[i * MoveModifier::ALL.len() + j] = Self {
                        part: MovablePart::ALL[i],
                        modifier: MoveModifier::ALL[j],
                    };
                    j += 1;
                }
                i += 1;
            }
            all
        };
    }

    #[test]
    fn every_move_has_its_own_index() {
        assert_round_trips::<MovablePart>();
        assert_round_trips::<MoveModifier>();
        assert_round_trips::<Move3x3>();
        assert_eq!(Move3x3::all().collect::<Vec<_>>(), Move3x3::ALL);
    }

    #[test]
    fn move_display_round_trip() {
        for m in Move3x3::ALL {
            assert_eq!(m, Move3x3::from_str(m.to_string().as_str()).unwrap());
        }
    }

    #[test]
    fn comments_run_to_end_of_line() -> anyhow::Result<()> {
        assert_eq!(
            "R U //this is a comment".parse::<Algorithm<Cube3x3>>()?,
            "R U".parse()?
        );
        assert_eq!(
            "y2 F' M F' R U' R U' Fw z' // FB
                 U R U r M' U' R U2' R' // SS
                 U R' U' R U' R' U' r // SP (CMLL skip)
                 U M' U' M U' U' M' U M // EOLR
                 U' U' M2' U' M U' U' M' U' U' M2' // EP"
                .parse::<Algorithm<Cube3x3>>()?,
            "y2 F' M F' R U' R U' Fw z'
                 U R U r M' U' R U2' R'
                 U R' U' R U' R' U' r
                 U M' U' M U' U' M' U M
                 U' U' M2' U' M U' U' M' U' U' M2'"
                .parse()?
        );
        Ok(())
    }

    #[test]
    fn empty_and_comment_only_sequences_have_no_moves() -> anyhow::Result<()> {
        assert!("".parse::<Algorithm<Cube3x3>>()?.is_empty());
        assert!(
            "// just a comment"
                .parse::<Algorithm<Cube3x3>>()?
                .is_empty()
        );
        assert!(
            "  \n\t// one\n// two\n"
                .parse::<Algorithm<Cube3x3>>()?
                .is_empty()
        );
        Ok(())
    }

    #[test]
    fn lowercase_face_is_the_wide_move() {
        for face in Face::ALL {
            let wide = MovablePart::Wide(face).to_string();
            let lower = MovablePart::Face(face).to_string().to_lowercase();
            for modifier in ["", "'", "2", "2'"] {
                assert_eq!(
                    Move3x3::from_str(format!("{lower}{modifier}").as_str()),
                    Move3x3::from_str(format!("{wide}{modifier}").as_str()),
                );
            }
        }
    }

    #[test]
    fn a_bad_move_is_the_error_variant_that_carries_it() {
        for (bad, expected_err) in [
            (
                "rw",
                ParseMoveError::BadModifier {
                    invalid_move: "rw".to_string(),
                    modifier: "w".to_string(),
                },
            ),
            (
                "Rww",
                ParseMoveError::BadModifier {
                    invalid_move: "Rww".to_string(),
                    modifier: "w".to_string(),
                },
            ),
            (
                "Q",
                ParseMoveError::BadPart {
                    invalid_move: "Q".to_string(),
                    part: "Q".to_string(),
                },
            ),
            (
                "R3",
                ParseMoveError::BadModifier {
                    invalid_move: "R3".to_string(),
                    modifier: "3".to_string(),
                },
            ),
            (
                "w",
                ParseMoveError::BadPart {
                    invalid_move: "w".to_string(),
                    part: "w".to_string(),
                },
            ),
            (
                "xw",
                ParseMoveError::BadModifier {
                    invalid_move: "xw".to_string(),
                    modifier: "w".to_string(),
                },
            ),
            (
                "Mw",
                ParseMoveError::BadModifier {
                    invalid_move: "Mw".to_string(),
                    modifier: "w".to_string(),
                },
            ),
            (
                "R''",
                ParseMoveError::BadModifier {
                    invalid_move: "R''".to_string(),
                    modifier: "''".to_string(),
                },
            ),
        ] {
            assert_eq!(Move3x3::from_str(bad), Err(expected_err));
        }
        assert_eq!(
            "R Q U".parse::<Algorithm<Cube3x3>>(),
            Err(ParseSequenceError {
                source: ParseMoveError::BadPart {
                    invalid_move: "Q".to_string(),
                    part: "Q".to_string()
                },
                line: 1,
                position: 2
            })
        );
    }
}
