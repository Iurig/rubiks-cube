use ansi_to_tui::IntoText;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{Block, Paragraph},
};
use rubiks_cube::Cube3x3;

use crate::app::{App, Mode};

pub fn view(app: &App, frame: &mut Frame) {
    match app.mode {
        Mode::BrowseSolves => walkthrough_recon(app, frame),
        Mode::Inspect => inspect_solve(app, frame),
        Mode::Exited => panic!("exit mode should never be view()ed"),
    }
}

fn draw_base(app: &App, frame: &mut Frame, solve_space: Rect) {
    const CUBE_HEIGHT: u16 = 11;
    const CUBE_LENGTH: u16 = 13; // ignores back

    let scramble_text = app.scramble();

    let solve_text = app.solution().collect::<Vec<_>>().join("\n");

    let cube_widget = Paragraph::new(
        Cube3x3::from_moves(scramble_text)
            .unwrap_or_else(|_| panic!("scramble {scramble_text} doesn't parse"))
            .colored()
            .to_string()
            .as_bytes()
            .into_text()
            .unwrap_or_else(|e| {
                panic!("original string should've been possible to parse to ratatui::Text: {e}")
            }),
    )
    .block(Block::bordered().title(format!("Solve {}", app.solve_index + 1)));

    let scramble_widget = Paragraph::new(scramble_text).block(Block::bordered().title("Scramble"));

    let solve_widget = Paragraph::new(solve_text).block(Block::bordered().title("Solve"));

    let layout = Layout::vertical([Constraint::Length(CUBE_HEIGHT), Constraint::Fill(1)]);

    let [cube_space, recon_space] = layout.areas(solve_space);

    let [_, cube_area, _] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(CUBE_LENGTH),
        Constraint::Fill(1),
    ])
    .areas(cube_space);

    let [scramble_area, solve_area] =
        Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(recon_space);

    frame.render_widget(cube_widget, cube_area);
    frame.render_widget(scramble_widget, scramble_area);
    frame.render_widget(solve_widget, solve_area);
}

fn walkthrough_recon(app: &App, frame: &mut Frame) {
    let [solve_space, help_space] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(frame.area());

    draw_base(app, frame, solve_space);

    let help_widgets = [
        Line::from("Inspect solve: Enter").centered(),
        Line::from("Next solve: N").centered(),
        Line::from("Previous solve: P").centered(),
        Line::from("Exit: Esc").centered(),
    ];

    let help_areas =
        Layout::horizontal(help_widgets.iter().map(|_| Constraint::Fill(1))).split(help_space);

    for (widget, area) in help_widgets.iter().zip(help_areas.iter()) {
        frame.render_widget(widget, *area);
    }
}

fn inspect_solve(app: &App, frame: &mut Frame) {
    let [solve_space, help_space] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(frame.area());
    draw_base(app, frame, solve_space);

    let help_widgets = [
        Line::from("Next move: N").centered(),
        Line::from("Previous move: P").centered(),
        Line::from("Back to browsing: Esc").centered(),
    ];

    let help_areas =
        Layout::horizontal(help_widgets.iter().map(|_| Constraint::Fill(1))).split(help_space);

    for (widget, area) in help_widgets.iter().zip(help_areas.iter()) {
        frame.render_widget(widget, *area);
    }
}
