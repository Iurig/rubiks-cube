use std::sync::{Arc, LazyLock};

use super::helpers::{chain_steps, set_options};
use crate::{
    AlgSet, BuildContext, Cube3x3, Indexed, Method, Piece3x3, PieceSet, SearchStep, Step,
    methods::Technique,
};

/// The Roux method for the 3x3 Rubik's Cube: first block, second block, CMLL, then the last six
/// edges.
///
/// Each stage can be split into smaller steps. Start from `Roux::default()` and pick the
/// split of a stage with its method:
///
/// ```rust
/// use rubiks_cube::{Roux, roux};
///
/// let roux = Roux::default()
///     .first_block(roux::FirstBlock::SquarePair)
///     .cmll(roux::Cmll::TwoLook);
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Roux {
    first_block: FirstBlock,
    second_block: SecondBlock,
    cmll: Cmll,
    lse: Lse,
}

impl Roux {
    set_options!("first block", first_block, FirstBlock);
    set_options!("second block", second_block, SecondBlock);
    set_options!("CMLL", cmll, Cmll);
    set_options!("LSE", lse, Lse);
}

/// How [`Roux`] builds the first block, the 1x2x3 block on the left.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum FirstBlock {
    /// One search for the whole block.
    #[default]
    OneLook,
    /// A 1x2x2 square, in the front or back, then the pair that finishes the block.
    SquarePair,
    /// The DL edge, then a square, then the pair.
    EdgePairPair,
}

/// How [`Roux`] builds the second block, the 1x2x3 block on the right.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum SecondBlock {
    /// One search for the whole block.
    OneLook,
    /// A 1x2x2 square, in the front or back, then the pair that finishes the block.
    SquarePair,
    /// The DR edge, then a square, then the pair.
    #[default]
    EdgePairPair,
}

/// How [`Roux`] solves the U-layer corners.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum Cmll {
    /// One algorithm from the full CMLL set.
    #[default]
    OneLook,
    /// One algorithm to orient the corners, then one to permute them.
    TwoLook,
}

/// How [`Roux`] solves the last six edges.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum Lse {
    /// One search for all six edges with `U` and `M`.
    #[default]
    Eolr,
    //EO,
}

static FB_MOVES: LazyLock<AlgSet<Cube3x3>> =
    LazyLock::new(|| AlgSet::from_static_parts("F U R L D B M r"));
static SB_MOVES: LazyLock<AlgSet<Cube3x3>> = LazyLock::new(|| AlgSet::from_static_parts("U R M r"));
static FREE_AUF: LazyLock<AlgSet<Cube3x3>> = LazyLock::new(|| AlgSet::from_static_parts("U"));
static CMLL_ALGS: LazyLock<AlgSet<Cube3x3>> =
    LazyLock::new(|| AlgSet::from_static(include_str!("algsets/cmll/one_look.txt")));
static LSE_MOVES: LazyLock<AlgSet<Cube3x3>> = LazyLock::new(|| AlgSet::from_static_parts("U M"));
static LSE_KEEPING_CORNERS: LazyLock<AlgSet<Cube3x3>> =
    LazyLock::new(|| AlgSet::from_static("M\n U M U'"));

impl Method<Cube3x3> for Roux {
    fn to_technique_in(&self, context: &BuildContext) -> Technique<Cube3x3> {
        let fb_tech = match self.first_block {
            FirstBlock::OneLook => chain_steps!(
                context: context, moves: &FB_MOVES, goal: &SB_MOVES,
                "FB",
            ),
            FirstBlock::SquarePair => chain_steps!(
                context: context, moves: &FB_MOVES, goal: &SB_MOVES,
                "FB Square" => (AlgSet::from_static_parts("B"), AlgSet::from_static_parts("F")),
                "FB Pair",
            ),
            FirstBlock::EdgePairPair => chain_steps!(
                context: context, moves: &FB_MOVES, goal: &SB_MOVES,
                "DL" => (AlgSet::from_static_parts("F B")),
                "FB Square" => (AlgSet::from_static_parts("B"), AlgSet::from_static_parts("F")),
                "FB Pair",
            ),
        };

        let sb_tech = match self.second_block {
            SecondBlock::OneLook => chain_steps!(
                context: context, moves: &SB_MOVES, goal: &CMLL_ALGS,
                "SB"
            ),
            SecondBlock::SquarePair => chain_steps!(
                context: context, moves: &SB_MOVES, goal: &CMLL_ALGS,
                "SB Square" => (AlgSet::from_static("r U r'"), AlgSet::from_static("r' U r")),
                "SB Pair"
            ),
            SecondBlock::EdgePairPair => chain_steps!(
                context: context, moves: &SB_MOVES, goal: &CMLL_ALGS,
                "DR" => (AlgSet::from_static("r U r'\n r' U r")),
                "SB Square" => (AlgSet::from_static("r U r'"), AlgSet::from_static("r' U r")),
                "SB Pair",
            ),
        };

        let after_cmll = PieceSet::from_algset(&LSE_KEEPING_CORNERS);

        let cmll_tech: Vec<Arc<dyn Step<Cube3x3>>> = match self.cmll {
            Cmll::OneLook => chain_steps!(
                context: context, moves: &CMLL_ALGS.combined_with(&LSE_MOVES),
                goal: &LSE_KEEPING_CORNERS,
                free: &FREE_AUF,
                "CMLL",
            ),
            Cmll::TwoLook => [
                chain_steps!(
                    context: context, moves: &AlgSet::from_static(include_str!("algsets/cmll/co.txt")).combined_with(&LSE_KEEPING_CORNERS),
                    goal: &LSE_MOVES,
                    free: &FREE_AUF,
                    "CO",
                ),
                chain_steps!(
                    context: context, moves: &AlgSet::from_static(include_str!("algsets/cmll/cp.txt")).combined_with(&LSE_KEEPING_CORNERS),
                    goal: &LSE_KEEPING_CORNERS,
                    "CP",
                ),
            ]
            .concat(),
        };

        let lse_tech: Vec<Arc<dyn Step<Cube3x3>>> = match self.lse {
            Lse::Eolr => vec![Arc::new(
                SearchStep::builder("LSE", LSE_MOVES.clone())
                    .prerequisite(after_cmll)
                    .goal(PieceSet::from_pieces(Piece3x3::all()))
                    .build_in(context)
                    .expect("manually constructed step should construct"),
            )],
            //[`LseOptions::EO`] => (),
        };

        [fb_tech, sb_tech, cmll_tech, lse_tech]
            .into_iter()
            .flatten()
            .collect()
    }
}

#[cfg(test)]
mod test {
    #![expect(
        clippy::panic_in_result_fn,
        reason = "`?` reports setup failures; `assert!` reports the property under test failing"
    )]

    use anyhow::Context;
    use itertools::iproduct;

    use super::*;
    use crate::{Cube3x3, Inv};

    #[test]
    fn cmll_step_leaves_the_cube_with_cmll_solved() {
        let cmll = Roux::default()
            .to_technique()
            .steps
            .into_iter()
            .find(|s| s.name() == "CMLL")
            .unwrap();
        let sune = Cube3x3::from_moves("R U R' U R U2 R'").unwrap();
        for scramble in [Cube3x3::from_moves("U2").unwrap(), sune.inv()] {
            let mut cube = scramble;
            cmll.solve(&mut cube).unwrap();
            assert!(cmll.is_done(&cube), "not solved:\n{cube}");
        }
    }

    /// Every `to_technique` call builds new step objects, so memos are shared through the global
    /// cache, keyed by `after` and the move sets. A step built again with the same goals shares its
    /// memo, and so does the one-look block with the pair that finishes it, because the pair's
    /// `before` lies inside its `after`.
    #[test]
    fn rebuilt_roux_steps_share_their_memos() {
        let fb_start = PieceSet::from_algset(&FB_MOVES);
        let fb_goal = PieceSet::from_algset(&SB_MOVES);
        let back_square =
            PieceSet::from_algset(&SB_MOVES.combined_with(&AlgSet::from_static_parts("F")));

        let search = |name: &'static str, before: &PieceSet<Cube3x3>, after: &PieceSet<Cube3x3>| {
            SearchStep::builder(name, FB_MOVES.clone())
                .prerequisite(before.clone())
                .goal(after.clone())
                .build()
                .unwrap()
        };

        let fb = search("FB", &fb_start, &fb_goal);
        let fb_again = search("FB", &fb_start, &fb_goal);
        let fb_pair = search("FB Pair", &back_square, &fb_goal);
        let fb_square = search("FB Square", &fb_start, &back_square);

        assert!(fb.shares_memo_with(&fb_again));
        assert!(fb.shares_memo_with(&fb_pair));
        assert!(!fb.shares_memo_with(&fb_square));
    }

    /// `R`, `U`, and `F` turns never touch the L center, DBL, BL, or DL, so the back square stays
    /// solved and costs nothing, while `F` breaks the front square.
    #[test]
    fn split_fb_builds_the_back_square_when_it_is_cheaper() -> anyhow::Result<()> {
        let roux = Roux {
            first_block: FirstBlock::SquarePair,
            ..Roux::default()
        };
        let mut cube = Cube3x3::from_moves("F R U")?;

        let solution = roux.solve(&mut cube)?;

        let fb_segment = solution
            .iter()
            .next()
            .context("the solution has no segments")?;
        assert_eq!(fb_segment.name(), "FB Square", "{solution}");
        assert!(
            fb_segment.moves().is_empty(),
            "the back square was already solved:\n{solution}"
        );
        Ok(())
    }

    /// `R U R'` displaces the FR edge and DFR corner but returns DR, BR, and DBR home, and
    /// neither `R` nor `U` touches the first block. So the second block's back square is solved
    /// and its front square is not.
    #[test]
    fn split_sb_builds_the_back_square_when_it_is_cheaper() -> anyhow::Result<()> {
        let roux = Roux {
            second_block: SecondBlock::SquarePair,
            ..Roux::default()
        };
        let mut cube = Cube3x3::from_moves("R U R'")?;

        let solution = roux.solve(&mut cube)?;

        let sb_segment = solution
            .iter()
            .find(|segment| segment.name().starts_with("SB"))
            .context("the solution has no SB segment")?;
        assert_eq!(sb_segment.name(), "SB Square", "{solution}");
        assert!(
            sb_segment.moves().is_empty(),
            "the back square was already solved:\n{solution}"
        );
        Ok(())
    }

    /// Every combination of the options.
    fn every_options() -> impl Iterator<Item = Roux> {
        use Cmll as C;
        use FirstBlock as F;
        use SecondBlock as S;
        iproduct!(
            [F::OneLook, F::SquarePair, F::EdgePairPair],
            [S::OneLook, S::SquarePair, S::EdgePairPair],
            [C::OneLook, C::TwoLook]
        )
        .map(|(fb, sb, cmll)| Roux {
            first_block: fb,
            second_block: sb,
            cmll,
            lse: Lse::Eolr,
        })
    }

    /// `Method` relies on `is_done` to catch a step that returns without meeting its goal, so
    /// every step must tell its goal apart from a cube that misses it.
    #[test]
    fn every_roux_step_is_done_on_a_solved_cube_and_not_on_a_scrambled_one() -> anyhow::Result<()> {
        let scrambled = Cube3x3::from_moves("R U' F2 L D' B R2 U F' L2 D B' U2 R'")?;

        for step in every_options().flat_map(|options| options.to_technique().steps) {
            assert!(
                step.is_done(&Cube3x3::default()),
                "{} is not done on a solved cube",
                step.name()
            );
            assert!(
                !step.is_done(&scrambled),
                "{} is done on a scrambled cube:\n{scrambled}",
                step.name()
            );
        }
        Ok(())
    }

    /// Each switch picks its own steps, independently of the others, and the steps stay in
    /// solving order.
    #[test]
    fn options_pick_the_steps_in_solving_order() {
        for options in every_options() {
            let names: Vec<String> = options
                .to_technique()
                .steps
                .iter()
                .map(|step| step.name().to_string())
                .collect();

            let mut expected: Vec<&str> = Vec::new();
            expected.extend_from_slice(match options.first_block {
                FirstBlock::OneLook => &["FB"][..],
                FirstBlock::SquarePair => &["FB Square", "FB Pair"],
                FirstBlock::EdgePairPair => &["DL", "FB Square", "FB Pair"],
            });
            expected.extend_from_slice(match options.second_block {
                SecondBlock::OneLook => &["SB"][..],
                SecondBlock::SquarePair => &["SB Square", "SB Pair"],
                SecondBlock::EdgePairPair => &["DR", "SB Square", "SB Pair"],
            });
            expected.extend_from_slice(match options.cmll {
                Cmll::OneLook => &["CMLL"][..],
                Cmll::TwoLook => &["CO", "CP"],
            });
            expected.push("LSE");

            assert_eq!(names, expected, "{options:?}");
        }
    }
}
