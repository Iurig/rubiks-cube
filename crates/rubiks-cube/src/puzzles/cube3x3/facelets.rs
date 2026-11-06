//! The sticker view of a cube state: which colour shows on each of the 54
//! facelets, printed as the usual unfolded net.
//!
//! A facelet is one square on the surface of the cube; a sticker is one face
//! of a piece. A cube state says which piece sits in each slot and how it is
//! turned, so it also says which sticker covers each facelet. A sticker is
//! named by the face it belongs to when solved, so every facelet reads as one
//! of `U F R B L D`.

use std::fmt::{self, Display};

use itertools::izip;
use owo_colors::{AnsiColors, OwoColorize};

use super::{
    Cube3x3,
    pieces::{Center, Corner, Edge, Orientation3x3, Piece3x3},
};
use crate::{Face, Mask, Piece, Puzzle, piece::index};

/// The colour on each facelet of a [`Cube3x3`], one `[Center; 9]` per face
/// in [`Center::ALL`] order, each read row by row as in [`Display`].
///
/// ```
/// use rubiks_cube::Cube3x3;
///
/// let cube = Cube3x3::from_moves("R")?;
/// let expected = [
///     "    UUF",
///     "    UUF",
///     "    UUF",
///     "LLL FFD RRR UBB",
///     "LLL FFD RRR UBB",
///     "LLL FFD RRR UBB",
///     "    DDB",
///     "    DDB",
///     "    DDB",
/// ]
/// .join("\n");
/// assert_eq!(cube.to_string(), expected);
/// # Ok::<(), rubiks_cube::ParseSequenceError>(())
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Facelets {
    faces: [[Center; 9]; 6],
}

/// Which slot and which of its faces each facelet belongs to, one `[_; 9]`
/// per face in [`Center::ALL`] order, `[U, F, R, B, L, D]`.
///
/// Each face is read as it is seen in the net below: `U` and `D` with `F`
/// toward the reader, the four side faces with `U` on top and unfolded left
/// to right as `L F R B`.
///
/// ```text
///     UUU
///     UUU
///     UUU
/// LLL FFF RRR BBB
/// LLL FFF RRR BBB
/// LLL FFF RRR BBB
///     DDD
///     DDD
///     DDD
/// ```
const NET: [[(Piece3x3, Center); 9]; 6] = {
    use Center::{B, D, F, L, R, U};
    use Piece3x3::{Center as Ce, Corner as Co, Edge as Ed};
    [
        [
            (Co(Corner::Ubl), U),
            (Ed(Edge::Ub), U),
            (Co(Corner::Ubr), U),
            (Ed(Edge::Ul), U),
            (Ce(U), U),
            (Ed(Edge::Ur), U),
            (Co(Corner::Ufl), U),
            (Ed(Edge::Uf), U),
            (Co(Corner::Ufr), U),
        ],
        [
            (Co(Corner::Ufl), F),
            (Ed(Edge::Uf), F),
            (Co(Corner::Ufr), F),
            (Ed(Edge::Fl), F),
            (Ce(F), F),
            (Ed(Edge::Fr), F),
            (Co(Corner::Dfl), F),
            (Ed(Edge::Df), F),
            (Co(Corner::Dfr), F),
        ],
        [
            (Co(Corner::Ufr), R),
            (Ed(Edge::Ur), R),
            (Co(Corner::Ubr), R),
            (Ed(Edge::Fr), R),
            (Ce(R), R),
            (Ed(Edge::Br), R),
            (Co(Corner::Dfr), R),
            (Ed(Edge::Dr), R),
            (Co(Corner::Dbr), R),
        ],
        [
            (Co(Corner::Ubr), B),
            (Ed(Edge::Ub), B),
            (Co(Corner::Ubl), B),
            (Ed(Edge::Br), B),
            (Ce(B), B),
            (Ed(Edge::Bl), B),
            (Co(Corner::Dbr), B),
            (Ed(Edge::Db), B),
            (Co(Corner::Dbl), B),
        ],
        [
            (Co(Corner::Ubl), L),
            (Ed(Edge::Ul), L),
            (Co(Corner::Ufl), L),
            (Ed(Edge::Bl), L),
            (Ce(L), L),
            (Ed(Edge::Fl), L),
            (Co(Corner::Dbl), L),
            (Ed(Edge::Dl), L),
            (Co(Corner::Dfl), L),
        ],
        [
            (Co(Corner::Dfl), D),
            (Ed(Edge::Df), D),
            (Co(Corner::Dfr), D),
            (Ed(Edge::Dl), D),
            (Ce(D), D),
            (Ed(Edge::Dr), D),
            (Co(Corner::Dbl), D),
            (Ed(Edge::Db), D),
            (Co(Corner::Dbr), D),
        ],
    ]
};

/// The three faces of a corner slot, clockwise as seen from outside the cube,
/// starting from its U or D face.
///
/// A piece is named by its home slot, so this is also the order of a corner
/// piece's stickers, starting from its U or D sticker. A twist of `t` puts
/// that sticker `t` steps clockwise from the slot's U or D face.
const fn corner_faces(corner: Corner) -> [Center; 3] {
    use Center::{B, D, F, L, R, U};
    match corner {
        Corner::Ubl => [U, L, B],
        Corner::Ubr => [U, B, R],
        Corner::Ufr => [U, R, F],
        Corner::Ufl => [U, F, L],
        Corner::Dfl => [D, L, F],
        Corner::Dfr => [D, F, R],
        Corner::Dbr => [D, R, B],
        Corner::Dbl => [D, B, L],
    }
}

/// The two faces of an edge slot, the one that defines its flip first: U or
/// D when the slot has one, otherwise F or B.
///
/// As with corners this is also the sticker order of the edge piece. An edge
/// is oriented when its first sticker is on the slot's first face.
const fn edge_faces(edge: Edge) -> [Center; 2] {
    use Center::{B, D, F, L, R, U};
    match edge {
        Edge::Ub => [U, B],
        Edge::Ur => [U, R],
        Edge::Uf => [U, F],
        Edge::Ul => [U, L],
        Edge::Fl => [F, L],
        Edge::Fr => [F, R],
        Edge::Br => [B, R],
        Edge::Bl => [B, L],
        Edge::Df => [D, F],
        Edge::Dr => [D, R],
        Edge::Db => [D, B],
        Edge::Dl => [D, L],
    }
}

/// The faces a slot has, in the order its stickers are counted.
fn slot_faces(slot: Piece3x3) -> Vec<Center> {
    match slot {
        Piece3x3::Center(c) => vec![c],
        Piece3x3::Edge(e) => edge_faces(e).to_vec(),
        Piece3x3::Corner(c) => corner_faces(c).to_vec(),
    }
}

/// The stickers `piece` shows when it sits in a slot with `orientation`, in
/// the same order as [`slot_faces`]. `None` when the orientation belongs to
/// another kind of piece.
fn stickers(piece: Piece3x3, orientation: Orientation3x3) -> Option<Vec<Center>> {
    match (piece, orientation) {
        (Piece3x3::Center(c), Orientation3x3::Fixed) => Some(vec![c]),
        (Piece3x3::Edge(e), Orientation3x3::Flip(flip)) => {
            let mut stickers = edge_faces(e);
            stickers.rotate_left(flip.value());
            Some(stickers.to_vec())
        }
        (Piece3x3::Corner(c), Orientation3x3::Twist(twist)) => {
            let mut stickers = corner_faces(c);
            stickers.rotate_right(twist.value());
            Some(stickers.to_vec())
        }
        _ => None,
    }
}

/// Where `facelet` falls in [`slot_faces`], or `None` when the slot has no
/// face there.
fn facelet_position(slot: Piece3x3, facelet: Center) -> Option<usize> {
    slot_faces(slot).iter().position(|&f| f == facelet)
}

/// The sticker showing on the `facelet` face of `slot`, or `None` when the
/// slot has no face there.
fn sticker(cube: &Cube3x3, slot: Piece3x3, facelet: Center) -> Option<Center> {
    let position = facelet_position(slot, facelet)?;
    stickers(cube.piece_at(slot), cube.orientation_at(slot))?
        .get(position)
        .copied()
}

/// The facelet position, as in [`slot_faces`], that holds a piece's first
/// sticker (its U or D sticker for corners, the flip-defining one for edges)
/// under `orientation`. `None` for centers, which have no orientation.
///
/// Both [`stickers`] rotations move the first sticker to the position equal
/// to the orientation's value.
const fn orientation_position(orientation: Orientation3x3) -> Option<usize> {
    match orientation {
        Orientation3x3::Fixed => None,
        Orientation3x3::Flip(flip) => Some(flip.value()),
        Orientation3x3::Twist(twist) => Some(twist.value()),
    }
}

/// The character a [`Mask`] shows on the `facelet` face of `slot`; see
/// [`Display for Mask<Cube3x3>`](Mask#impl-Display-for-Mask<Cube3x3>).
fn mask_char(mask: &Mask<Cube3x3>, slot: Piece3x3, facelet: Center) -> char {
    let position = facelet_position(slot, facelet)
        .expect("the `const _` block below checks every entry of `NET`");
    let condition = mask.condition(slot);
    match (condition.label, condition.orient) {
        (Some(piece), Some(orientation)) => stickers(piece, orientation)
            .and_then(|stickers| stickers.get(position).copied())
            .map_or('?', face_to_letter),
        (Some(Piece3x3::Center(c)), None) => face_to_letter(c),
        (Some(_), None) => '?',
        (None, Some(orientation)) if orientation_position(orientation) == Some(position) => 'o',
        (None, _) => '.',
    }
}

const fn face_to_letter(face: Center) -> char {
    match face {
        Center::U => 'U',
        Center::F => 'F',
        Center::R => 'R',
        Center::B => 'B',
        Center::L => 'L',
        Center::D => 'D',
    }
}

const fn letter_to_face(letter: char) -> Option<Face> {
    match letter {
        'U' => Some(Center::U),
        'F' => Some(Center::F),
        'R' => Some(Center::R),
        'B' => Some(Center::B),
        'L' => Some(Center::L),
        'D' => Some(Center::D),
        _ => None,
    }
}

impl Facelets {
    /// The nine facelets of `face`, read row by row as in [`Display`].
    #[must_use]
    pub const fn face(&self, face: Face) -> &[Face; 9] {
        &self.faces[index(face)]
    }

    /// Which sticker is in a position of a face, measured from the top left.
    #[must_use]
    pub fn sticker(&self, pos: (usize, usize)) -> Option<Face> {
        self.to_string()
            .lines()
            .nth(pos.0)
            .and_then(|line| letter_to_face(line.chars().nth(pos.1 + pos.1 / 3)?))
    }
}

impl Cube3x3 {
    /// The colour on every facelet of a cube state.
    #[must_use]
    #[expect(
        clippy::missing_panics_doc,
        reason = "the `const _` block below checks every entry of `NET`, so `sticker` never returns `None`"
    )]
    pub fn facelets(&self) -> Facelets {
        Facelets {
            faces: NET.map(|face| {
                face.map(|(slot, facelet)| {
                    sticker(self, slot, facelet)
                        .expect("the `const _` block below checks every entry of `NET`")
                })
            }),
        }
    }
}

/// Writes one character per facelet as the unfolded net, `faces` in
/// [`Center::ALL`] order like [`NET`].
#[expect(
    clippy::indexing_slicing,
    reason = "`index` of a `Center` is below 6, the length of `faces`"
)]
fn write_net(f: &mut fmt::Formatter<'_>, faces: &[[char; 9]; 6]) -> fmt::Result {
    let rows = |face: Center| faces[index(face)].chunks(3);
    let write_row =
        |f: &mut fmt::Formatter<'_>, row: &[char]| row.iter().try_for_each(|&c| write!(f, "{c}"));
    for row in rows(Center::U) {
        write!(f, "    ")?;
        write_row(f, row)?;
        writeln!(f)?;
    }
    for (l, fr, r, b) in izip!(
        rows(Center::L),
        rows(Center::F),
        rows(Center::R),
        rows(Center::B)
    ) {
        write_row(f, l)?;
        write!(f, " ")?;
        write_row(f, fr)?;
        write!(f, " ")?;
        write_row(f, r)?;
        write!(f, " ")?;
        write_row(f, b)?;
        writeln!(f)?;
    }
    for (i, row) in rows(Center::D).enumerate() {
        if i > 0 {
            writeln!(f)?;
        }
        write!(f, "    ")?;
        write_row(f, row)?;
    }
    Ok(())
}

impl Display for Facelets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_net(f, &self.faces.map(|face| face.map(face_to_letter)))
    }
}

impl Display for Cube3x3 {
    /// The facelet net; see [`Facelets`].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.facelets().fmt(f)
    }
}

/// A colored representation of a [`Cube3x3`].
///
/// Should be created by invoking [`colored`](Cube3x3::colored) or
/// [`with_colorscheme`](Cube3x3::with_color_scheme).
///
/// # Example
///
/// ```
/// use rubiks_cube::{ColorScheme3x3, Cube3x3};
///
/// println!("{}", Cube3x3::default().colored());
/// println!(
///     "{}",
///     Cube3x3::default().with_color_scheme(ColorScheme3x3::JAPANESE)
/// );
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Colored3x3 {
    cube: Cube3x3,
    scheme: ColorScheme3x3,
}

/// A representation of a [`Cube3x3`]s color scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorScheme3x3 {
    /// The color of the U face.
    pub u: AnsiColors,
    /// The color of the F face.
    pub f: AnsiColors,
    /// The color of the R face.
    pub r: AnsiColors,
    /// The color of the L face.
    pub l: AnsiColors,
    /// The color of the B face.
    pub b: AnsiColors,
    /// The color of the D face.
    pub d: AnsiColors,
}

impl ColorScheme3x3 {
    /// The standard [`Cube3x3`] color scheme, held in WCA scrambling orientation.
    pub const STANDARD: Self = Self {
        u: AnsiColors::BrightWhite,
        f: AnsiColors::Green,
        r: AnsiColors::Red,
        l: AnsiColors::BrightYellow,
        b: AnsiColors::Blue,
        d: AnsiColors::Yellow,
    };

    /// The Japanese [`Cube3x3`] color scheme, held in WCA scrambling orientation.
    pub const JAPANESE: Self = Self {
        u: AnsiColors::BrightWhite,
        f: AnsiColors::Green,
        r: AnsiColors::Red,
        l: AnsiColors::Yellow,
        b: AnsiColors::BrightYellow,
        d: AnsiColors::Blue,
    };

    /// The [`AnsiColors`] of a given [`Face`] under the [`ColorScheme3x3`].
    #[must_use]
    pub const fn face_color(self, face: Face) -> AnsiColors {
        match face {
            Face::U => self.u,
            Face::F => self.f,
            Face::R => self.r,
            Face::L => self.l,
            Face::D => self.d,
            Face::B => self.b,
        }
    }
}

impl Cube3x3 {
    /// Returns a colored representation of the given [`Cube3x3`].
    ///
    /// The representation is colored by the standard color scheme held in the standard WCA
    /// scrambling orientation.
    #[must_use]
    pub const fn colored(&self) -> Colored3x3 {
        Colored3x3 {
            cube: *self,
            scheme: ColorScheme3x3::STANDARD,
        }
    }

    /// Returns a colored representation of a given [`Cube3x3`] under a provided [`ColorScheme3x3`].
    #[must_use]
    pub const fn with_color_scheme(&self, scheme: ColorScheme3x3) -> Colored3x3 {
        Colored3x3 {
            cube: *self,
            scheme,
        }
    }
}

impl Display for Colored3x3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.cube.to_string().chars().try_for_each(|c| match c {
            'U' => write!(f, "{}", 'U'.on_color(self.scheme.u)),
            'F' => write!(f, "{}", 'F'.on_color(self.scheme.f)),
            'R' => write!(f, "{}", 'R'.on_color(self.scheme.r)),
            'L' => write!(f, "{}", 'L'.on_color(self.scheme.l)),
            'B' => write!(f, "{}", 'B'.on_color(self.scheme.b)),
            'D' => write!(f, "{}", 'D'.on_color(self.scheme.d)),

            c => write!(f, "{c}"),
        })
    }
}

/// The net of what the mask asks for, one character per facelet:
///
/// - a face letter where the mask names both the piece and its orientation, so the sticker is known
///   (and on every facelet of a named center);
/// - `?` where the mask names the piece but not its orientation;
/// - `o` where the mask names only an orientation, on the facelet that must show the piece's U or D
///   sticker (for edges, its U/D sticker or else its F/B one); the slot's other facelets show `.`;
/// - `.` where the mask asks nothing.
///
/// ```
/// use rubiks_cube::{Cube3x3, Edge, Mask, Piece3x3};
///
/// let uf_solved = Mask::<Cube3x3>::from_pieces([Edge::Uf]);
/// let expected = [
///     "    ...",
///     "    ...",
///     "    .U.",
///     "... .F. ... ...",
///     "... ... ... ...",
///     "... ... ... ...",
///     "    ...",
///     "    ...",
///     "    ...",
/// ]
/// .join("\n");
/// assert_eq!(uf_solved.to_string(), expected);
/// ```
impl Display for Mask<Cube3x3> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_net(
            f,
            &NET.map(|face| face.map(|(slot, facelet)| mask_char(self, slot, facelet))),
        )
    }
}

/// Every entry of [`NET`] names a face its slot has, sits on the face its row
/// belongs to, and appears once.
const _: () = {
    const fn same_face(a: Center, b: Center) -> bool {
        a as u8 == b as u8
    }
    const fn same_slot(a: Piece3x3, b: Piece3x3) -> bool {
        match (a, b) {
            (Piece3x3::Center(x), Piece3x3::Center(y)) => same_face(x, y),
            (Piece3x3::Edge(x), Piece3x3::Edge(y)) => x as u8 == y as u8,
            (Piece3x3::Corner(x), Piece3x3::Corner(y)) => x as u8 == y as u8,
            _ => false,
        }
    }
    const fn slot_has(slot: Piece3x3, face: Center) -> bool {
        match slot {
            Piece3x3::Center(c) => same_face(c, face),
            Piece3x3::Edge(e) => {
                let faces = edge_faces(e);
                same_face(faces[0], face) || same_face(faces[1], face)
            }
            Piece3x3::Corner(c) => {
                let faces = corner_faces(c);
                same_face(faces[0], face) || same_face(faces[1], face) || same_face(faces[2], face)
            }
        }
    }
    let mut face = 0;
    while face < NET.len() {
        let mut i = 0;
        while i < NET[face].len() {
            let (slot, facelet) = NET[face][i];
            assert!(slot_has(slot, facelet), "facelet not on its slot");
            assert!(
                same_face(facelet, Center::ALL[face]),
                "facelet on the wrong face of the net"
            );
            let mut other_face = 0;
            while other_face < NET.len() {
                let mut j = 0;
                while j < NET[other_face].len() {
                    let (other_slot, other_facelet) = NET[other_face][j];
                    assert!(
                        (face == other_face && i == j)
                            || !(same_slot(slot, other_slot) && same_face(facelet, other_facelet)),
                        "facelet listed twice"
                    );
                    j += 1;
                }
                other_face += 1;
            }
            i += 1;
        }
        face += 1;
    }
};

#[cfg(test)]
#[expect(
    clippy::panic_in_result_fn,
    reason = "tests should panic if failed, and return result for `?` convenience"
)]
mod tests {

    use super::*;
    use crate::{Piece, Puzzle};

    fn net(rows: [&str; 9]) -> String {
        let [u1, u2, u3, m1, m2, m3, d1, d2, d3] = rows;
        format!("    {u1}\n    {u2}\n    {u3}\n{m1}\n{m2}\n{m3}\n    {d1}\n    {d2}\n    {d3}")
    }

    #[test]
    fn solved_cube_shows_each_face_in_one_colour() {
        assert_eq!(
            Cube3x3::default().to_string(),
            net([
                "UUU",
                "UUU",
                "UUU",
                "LLL FFF RRR BBB",
                "LLL FFF RRR BBB",
                "LLL FFF RRR BBB",
                "DDD",
                "DDD",
                "DDD",
            ])
        );
    }

    // Physical facts: `R` carries F stickers to U, U to B, B to D, D to F.
    #[test]
    fn r_moves_the_right_column_of_each_face_along_the_ring() -> anyhow::Result<()> {
        assert_eq!(
            Cube3x3::from_moves("R")?.to_string(),
            net([
                "UUF",
                "UUF",
                "UUF",
                "LLL FFD RRR UBB",
                "LLL FFD RRR UBB",
                "LLL FFD RRR UBB",
                "DDB",
                "DDB",
                "DDB",
            ])
        );
        Ok(())
    }

    // `U` carries F stickers to L, L to B, B to R, R to F.
    #[test]
    fn u_moves_the_top_row_of_the_side_faces() -> anyhow::Result<()> {
        assert_eq!(
            Cube3x3::from_moves("U")?.to_string(),
            net([
                "UUU",
                "UUU",
                "UUU",
                "FFF RRR BBB LLL",
                "LLL FFF RRR BBB",
                "LLL FFF RRR BBB",
                "DDD",
                "DDD",
                "DDD",
            ])
        );
        Ok(())
    }

    // `F` carries U stickers to R, R to D, D to L, L to U. It twists the four
    // F corners and flips the four F edges, so this pins both conventions.
    #[test]
    fn f_moves_the_ring_around_the_front_face() -> anyhow::Result<()> {
        assert_eq!(
            Cube3x3::from_moves("F")?.to_string(),
            net([
                "UUU",
                "UUU",
                "LLL",
                "LLD FFF URR BBB",
                "LLD FFF URR BBB",
                "LLD FFF URR BBB",
                "RRR",
                "DDD",
                "DDD",
            ])
        );
        Ok(())
    }

    // `M` follows `L`: U stickers go to F, F to D, D to B, B to U.
    #[test]
    fn m_moves_the_middle_column_and_the_centers() -> anyhow::Result<()> {
        assert_eq!(
            Cube3x3::from_moves("M")?.to_string(),
            net([
                "UBU",
                "UBU",
                "UBU",
                "LLL FUF RRR BDB",
                "LLL FUF RRR BDB",
                "LLL FUF RRR BDB",
                "DFD",
                "DFD",
                "DFD",
            ])
        );
        Ok(())
    }

    // `y` follows `U`: the whole cube turns, so R stickers show at F.
    #[test]
    fn y_rotates_the_side_faces_whole() -> anyhow::Result<()> {
        assert_eq!(
            Cube3x3::from_moves("y")?.to_string(),
            net([
                "UUU",
                "UUU",
                "UUU",
                "FFF RRR BBB LLL",
                "FFF RRR BBB LLL",
                "FFF RRR BBB LLL",
                "DDD",
                "DDD",
                "DDD",
            ])
        );
        Ok(())
    }

    #[test]
    fn face_reads_the_same_facelets_display_prints() -> anyhow::Result<()> {
        use Center::{B, D, F, R, U};
        let facelets = Cube3x3::from_moves("R")?.facelets();
        assert_eq!(facelets.face(U), &[U, U, F, U, U, F, U, U, F]);
        assert_eq!(facelets.face(B), &[U, B, B, U, B, B, U, B, B]);
        assert_eq!(facelets.face(R), &[R; 9]);
        assert_eq!(facelets.face(D), &[D, D, B, D, D, B, D, D, B]);
        Ok(())
    }

    #[test]
    fn every_state_shows_nine_stickers_of_each_colour() {
        for seed in 0..50 {
            let facelets = Cube3x3::scrambled_with_seed(seed).facelets();
            for colour in Center::ALL {
                let count = Center::ALL
                    .iter()
                    .flat_map(|&face| facelets.face(face))
                    .filter(|&&c| c == colour)
                    .count();
                assert_eq!(count, 9, "{colour:?} in\n{facelets}");
            }
        }
    }

    #[test]
    fn empty_mask_shows_only_dots() {
        let dots = "... ... ... ...";
        assert_eq!(
            Mask::<Cube3x3>::default().to_string(),
            net(["...", "...", "...", dots, dots, dots, "...", "...", "...",])
        );
    }

    #[test]
    fn fully_solved_mask_shows_the_solved_net() {
        let goal = Mask::from_pieces(Cube3x3::ALL_PIECES.iter().copied());
        assert_eq!(goal.to_string(), Cube3x3::default().to_string());
    }

    // Roux's CO asks only that the U corners be oriented: an `o` on the U
    // facelet of each U corner, nothing anywhere else.
    #[test]
    fn orientation_only_mask_marks_the_u_facelet_of_each_corner() {
        let u_corners = [Corner::Ubl, Corner::Ubr, Corner::Ufr, Corner::Ufl].map(Piece3x3::Corner);
        let dots = "... ... ... ...";
        assert_eq!(
            Mask::<Cube3x3>::from_labels_and_orientations([], u_corners).to_string(),
            net(["o.o", "...", "o.o", dots, dots, dots, "...", "...", "...",])
        );
    }

    #[test]
    fn piece_without_orientation_shows_question_marks() {
        let mask = Mask::<Cube3x3>::from_pieces_and_orientations([Edge::Fr], [] as [Edge; 0]);
        assert_eq!(
            mask.to_string(),
            net([
                "...",
                "...",
                "...",
                "... ... ... ...",
                "... ..? ?.. ...",
                "... ... ... ...",
                "...",
                "...",
                "...",
            ])
        );
    }

    #[test]
    fn a_move_and_its_inverse_restore_the_net() -> anyhow::Result<()> {
        let solved = Cube3x3::default().to_string();
        for m in [
            "R", "U", "F", "L", "D", "B", "M", "E", "S", "x", "y", "z", "r",
        ] {
            let there_and_back = Cube3x3::from_moves(&format!("{m} {m}'"))?;
            assert_eq!(there_and_back.to_string(), solved, "{m}");
        }
        Ok(())
    }

    /// Wraps each face letter of a plain net in the ANSI background code `bg` gives its face, and
    /// the code that resets the background after it.
    fn paint(plain: &str, bg: [(char, u8); 6]) -> String {
        plain
            .chars()
            .map(|c| match bg.iter().find(|&&(face, _)| face == c) {
                Some((_, code)) => format!("\x1b[{code}m{c}\x1b[49m"),
                None => c.to_string(),
            })
            .collect()
    }

    // ANSI background codes: 41 red, 42 green, 43 yellow (our orange), 44 blue, 47 white,
    // 103 bright yellow.
    #[test]
    fn colored_paints_r_in_the_standard_scheme() -> anyhow::Result<()> {
        let r = Cube3x3::from_moves("R")?;
        let standard = [
            ('U', 47),
            ('F', 42),
            ('R', 41),
            ('L', 43),
            ('B', 44),
            ('D', 103),
        ];
        assert_eq!(r.colored().to_string(), paint(&r.to_string(), standard));
        Ok(())
    }

    #[test]
    fn with_color_scheme_paints_r_in_the_japanese_scheme() -> anyhow::Result<()> {
        let r = Cube3x3::from_moves("R")?;
        let japanese_codes = [
            ('U', 47),
            ('F', 42),
            ('R', 41),
            ('L', 43),
            ('B', 103),
            ('D', 44),
        ];
        assert_eq!(
            r.with_color_scheme(ColorScheme3x3::JAPANESE).to_string(),
            paint(&r.to_string(), japanese_codes)
        );
        Ok(())
    }
}
