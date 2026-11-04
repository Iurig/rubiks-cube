#![doc = include_str!("DESIGN.md")]
#![allow(
    clippy::pedantic,
    clippy::nursery,
    clippy::allow_attributes_without_reason,
    missing_docs
)]

use rubiks_cube::*;

fn main() {
    let eolr = SearchStep::builder("EOLR", AlgSet::from_static_parts("M U"))
        .goal(PieceSet::from_algset(&AlgSet::<Cube3x3>::from_static(
            "M' U2 M U2\n M U2 M' U2",
        )))
        .clone();

    let advanced_roux = Technique::<Cube3x3>::start_with(SearchStep::builder(
        "FB",
        AlgSet::from_static_parts("R U F B D M r"),
    ))
    .then(SearchStep::builder(
        "SB",
        AlgSet::from_static_parts("R U r M"),
    ))
    .then(
        SearchStep::builder(
            "CMLL",
            AlgSet::from_static(include_str!(
                "../src/methods/cube3x3/algsets/cmll/one_look.txt"
            )),
        )
        .free_algs(AlgSet::from_static_parts("U M"))
        .goal(PieceSet::from_algset(&AlgSet::from_static("U M' U' \n M")))
        .clone(),
    )
    .then(eolr)
    .finish(SearchStep::builder("LSE", AlgSet::from_static_parts("M U")))
    .unwrap();

    let mut cube = Cube3x3::scrambled_with_seed(3);

    advanced_roux.solve(&mut cube).unwrap();

    assert!(cube.is_solved());
}
