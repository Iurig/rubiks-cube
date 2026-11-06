use ansi_to_tui::IntoText;
use itertools::Itertools;
use owo_colors::AnsiColors;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Padding, Paragraph},
};
use rubiks_cube::{ColorScheme3x3, Cube3x3, Face};

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

pub fn draw_cli_widget(solve_number: usize, cube: &Cube3x3, area: Rect, frame: &mut Frame) {
    let border = Block::bordered().title(format!("Solve {}", solve_number + 1));

    frame.render_widget(border.clone(), area);

    let inner_area = border.inner(area);

    #[rustfmt::skip]
    let faces_grid = [
        [None,           Some(Face::U),           None,],
        [Some(Face::L),  Some(Face::F),  Some(Face::R),],
        [None,           Some(Face::D),           None,],
    ];

    let facelet_layout = inner_area
        .centered_horizontally(Constraint::Length(4 * 9))
        .layout::<9>(&Layout::horizontal([Constraint::Length(4); 9]))
        .iter()
        .map(|rec| {
            rec.centered_vertically(Constraint::Length(2 * 9))
                .layout::<9>(&Layout::vertical([Constraint::Length(2); 9]))
        })
        .collect::<Vec<_>>();

    (0..9)
        .cartesian_product(0..9)
        .map(|(line, column)| {
            frame.render_widget(
                if let Some(_) = faces_grid[line / 3][column / 3] {
                    Paragraph::new(Text::styled(
                        "██",
                        Style::new()
                            .bg(owo_ansi_to_ratatui(
                                ColorScheme3x3::STANDARD.face_color(
                                    cube.facelets()
                                        .sticker((line, column))
                                        .expect(&format!("pos = ({line}, {column})")),
                                ),
                            ))
                            .fg(owo_ansi_to_ratatui(
                                ColorScheme3x3::STANDARD.face_color(
                                    cube.facelets()
                                        .sticker((line, column))
                                        .expect(&format!("pos = ({line}, {column})")),
                                ),
                            )),
                    ))
                    .centered()
                } else {
                    Paragraph::new(Text::raw("  ")).block(Block::new())
                },
                facelet_layout[column][line],
            );
        })
        .for_each(drop);
}

fn owo_ansi_to_ratatui(color: AnsiColors) -> Color {
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
