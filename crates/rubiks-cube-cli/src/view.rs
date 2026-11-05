use ansi_to_tui::IntoText;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Offset, Rect},
    text::Line,
    widgets::{Block, Paragraph},
};
use rubiks_cube::Cube3x3;

use crate::app::{App, Mode, total_moves_in_cur_step};

const CUBE_HEIGHT: u16 = 11;
const CUBE_LENGTH: u16 = 13; // ignores back
const SCRAMBLE_HEIGHT: u16 = 3;

pub fn view(app: &App, frame: &mut Frame) {
    let scramble_text = app.scramble();

    let cube = Cube3x3::from_moves(scramble_text).expect("scramble should parse");

    match app.mode {
        Mode::BrowseSolves => view_browse(app, frame, cube),
        Mode::Inspect {
            step: _,
            move_in_step: _,
        } => view_inspect(app, frame, cube),
        Mode::Exited => panic!("exit mode should never be view()ed"),
    }
}

fn draw_base(app: &App, frame: &mut Frame, solve_space: Rect, cube: Cube3x3) {
    let scramble_text = app.scramble();

    let solve_text = app.solution().collect::<Vec<_>>().join("\n");

    let cube_widget = Paragraph::new(
        cube.colored()
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
        Layout::vertical([Constraint::Length(SCRAMBLE_HEIGHT), Constraint::Fill(1)])
            .areas(recon_space);

    frame.render_widget(cube_widget, cube_area);
    frame.render_widget(scramble_widget, scramble_area);
    frame.render_widget(solve_widget, solve_area);
}

fn view_browse(app: &App, frame: &mut Frame, cube: Cube3x3) {
    let [solve_space, help_space] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(frame.area());

    draw_base(app, frame, solve_space, cube);

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

fn view_inspect(app: &App, frame: &mut Frame, mut cube: Cube3x3) {
    let [solve_space, help_space, dbg_area] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(2),
        Constraint::Length(4),
    ])
    .areas(frame.area());

    let help_widgets = [
        Line::from("Step navigation: ↑/↓").centered(),
        Line::from("Move navigation: ←/→").centered(),
        Line::from("Back to browsing: Esc").centered(),
    ];

    let [_, recon_space] =
        Layout::vertical([Constraint::Length(CUBE_HEIGHT), Constraint::Fill(1)]).areas(solve_space);

    let [_, solve_area] =
        Layout::vertical([Constraint::Length(SCRAMBLE_HEIGHT), Constraint::Fill(1)])
            .areas(recon_space);

    let cursor_position_in_recon = Offset::new(
        app.step_text()
            .char_indices()
            .filter(|&(_, c)| c == ' ')
            .nth(app.mode.cur_move())
            .map(|(index, _)| index)
            .expect("move out of range") as i32
            + 1,
        app.mode.cur_step() as i32 + 1,
    );

    for (step, line) in app.solution().enumerate() {
        if step >= app.mode.cur_step() {
            break;
        }
        cube = cube.apply_moves(line).expect("should parse");
    }

    cube = cube
        .apply_moves(
            &app.step_text()
                .chars()
                .zip(0..cursor_position_in_recon.x)
                .map(|(c, _)| c)
                .collect::<String>(),
        )
        .expect("should parse");

    draw_base(app, frame, solve_space, cube);

    frame.set_cursor_position(solve_area.as_position() + cursor_position_in_recon);

    let help_areas =
        Layout::horizontal(help_widgets.iter().map(|_| Constraint::Fill(1))).split(help_space);

    for (widget, area) in help_widgets.iter().zip(help_areas.iter()) {
        frame.render_widget(widget, *area);
    }

    let mut dbg_rows = dbg_area.rows();

    frame.render_widget(
        Line::from(format!("Step: {}", app.mode.cur_step())),
        dbg_rows.next().unwrap(),
    );
    frame.render_widget(
        Line::from(format!("Move: {}", app.mode.cur_move())),
        dbg_rows.next().unwrap(),
    );
    frame.render_widget(
        Line::from(format!("Total moves: {}", total_moves_in_cur_step(app))),
        dbg_rows.next().unwrap(),
    );
}
