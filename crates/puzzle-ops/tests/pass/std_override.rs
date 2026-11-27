type Result = ();
type Option = ();

struct From;
struct Mul;
struct Inv;
struct Puzzle;

impl Unit {
    fn default(&self) -> Self {
        Unit
    }
    fn mul(&self) -> Self {
        Unit
    }
}

use puzzle_ops::PuzzleOps;

#[derive(Debug, Copy, Default, Clone, Hash, PartialEq, Eq, PuzzleOps)]
struct Unit;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
enum Empty {}

impl rubiks_cube::Inv for Empty {
    fn inv(self) -> Self {
        unreachable!()
    }
}

impl std::ops::Mul<Empty> for Unit {
    type Output = Self;
    fn mul(self, _rhd: Empty) -> Self {
        unreachable!()
    }
}

impl std::str::FromStr for Empty {
    type Err = rubiks_cube::ParseMoveError;
    fn from_str(_: &str) -> std::result::Result<Self, rubiks_cube::ParseMoveError> {
        panic!()
    }
}

impl rubiks_cube::Indexed for Unit {
    const COUNT: usize = 1;
    fn index(&self) -> usize {
        0
    }
    fn from_index(index: usize) -> Self {
        if index == 0 { Self } else { panic!() }
    }
}

impl rubiks_cube::Indexed for Empty {
    const COUNT: usize = 0;
    fn index(&self) -> usize {
        unreachable!()
    }
    fn from_index(_index: usize) -> Self {
        panic!()
    }
}

impl std::ops::Add for Unit {
    type Output = Self;
    fn add(self, _: Self) -> Self {
        Unit
    }
}

impl std::fmt::Display for Unit {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        Ok(())
    }
}

impl std::fmt::Display for Empty {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        unreachable!()
    }
}

impl rubiks_cube::Puzzle for Unit {
    type Piece = Self;
    type Orientation = Self;
    type Move = Empty;

    fn slot_of(&self, _piece: Self::Piece) -> Self::Piece {
        Unit
    }

    fn is_solved(&self) -> bool {
        true
    }

    fn piece_at(&self, _slot: Self::Piece) -> Self::Piece {
        Unit
    }

    fn orientation_at(&self, _slot: Self::Piece) -> Self::Orientation {
        Unit
    }

    fn scrambled_with_seed(_seed: u64) -> Self {
        Unit
    }

    fn scramble_with_seed(
        _seed: u64,
    ) -> std::result::Result<rubiks_cube::Algorithm<Self>, rubiks_cube::SolveError> {
        Ok(rubiks_cube::Algorithm::from_iter([]))
    }
}

fn main() {}
