use crate::{ColorScheme3x3, Cube3x3, Face};
use ratatui::{
    buffer::{Buffer, Cell},
    layout::{Constraint, Layout, Rect},
    style::Color,
    text::Text,
    widgets::Widget,
};

const fn set_color(cell: &mut Cell, color: Color) -> &mut Cell {
    cell.set_bg(color).set_fg(color)
}

/// A [`ratatui`] [`Widget`] for displaying a [`Cube3x3`].
#[derive(Debug, Clone, Copy)]
pub struct WidgetCube3x3 {
    cube: Cube3x3,
    color_scheme: ColorScheme3x3,
    background: Color,
}

impl WidgetCube3x3 {
    /// Applies the given [`ColorScheme3x3`] to the widget.
    #[must_use]
    pub const fn with_colorscheme(&self, color_scheme: ColorScheme3x3) -> Self {
        Self {
            cube: self.cube,
            color_scheme,
            background: self.background,
        }
    }

    /// The [`Cube3x3`] the widget displays.
    #[must_use]
    pub const fn cube(&self) -> Cube3x3 {
        self.cube
    }

    /// A mutable reference for the [`Cube3x3`] the widged displays.
    pub const fn mut_cube(&mut self) -> &mut Cube3x3 {
        &mut self.cube
    }
}

impl Cube3x3 {
    /// A [`ratatui`] [`Widget`] displaying the given [`Cube3x3`].
    #[must_use]
    pub const fn widget(self) -> WidgetCube3x3 {
        WidgetCube3x3 {
            cube: self,
            color_scheme: ColorScheme3x3::STANDARD,
            background: Color::Rgb(0, 0, 0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WidgetFace3x3 {
    face: [Face; 9],
    color_scheme: ColorScheme3x3,
    background: Color,
}

const FACE_SIZE: u16 = 7;
const FACELET_LENGTH: u16 = 4;

impl Widget for WidgetFace3x3 {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        for dx in 0..=(3 * FACELET_LENGTH) {
            let x = area.left() + dx;
            buf[(x, area.top() + FACE_SIZE - 1)]
                .set_fg(self.background)
                .set_symbol("▀");
        }

        for dy in (0..FACE_SIZE - 1).step_by(2) {
            for dx in 0..=(3 * FACELET_LENGTH) {
                let x = area.left() + dx;
                let y = area.top() + dy;

                if dx % FACELET_LENGTH == 0 {
                    set_color(&mut buf[(x, y)], self.background).set_symbol("█");
                    set_color(&mut buf[(x, y + 1)], self.background).set_symbol("█");
                } else {
                    buf[(x, y)]
                        .set_fg(
                            self.color_scheme
                                .face_color(
                                    *self
                                        .face
                                        .get((3 * (dy / 2) + dx / FACELET_LENGTH) as usize)
                                        .expect("broken index invariants"),
                                )
                                .into(),
                        )
                        .set_symbol("▄")
                        .set_bg(self.background);
                    set_color(
                        &mut buf[(x, y + 1)],
                        self.color_scheme
                            .face_color(
                                *self
                                    .face
                                    .get((3 * (dy / 2) + dx / FACELET_LENGTH) as usize)
                                    .expect("broken index invariants"),
                            )
                            .into(),
                    )
                    .set_symbol("█");
                }
            }
        }
    }
}

impl Widget for WidgetCube3x3 {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        if area.width < 3 * FACELET_LENGTH + 1 || area.height < 3 * FACE_SIZE {
            Text::from("Terminal too small")
                .centered()
                .render(area.centered_vertically(Constraint::Length(1)), buf);
            return;
        }

        let areas = area
            .layout::<3>(&Layout::horizontal([
                Constraint::Length(14),
                Constraint::Length(14),
                Constraint::Length(13),
            ]))
            .map(|vert| vert.layout::<3>(&Layout::vertical([Constraint::Fill(1); 3])));

        macro_rules! render_face {
            ($face:ident, ($x:literal, $y:literal)) => {
                WidgetFace3x3 {
                    face: *self.cube.facelets().face(Face::$face),
                    color_scheme: self.color_scheme,
                    background: self.background,
                }
                .render(areas[$x][$y], buf);
            };
        }

        render_face!(U, (1, 0));
        render_face!(F, (1, 1));
        render_face!(R, (2, 1));
        render_face!(L, (0, 1));
        render_face!(D, (1, 2));
    }
}
