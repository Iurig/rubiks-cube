use std::sync::{Arc, LazyLock};

use super::helpers::set_options;
use crate::methods::cube3x3::helpers::{
    BlockGoal, LastLayer, ZBLL_ALGS, chain_steps, into_steps, split_by_blocks,
};

use crate::{AlgSet, BuildContext, Cube3x3, Method, PieceSet, Step, Technique};

/// The ZZ method for the 3x3 Rubik's Cube: EO line, first two layers, then last layer.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ZZ {
    eoline: EoLine,
    f2l: F2L,
    ll: LL,
}

impl ZZ {
    /// The ZZ-a variant of the ZZ method. Uses ZBLL on the last layer.
    pub const ZZ_A: Self = Self {
        eoline: EoLine,
        f2l: F2L,
        ll: LL::OneLook,
    };
    set_options!("EOLine", eoline, EoLine);
    set_options!("F2L", f2l, F2L);
    set_options!("last layer", ll, LL);
}

/// How [`ZZ`] builds its first step, the `EOLine`.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct EoLine;
/// How [`ZZ`] builds its F2L.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct F2L;
/// How [`ZZ`] builds its last layer.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LL {
    /// 2-look last layer.
    TwoLook {
        /// The first look.
        first: FirstLook,
        /// The second look.
        second: SecondLook,
    },
    /// 1-look last layer, solved with ZBLL.
    OneLook,
}
/// The first step of a two-look ZZ last layer.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum FirstLook {
    /// Orientation of the Corners of the Last Layer.
    #[default]
    Ocll,
    /// Orientation and permutation of the Corners of the Last Layer.
    Coll,
}
/// The second step of a two-look ZZ last layer.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SecondLook {
    /// Standard PLL
    #[default]
    OneLook,
    /// Two-look PLL
    TwoLook,
}

impl Default for LL {
    fn default() -> Self {
        Self::TwoLook {
            first: FirstLook::default(),
            second: SecondLook::default(),
        }
    }
}

impl Method<Cube3x3> for ZZ {
    fn to_technique_in(&self, context: &BuildContext) -> Technique<Cube3x3> {
        let eoline_tech = match self.eoline {
            EoLine => {
                chain_steps!(context: context, moves: &EO_LINE_MOVES, goal: &F2L_MOVES, "EO Line")
            }
        };

        let f2l_tech = match self.f2l {
            F2L => vec![Arc::from(split_by_blocks(
                "F2L",
                PieceSet::from_algset(&F2L_MOVES),
                F2L_BLOCKS.iter(),
                F2L_MOVES.clone(),
            )) as Arc<dyn Step<Cube3x3>>],
        };

        let ll_tech = match self.ll {
            LL::TwoLook {
                first: oll,
                second: pll,
            } => {
                let oll_tech = match oll {
                    FirstLook::Ocll => into_steps(LastLayer::ocll()),
                    FirstLook::Coll => into_steps(LastLayer::coll()),
                };
                let pll_tech = match pll {
                    SecondLook::OneLook => into_steps(LastLayer::pll()),
                    SecondLook::TwoLook => {
                        let corners = into_steps(LastLayer::cpll());
                        let edges = into_steps(LastLayer::epll());
                        [corners, edges].concat()
                    }
                };
                [oll_tech, pll_tech].concat()
            }
            LL::OneLook => into_steps(LastLayer::zbll()),
        };

        [eoline_tech, f2l_tech, ll_tech]
            .into_iter()
            .flatten()
            .collect()
    }
}

static EO_LINE_MOVES: LazyLock<AlgSet<Cube3x3>> =
    LazyLock::new(|| AlgSet::from_static_parts("F B U R L D"));
static F2L_MOVES: LazyLock<AlgSet<Cube3x3>> = LazyLock::new(|| AlgSet::from_static_parts("U R L"));
static F2L_BLOCKS: LazyLock<[BlockGoal; 2]> = LazyLock::new(|| {
    let f2l = PieceSet::from_algset(&ZBLL_ALGS);
    [
        BlockGoal::new(
            &f2l,
            &PieceSet::from_algset(&AlgSet::from_static_parts("R U")),
            &F2L_MOVES,
        ),
        BlockGoal::new(
            &f2l,
            &PieceSet::from_algset(&AlgSet::from_static_parts("L U")),
            &F2L_MOVES,
        ),
    ]
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Puzzle;

    /// How many last layers each option has to solve.
    const LAST_LAYERS: usize = 500;

    /// Random products of last-layer algorithms: what ZZ's F2L can leave, with F2L solved and
    /// every edge oriented.
    fn random_last_layers() -> Vec<Cube3x3> {
        let pool = LastLayer::pll()
            .algs
            .combined_with(&LastLayer::ocll().algs)
            .combined_with(&LastLayer::coll().algs)
            .combined_with(&AlgSet::from_static_parts("U"));
        let after_f2l = PieceSet::from_algset(&ZBLL_ALGS);
        let algs = pool.algs();
        let mut random = fastrand::Rng::with_seed(0);
        (0..LAST_LAYERS)
            .map(|_| {
                let state = (0..6).fold(Cube3x3::default(), |cube, _| {
                    cube.apply(&algs[random.usize(..algs.len())])
                });
                assert!(after_f2l.applies_to(&state), "not a last layer:\n{state}");
                state
            })
            .collect()
    }

    /// Runs `steps` in order on every random last layer, reporting all the ones that fail.
    fn assert_solves_every_last_layer(steps: &[LastLayer]) {
        let mut failures = Vec::new();
        for start in random_last_layers() {
            let mut cube = start;
            let failed = steps
                .iter()
                .find_map(|step| step.solve(&mut cube).err().map(|e| (step.name(), e)));
            if let Some((step, error)) = failed {
                failures.push(format!("{step}: {error}\n{start}"));
            } else if !cube.is_solved() {
                failures.push(format!("ended unsolved\n{start}"));
            }
        }
        assert!(
            failures.is_empty(),
            "{} of {LAST_LAYERS} last layers failed. The first:\n{}",
            failures.len(),
            failures[0]
        );
    }

    #[test]
    fn zbll_solves_every_last_layer() {
        assert_solves_every_last_layer(&[LastLayer::zbll()]);
    }

    #[test]
    fn ocll_then_pll_solves_every_last_layer() {
        assert_solves_every_last_layer(&[LastLayer::ocll(), LastLayer::pll()]);
    }

    #[test]
    fn ocll_then_two_look_pll_solves_every_last_layer() {
        assert_solves_every_last_layer(&[LastLayer::ocll(), LastLayer::cpll(), LastLayer::epll()]);
    }

    #[test]
    fn coll_then_pll_solves_every_last_layer() {
        assert_solves_every_last_layer(&[LastLayer::coll(), LastLayer::pll()]);
    }

    #[test]
    fn coll_then_two_look_pll_solves_every_last_layer() {
        assert_solves_every_last_layer(&[LastLayer::coll(), LastLayer::cpll(), LastLayer::epll()]);
    }
}
