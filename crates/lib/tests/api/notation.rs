use rubiks_cube::*;
use std::error::Error;

#[test]
fn multiple_moves_break_down_correctly() -> anyhow::Result<()> {
    assert_eq!(
        Cube3x3::from_moves("R U R' U'")?,
        Cube3x3::default()
            .apply_moves("R")?
            .apply_moves("U")?
            .apply_moves("R'")?
            .apply_moves("U'")?
    );
    Ok(())
}

#[test]
fn bad_move_error_names_its_line_and_position() {
    let e = Cube3x3::from_moves(
        "R U R'
    F U F'
    Mw' M",
    )
    .unwrap_err();

    let _expected = rubiks_cube::ParseMoveError::BadModifier {
        invalid_move: "Mw'".to_string(),
        modifier: "w'".to_string(),
    };

    assert_eq!(e.line(), 3);
    assert_eq!(e.position(), 1);
    assert!(matches!(
        e.source()
            .expect("bad string should error with source")
            .downcast_ref::<ParseMoveError>(),
        Some(_expected)
    ));
}

#[test]
fn incorrect_strings_return_error() {
    let cases = [
        (
            "Q",
            ParseMoveError::BadPart {
                invalid_move: "Q".to_string(),
                part: "Q".to_string(),
            },
            1,
        ),
        (
            "R Q U",
            ParseMoveError::BadPart {
                invalid_move: "Q".to_string(),
                part: "Q".to_string(),
            },
            2,
        ),
        (
            "R3",
            ParseMoveError::BadModifier {
                invalid_move: "R3".to_string(),
                modifier: "3".to_string(),
            },
            1,
        ),
    ];
    for (text, _cause, position) in cases {
        let e = Cube3x3::from_moves(text).unwrap_err();
        assert!(matches!(
            e.source().unwrap().downcast_ref::<ParseMoveError>(),
            Some(_cause),
        ));
        assert_eq!(e.line(), 1, "{text}");
        assert_eq!(e.position(), position, "{text}");
    }
}

#[test]
fn r_prime_is_inverse_of_r() -> anyhow::Result<()> {
    let r = Cube3x3::from_moves("R")?;
    let r_prime = Cube3x3::from_moves("R'")?;
    assert_eq!(r.inv(), r_prime);
    Ok(())
}

#[test]
fn sequences_without_moves_leave_the_cube_unchanged() -> anyhow::Result<()> {
    let cube = Cube3x3::from_moves("R U")?;
    assert_eq!(cube.apply_moves("")?, cube);
    assert_eq!(cube.apply_moves("// nothing here")?, cube);
    assert_eq!(
        Cube3x3::from_moves("\n  // only comments\n")?,
        Cube3x3::default()
    );
    Ok(())
}
