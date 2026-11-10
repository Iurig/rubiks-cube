use std::fmt::Debug;

use crate::Indexed;
use crate::ops::Inv;
use crate::zn::Zn;
#[expect(
    unnameable_types,
    reason = "This trait only exists to seal private implementations"
)]
pub mod private {
    pub trait Sealed {}
}

/// One of the `N` pieces of a kind, named by its home slot.
pub trait Piece<const N: usize>: Indexed + Copy + Eq + private::Sealed + Debug {
    /// Every piece, in slot order.
    const ALL: [Self; N];
}

/// A uniformly random order of all `N` pieces.
fn random_permutation<const N: usize, P: Piece<N>>(rng: &mut fastrand::Rng) -> [P; N] {
    let mut permutation = P::ALL;
    for i in 0..N {
        permutation.swap(i, rng.usize(i..N));
    }
    permutation
}

#[must_use]
#[expect(
    clippy::redundant_pub_crate,
    reason = "`pub(crate)` says `index` is internal even if this module becomes public"
)]
pub(crate) const fn index<P, const N: usize>(piece: P) -> usize
where
    P: Piece<N>,
{
    const { assert!(std::mem::size_of::<P>() == 1) }
    // SAFETY: `Piece` is sealed, and the only implementations come from `new_piece!`, which
    // makes a fieldless `#[repr(u8)]` enum. Such an enum is one byte holding its discriminant,
    // so reading that byte is in bounds and gives an initialized `u8`. `Sealed` alone does not
    // check the layout; the macro does, and the size assertion above catches a non-macro impl
    // whose type is not one byte.
    unsafe { (&raw const piece).cast::<u8>().read() as usize }
}

/// Where each of `N` pieces sits and how it is oriented, mod `O`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PieceConfiguration<P, const N: usize, const O: usize> {
    pub(crate) permutation: [P; N],
    pub(crate) orientation: [Zn<O>; N],
}

impl<P, const N: usize, const O: usize> Default for PieceConfiguration<P, N, O>
where
    P: Piece<N>,
{
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl<P, const N: usize, const O: usize> Inv for PieceConfiguration<P, N, O>
where
    P: Piece<N>,
{
    #[expect(
        clippy::indexing_slicing,
        reason = "wrong indexing should panic instead of failing silently"
    )]
    fn inv(self) -> Self {
        let mut inv = Self::IDENTITY;
        for ((&piece, orientation), home) in
            self.permutation.iter().zip(&self.orientation).zip(P::ALL)
        {
            inv.permutation[index(piece)] = home;
            inv.orientation[index(piece)] = -*orientation;
        }
        inv
    }
}

impl<P, const N: usize, const O: usize> PieceConfiguration<P, N, O>
where
    P: Piece<N>,
{
    /// The solved state for a given piece type
    pub const IDENTITY: Self = {
        const { assert!(N == P::COUNT) };
        Self {
            permutation: P::ALL,
            orientation: [Zn::ZERO; N],
        }
    };

    /// The piece now sitting in `slot`.
    ///
    /// Slots and pieces share a type: a piece is named by its home slot. So
    /// `piece_at(Ubr) == Ufr` reads "the UFR piece sits in the UBR slot", and
    /// the returned piece can be fed back in as the next slot when tracing a
    /// cycle, the way blind memorization does.
    #[must_use]
    pub const fn piece_at(&self, slot: P) -> P {
        self.permutation[index(slot)]
    }

    /// The orientation held at `slot`: the twist or flip of the piece sitting
    /// there, relative to the slot.
    ///
    /// Centers have no orientation, so on a center configuration this always
    /// returns zero.
    #[must_use]
    pub const fn orientation_at(&self, slot: P) -> Zn<O> {
        self.orientation[index(slot)]
    }

    /// # Panics
    ///
    /// Panics if `self.permutation` is not a valid permutation
    #[must_use]
    pub(crate) fn parity(&self) -> Zn<2> {
        let mut visited = Vec::new();
        let mut par = Zn::ZERO;
        for p in self.permutation {
            if !visited.contains(&p) {
                visited.push(p);
                let mut travel = self
                    .permutation
                    .get(index(p))
                    .expect("self.permutation must be a valid permutation");
                let mut cycle_size = 1;
                while travel != &p {
                    visited.push(*travel);
                    cycle_size += 1;
                    travel = self
                        .permutation
                        .get(index(*travel))
                        .expect("self.permutation must be a valid permutation");
                }
                par += Zn::new(cycle_size - 1);
            }
        }
        par
    }

    #[must_use]
    pub(crate) fn orientation_sum(&self) -> Zn<O> {
        self.orientation.iter().copied().sum()
    }

    /// Compose permutations done by `self` with `other`
    #[must_use]
    #[expect(
        clippy::indexing_slicing,
        reason = "`i < N`, and `index(piece) < N` by the `const _` block in `pieces.rs`"
    )]
    pub(crate) fn then(&self, other: &Self) -> Self {
        let mut composed = Self::IDENTITY;
        for i in 0..N {
            composed.permutation[i] = self.permutation[index(other.permutation[i])];
            composed.orientation[i] =
                self.orientation[index(other.permutation[i])] + (other.orientation[i]);
        }
        composed
    }

    pub(crate) const fn cycle<const CYCLE_SIZE: usize, const CYCLE_AMOUNT: usize>(
        to_cycle: [[P; CYCLE_SIZE]; CYCLE_AMOUNT],
    ) -> Self {
        let mut resp = Self::IDENTITY;
        let mut i = 0;
        while i < CYCLE_AMOUNT {
            let mut j = 0;
            while j < CYCLE_SIZE {
                resp.permutation[index(to_cycle[i][(j + 1) % CYCLE_SIZE])] = to_cycle[i][j];
                j += 1;
            }
            i += 1;
        }
        resp
    }

    /// A random permutation with random orientations. The orientations may not sum to zero and
    /// the permutation parity is not fixed, so the caller repairs both.
    pub(crate) fn random_state(rng: &mut fastrand::Rng) -> Self {
        Self {
            permutation: random_permutation(rng),
            orientation: {
                let mut or = [Zn::ZERO; N];
                for flip in or.iter_mut().take(N) {
                    *flip = Zn::new(rng.usize(0..O));
                }
                or
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzles::cube3x3::pieces::*;
    #[test]
    fn corner_and_edge_all_match_discriminants() {
        for (i, c) in Corner::ALL.iter().enumerate() {
            assert_eq!(*c as usize, i);
            assert_eq!(Corner::from_index(i), *c);
        }
        for (i, e) in Edge::ALL.iter().enumerate() {
            assert_eq!(*e as usize, i);
            assert_eq!(Edge::from_index(i), *e);
        }
    }

    #[test]
    fn cycle_is_a_permutation_and_moves_pieces_forward_and_leaves_rest() {
        use Corner::{Dbr, Dfr, Ubr, Ufr};
        let mut perm = CornerConfiguration::cycle([[Ufr, Ubr, Dbr, Dfr]]).permutation;
        assert_eq!(perm[index(Ubr)], Ufr);
        assert_eq!(perm[index(Dbr)], Ubr);
        assert_eq!(perm[index(Dfr)], Dbr);
        assert_eq!(perm[index(Ufr)], Dfr);
        for c in [Corner::Ubl, Corner::Ufl, Corner::Dfl, Corner::Dbl] {
            assert_eq!(perm[c as usize], c);
        }
        perm.sort();
        assert_eq!(perm, Corner::ALL);
    }

    #[test]
    fn cycle_with_disjoint_cycles_is_a_permutation() {
        use Edge::{Dl, Dr, Ub, Uf};
        let mut perm = EdgeConfiguration::cycle::<2, 2>([[Ub, Uf], [Dl, Dr]]).permutation;
        assert_eq!(perm[Uf as usize], Ub);
        assert_eq!(perm[Ub as usize], Uf);
        perm.sort();
        assert_eq!(perm, Edge::ALL);
    }
}
