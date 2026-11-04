//! A model of the 3×3×3 Rubik's Cube, and a solver that solves it the way a person would, one
//! named step at a time.
//!
//! A cube state is a [`Cube3x3`]. Moves are cube states too, and `a * b` applies `a` and then
//! `b`, in the same order as notation reads:
//!
//! ```
//! use rubiks_cube::{Cube3x3, Inv, Pow, Puzzle};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let scramble = Cube3x3::from_moves("R' U' F D2 L2 F R2 U2 R2 B D2 L B2 D' B2 L' R' B D2 B U2 L U2 R' U' F")?;
//! let solution = Cube3x3::from_moves("D2 F' D2 U2 F' L2 D R2 D B2 F L2 R' F' D U'")?;
//! assert!((scramble * solution).is_solved());
//!
//! assert_eq!(Cube3x3::from_moves("R")?.inv(), Cube3x3::from_moves("R'")?);
//! assert!(Cube3x3::from_moves("R U R' U'")?.pow(6).is_solved());
//! # Ok(())
//! # }
//! ```
//!
//! Move strings use standard notation: the face turns `R L U D F B`, the slices `M E S`, the
//! rotations `x y z`, and the wide moves `Rw` or `r`, each with no modifier, `'`, `2`, or `2'`.
//! Whitespace separates moves, and `//` starts a comment that runs to the end of the line. A
//! string that is not valid notation returns a [`ParseSequenceError`] naming the line and
//! position of the bad move.
//!
//! # Solving
//!
//! A [`Method`], such as [`Roux`] or [`Kociemba`], turns its options into a [`Technique`]: an
//! ordered list of [`Step`]s. Roux's builder methods, such as [`Roux::first_block`], choose
//! between its step variants. Solving returns a [`Solution`], which prints as notation with
//! one line per step:
//!
//! ```rust
//! use rubiks_cube::{Cube3x3, Method, Puzzle, Roux};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut cube = Cube3x3::from_moves("D2 F2 R2 U L2 D R2 U' B2 L2 B L2 F' L D2 U R' B D2")?;
//! let solution = Roux::default().solve(&mut cube)?;
//! assert!(cube.is_solved());
//! print!("{solution}");
//! # Ok(())
//! # }
//! ```
//!
//! ```text
//! U' F' B2 D R2 B D2    //FB
//! U2 R2    //DR
//! R2 U' R U R' U R'    //SB Square
//! U M' U2 Rw2 R U' Rw    //SB Pair
//! U2 R U R' U' R' F R F' R U R' U R U2 R' U'    //CMLL
//! U2 M' U M' U2 M U2 M' U' M' U2 M'    //LSE
//! ```
//!
//! To build other methods, combine [`SearchStep`]s, which search for a goal given as a
//! [`Mask`], and [`Shortest`], which keeps the shortest of several alternatives. Any type that
//! implements [`Step`] can join them. [`Method::solve_steps`] runs a method one step per
//! iteration, for timing or progress reports.
//!
//! # Errors
//!
//! A solve that fails returns a [`SolveError`] naming the step it stopped at. Its
//! [`kind`](SolveError::kind) says why: the step's requirements weren't met, the step reported a
//! [`StepError`], or the step returned without meeting its goal. When the step reported one,
//! [`inner`](SolveError::inner) returns the [`StepError`].
//!
//! # Lower levels
//!
//! [`Cube3x3`] is built from three [`PieceConfiguration`]s (centers, corners, and edges), each a
//! permutation of [`Piece`]s with an orientation per piece in [`zn::Zn`]. The [`Puzzle`] trait
//! is what the solver needs from a puzzle, and [`Facelets`] shows a cube as a sticker net.
mod error;
mod fast_hash;
mod indexed;
mod methods;
mod ops;
mod ordinal;
mod piece;
mod puzzles;
pub mod zn;

pub use error::{
    BuildStepError, ParseMoveError, ParseSequenceError, SearchStepError, SolveError,
    SolveErrorKind, StepError, TechniqueError,
};
pub use indexed::Indexed;
// Methods with options are re-exported as modules too (`roux::Cmll`), so the options need no method
// prefix. `kociemba` has no options.
pub use methods::{
    Method, Segment, Solution, Technique, TechniqueBuilder,
    context::BuildContext,
    cube3x3::{kociemba::Kociemba, roux, roux::Roux, zz, zz::ZZ},
    step::{
        Step, StepBuilder,
        choose::Shortest,
        search_step::{SearchCache, SearchStep, SearchStepBuilder},
    },
};
pub use ops::{Inv, Pow};
pub use piece::{Piece, PieceConfiguration};
pub use puzzles::cube3x3::{
    Cube3x3,
    facelets::{ColorScheme3x3, Colored3x3, Facelets},
    pieces::{Center, Corner, Edge, Orientation3x3, Piece3x3},
};
pub use puzzles::{
    Puzzle,
    algorithm::Algorithm,
    algset::AlgSet,
    label::{ByIdentity, ByMembership, Labeled, Marker, Mask, PieceSet},
};

pub mod cube3x3_method_helpers {
    //! Tools for defining your own 3x3 method.
    pub use crate::methods::cube3x3::helpers::chain_steps;
}

/// Runs the Rust examples in `README.md` as doc tests, so they cannot drift from the code.
/// This item exists only while doc tests run and never appears in the docs.
#[cfg(doctest)]
#[doc = include_str!("../../../README.md")]
struct ReadmeDoctests;
