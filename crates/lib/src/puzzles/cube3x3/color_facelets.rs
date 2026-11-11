use std::fmt::Display;

use crate::{Cube3x3, Face};
use anstyle::{AnsiColor, Color, Reset, RgbColor, Style};
use termprofile::TermProfile;

/// A colored representation of a [`Cube3x3`].
///
/// Should be created by invoking [`colored`](Cube3x3::colored) or
/// [`with_color_scheme`](Cube3x3::with_color_scheme).
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
    pub u: Color,
    /// The color of the F face.
    pub f: Color,
    /// The color of the R face.
    pub r: Color,
    /// The color of the L face.
    pub l: Color,
    /// The color of the B face.
    pub b: Color,
    /// The color of the D face.
    pub d: Color,
}

const WHITE: (u8, u8, u8) = (255, 255, 255);
const GREEN: (u8, u8, u8) = (92, 230, 92);
const RED: (u8, u8, u8) = (185, 37, 37);
const ORANGE: (u8, u8, u8) = (255, 126, 51);
const BLUE: (u8, u8, u8) = (0, 84, 230);
const YELLOW: (u8, u8, u8) = (255, 235, 77);

macro_rules! rgb {
    ($color_code:ident) => {
        RgbColor($color_code.0, $color_code.1, $color_code.2)
    };
}

impl ColorScheme3x3 {
    /// The standard [`Cube3x3`] color scheme, held in WCA scrambling orientation.
    pub const STANDARD: Self = Self {
        u: Color::Rgb(rgb!(WHITE)),
        f: Color::Rgb(rgb!(GREEN)),
        r: Color::Rgb(rgb!(RED)),
        l: Color::Rgb(rgb!(ORANGE)),
        b: Color::Rgb(rgb!(BLUE)),
        d: Color::Rgb(rgb!(YELLOW)),
    };

    /// The Japanese [`Cube3x3`] color scheme, held in WCA scrambling orientation.
    pub const JAPANESE: Self = Self {
        u: Color::Ansi(AnsiColor::BrightWhite),
        f: Color::Ansi(AnsiColor::Green),
        r: Color::Ansi(AnsiColor::Red),
        l: Color::Ansi(AnsiColor::BrightYellow),
        b: Color::Ansi(AnsiColor::Yellow),
        d: Color::Ansi(AnsiColor::Blue),
    };

    /// Styles a [`ColorScheme3x3`] according to the terminal.
    ///
    /// Uses `truecolor` in terminals that support it, failing that, falls back to `ANSI 256`,
    /// failing that, falls back to basic `ANSI`, and ultimately, simply black.
    #[must_use]
    pub fn on_terminal(&self, profile: TermProfile) -> Self {
        Self {
            u: profile
                .adapt_color(self.u)
                .unwrap_or(Color::Ansi(AnsiColor::Black)),
            f: profile
                .adapt_color(self.f)
                .unwrap_or(Color::Ansi(AnsiColor::Black)),
            r: profile
                .adapt_color(self.r)
                .unwrap_or(Color::Ansi(AnsiColor::Black)),
            l: profile
                .adapt_color(self.l)
                .unwrap_or(Color::Ansi(AnsiColor::Black)),
            b: profile
                .adapt_color(self.b)
                .unwrap_or(Color::Ansi(AnsiColor::Black)),
            d: profile
                .adapt_color(self.d)
                .unwrap_or(Color::Ansi(AnsiColor::Black)),
        }
    }

    /// The [`Color`] of a given [`Face`] under the [`ColorScheme3x3`].
    #[must_use]
    pub const fn face_color(self, face: Face) -> Color {
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.cube.to_string().chars().try_for_each(|c| match c {
            'U' => write!(f, "{}U{Reset}", Style::new().bg_color(Some(self.scheme.u))),
            'F' => write!(f, "{}F{Reset}", Style::new().bg_color(Some(self.scheme.f))),
            'R' => write!(f, "{}R{Reset}", Style::new().bg_color(Some(self.scheme.r))),
            'L' => write!(f, "{}L{Reset}", Style::new().bg_color(Some(self.scheme.l))),
            'B' => write!(f, "{}B{Reset}", Style::new().bg_color(Some(self.scheme.b))),
            'D' => write!(f, "{}D{Reset}", Style::new().bg_color(Some(self.scheme.d))),

            c => write!(f, "{c}"),
        })
    }
}
