use rubiks_cube::*;

#[test]
fn mask_ignores_duplicates() {
    let pieces1 = [
        Piece3x3::Center(Center::F),
        Piece3x3::Center(Center::U),
        Piece3x3::Edge(Edge::Uf),
    ];
    let pieces2 = [
        Piece3x3::Center(Center::F),
        Piece3x3::Center(Center::F),
        Piece3x3::Center(Center::U),
        Piece3x3::Edge(Edge::Uf),
    ];
    assert_eq!(
        Mask::<Cube3x3>::from_pieces_and_orientations(pieces1, pieces1),
        Mask::<Cube3x3>::from_pieces_and_orientations(pieces2, pieces2)
    );
}

#[test]
fn mask_ignores_order() {
    let pieces1 = [
        Piece3x3::Center(Center::F),
        Piece3x3::Center(Center::U),
        Piece3x3::Edge(Edge::Uf),
    ];
    let pieces2 = [
        Piece3x3::Center(Center::F),
        Piece3x3::Edge(Edge::Uf),
        Piece3x3::Center(Center::U),
    ];
    assert_eq!(
        Mask::<Cube3x3>::from_pieces_and_orientations(pieces1, pieces1),
        Mask::<Cube3x3>::from_pieces_and_orientations(pieces2, pieces2)
    );
}

#[test]
fn masked_default_applies_to_default_but_not_r() {
    let pieces1 = [
        Piece3x3::Center(Center::F),
        Piece3x3::Center(Center::U),
        Piece3x3::Edge(Edge::Ur),
    ];
    let pieces2 = [
        Piece3x3::Center(Center::F),
        Piece3x3::Edge(Edge::Ur),
        Piece3x3::Center(Center::U),
    ];
    assert!(Mask::from_pieces_and_orientations(pieces1, pieces2).applies_to(&Cube3x3::default()));
    assert!(
        !Mask::from_pieces_and_orientations(pieces1, pieces2)
            .applies_to(&Cube3x3::from_moves("R").unwrap())
    );
}

#[test]
fn masked_default_applies_to_default_and_r_if_mask_excludes_r() {
    let pieces1 = [
        Piece3x3::Center(Center::F),
        Piece3x3::Center(Center::U),
        Piece3x3::Edge(Edge::Ul),
    ];
    let pieces2 = [
        Piece3x3::Center(Center::F),
        Piece3x3::Edge(Edge::Ul),
        Piece3x3::Center(Center::U),
    ];
    assert!(Mask::from_pieces_and_orientations(pieces1, pieces2).applies_to(&Cube3x3::default()));
    assert!(
        Mask::from_pieces_and_orientations(pieces1, pieces2)
            .applies_to(&Cube3x3::from_moves("R").unwrap())
    );
}

#[test]
fn masked_with_correct_permutation_but_wrong_orientation_parses_correctly() {
    let pieces = [Corner::Ufr];
    assert!(
        Mask::from_pieces_and_orientations(pieces, [] as [Corner; 0])
            .applies_to(&Cube3x3::from_moves("R U").unwrap())
    );
    assert!(
        !Mask::from_pieces_and_orientations([] as [Corner; 0], pieces)
            .applies_to(&Cube3x3::from_moves("R U").unwrap())
    );
}

#[test]
fn masked_with_correct_orientation_but_wrong_permutation_parses_correctly() {
    let pieces = [Corner::Ufr];
    assert!(
        !Mask::from_pieces_and_orientations(pieces, [] as [Corner; 0])
            .applies_to(&Cube3x3::from_moves("U").unwrap())
    );
    assert!(
        Mask::from_pieces_and_orientations([] as [Corner; 0], pieces)
            .applies_to(&Cube3x3::from_moves("U").unwrap())
    );
}

#[test]
fn empty_mask_always_applies() {
    for seed in 0..100 {
        assert!(
            Mask::from_labels_and_orientations([], [])
                .applies_to(&Cube3x3::scrambled_with_seed(seed))
        );
    }
}
