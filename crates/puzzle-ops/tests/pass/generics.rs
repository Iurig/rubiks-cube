use puzzle_ops::PuzzleOps;
use std::marker::PhantomData;

#[derive(Debug, Copy, Default, Clone, Hash, PartialEq, Eq, PuzzleOps)]
struct Unit<T: Generic> {
    phantom: std::marker::PhantomData<T>,
}

trait Generic:
    std::any::Any
    + std::fmt::Debug
    + std::cmp::Eq
    + std::clone::Clone
    + std::hash::Hash
    + std::marker::Copy
    + std::marker::Send
    + std::marker::Sync
    + std::hash::Hash
    + std::marker::Copy
    + std::hash::Hash
    + std::default::Default
{
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
enum Empty {}

impl rubiks_cube::Inv for Empty {
    fn inv(self) -> Self {
        unreachable!()
    }
}

impl<T: Generic> std::ops::Mul<Empty> for Unit<T> {
    type Output = Self;
    fn mul(self, _rhd: Empty) -> Self {
        unreachable!()
    }
}

impl std::str::FromStr for Empty {
    type Err = rubiks_cube::ParseMoveError;
    fn from_str(_: &str) -> Result<Self, rubiks_cube::ParseMoveError> {
        panic!()
    }
}

impl<T: Generic> rubiks_cube::Indexed for Unit<T> {
    const COUNT: usize = 1;
    fn index(&self) -> usize {
        0
    }
    fn from_index(index: usize) -> Self {
        if index == 0 {
            Self {
                phantom: PhantomData,
            }
        } else {
            panic!()
        }
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

impl<T: Generic> std::ops::Add for Unit<T> {
    type Output = Self;
    fn add(self, _: Self) -> Self {
        Unit {
            phantom: PhantomData,
        }
    }
}

impl<T: Generic> std::fmt::Display for Unit<T> {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}

impl std::fmt::Display for Empty {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        unreachable!()
    }
}

impl<T: Generic> rubiks_cube::Puzzle for Unit<T> {
    type Piece = Self;
    type Orientation = Self;
    type Move = Empty;

    fn slot_of(&self, _piece: Self::Piece) -> Self::Piece {
        Unit {
            phantom: PhantomData,
        }
    }

    fn is_solved(&self) -> bool {
        true
    }

    fn piece_at(&self, _slot: Self::Piece) -> Self::Piece {
        Unit {
            phantom: PhantomData,
        }
    }

    fn orientation_at(&self, _slot: Self::Piece) -> Self::Orientation {
        Unit {
            phantom: PhantomData,
        }
    }

    fn scrambled_with_seed(_seed: u64) -> Self {
        Unit {
            phantom: PhantomData,
        }
    }

    fn scramble_with_seed(
        _seed: u64,
    ) -> Result<rubiks_cube::Algorithm<Self>, rubiks_cube::SolveError> {
        Ok(rubiks_cube::Algorithm::from_iter([]))
    }
}

fn main() {}
