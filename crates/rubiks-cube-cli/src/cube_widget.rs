use ansi_to_tui::IntoText;
use owo_colors::AnsiColors;
use ratatui::{
    Frame,
    buffer::{Buffer, Cell},
    layout::{Constraint, Layout, Rect},
    style::Color,
    widgets::{Block, Paragraph, Widget},
};
use rubiks_cube::{ColorScheme3x3, Cube3x3, Face, Facelets};

pub fn draw_lib_native_widget(solve_number: usize, cube: &Cube3x3, area: Rect, frame: &mut Frame) {
    let widget = Paragraph::new(
        cube.colored()
            .to_string()
            .as_bytes()
            .into_text()
            .unwrap_or_else(|e| {
                panic!("original string should've been possible to parse to ratatui::Text: {e}")
            }),
    )
    .block(Block::bordered().title(format!("Solve {}", solve_number + 1)));

    frame.render_widget(widget, area);
}

const fn set_color(cell: &mut Cell, color: Color) -> &mut Cell {
    cell.set_bg(color).set_fg(color)
}

#[derive(Debug)]
struct ColoredCube {
    facelets: Facelets,
    color_scheme: ColorScheme3x3,
    background: Color,
}

struct ColoredFace {
    face: [Face; 9],
    color_scheme: ColorScheme3x3,
    background: Color,
}

impl Widget for ColoredFace {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        const FACE_SIZE: u16 = 7;
        const FACELET_LENGTH: u16 = 4;

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
                        .set_fg(owo_ansi_to_ratatui(
                            self.color_scheme.face_color(
                                *self
                                    .face
                                    .get((3 * (dy / 2) + dx / FACELET_LENGTH) as usize)
                                    .expect("broken index invariants"),
                            ),
                        ))
                        .set_symbol("▄")
                        .set_bg(self.background);
                    set_color(
                        &mut buf[(x, y + 1)],
                        owo_ansi_to_ratatui(
                            self.color_scheme.face_color(
                                *self
                                    .face
                                    .get((3 * (dy / 2) + dx / FACELET_LENGTH) as usize)
                                    .expect("broken index invariants"),
                            ),
                        ),
                    )
                    .set_symbol("█");
                }
            }
        }
    }
}

impl Widget for ColoredCube {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let areas = area
            .layout::<3>(&Layout::horizontal([
                Constraint::Length(14),
                Constraint::Length(14),
                Constraint::Length(13),
            ]))
            .map(|vert| vert.layout::<3>(&Layout::vertical([Constraint::Fill(1); 3])));

        macro_rules! render_face {
            ($face:ident, ($x:literal, $y:literal)) => {
                ColoredFace {
                    face: *self.facelets.face(Face::$face),
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

pub fn draw_cli_widget(solve_number: usize, cube: &Cube3x3, area: Rect, frame: &mut Frame) {
    let border = Block::bordered().title(format!("Solve {}", solve_number + 1));

    frame.render_widget(border.clone(), area);

    let inner_area = border.inner(area);

    let colored = ColoredCube {
        facelets: cube.facelets(),
        color_scheme: ColorScheme3x3::STANDARD,
        background: Color::Black,
    };

    frame.render_widget(colored, inner_area);
}

const fn owo_ansi_to_ratatui(color: AnsiColors) -> Color {
    match color {
        AnsiColors::Black => Color::Black,
        AnsiColors::Red => Color::Red,
        AnsiColors::Green => Color::Green,
        AnsiColors::Yellow => Color::Yellow,
        AnsiColors::Blue => Color::Blue,
        AnsiColors::Magenta => Color::Magenta,
        AnsiColors::Cyan => Color::Cyan,
        AnsiColors::White => Color::Gray,
        AnsiColors::BrightBlack => Color::DarkGray,
        AnsiColors::BrightRed => Color::LightRed,
        AnsiColors::BrightGreen => Color::LightGreen,
        AnsiColors::BrightYellow => Color::LightYellow,
        AnsiColors::BrightBlue => Color::LightBlue,
        AnsiColors::BrightMagenta => Color::LightMagenta,
        AnsiColors::BrightCyan => Color::LightCyan,
        AnsiColors::BrightWhite => Color::White,
        AnsiColors::Default => Color::Reset,
    }
}
