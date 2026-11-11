use std::io::stdout;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Offset, Rect},
    text::Line,
    widgets::{Block, Paragraph},
};
use rubiks_cube::{ColorScheme3x3, Cube3x3};
use termprofile::{DetectorSettings, TermProfile};

use crate::app::{App, Mode};

const CUBE_HEIGHT: u16 = 23;
const CUBE_LENGTH: u16 = 43; // ignores back
const SCRAMBLE_HEIGHT: u16 = 3;

pub fn draw_cli_widget(solve_number: usize, cube: &Cube3x3, area: Rect, frame: &mut Frame) {
    let border = Block::bordered().title(format!("Solve {}", solve_number + 1));

    frame.render_widget(border.clone(), area);

    let inner_area = border.inner(area);

    let colored = cube.widget().with_colorscheme(
        ColorScheme3x3::STANDARD
            .on_terminal(TermProfile::detect(&stdout(), DetectorSettings::default())),
    );

    frame.render_widget(colored, inner_area);
}

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

    let scramble_widget = Paragraph::new(scramble_text).block(Block::bordered().title("Scramble"));

    let solve_widget = Paragraph::new(solve_text).block(Block::bordered().title("Solve"));

    let layout = if (solve_space.height as usize)
        < CUBE_HEIGHT as usize
            + 4
            + scramble_text.lines().count()
            + solve_widget.line_count(solve_space.width - 2)
    {
        Layout::horizontal([Constraint::Length(CUBE_LENGTH), Constraint::Fill(1)])
    } else {
        Layout::vertical([Constraint::Length(CUBE_HEIGHT), Constraint::Fill(1)])
    };

    let [cube_space, recon_space] = layout.areas(solve_space);

    let cube_area = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(CUBE_LENGTH),
        Constraint::Fill(1),
    ])
    .areas::<3>(cube_space)
    .iter()
    .map(|rec| rec.centered_vertically(Constraint::Length(CUBE_HEIGHT)))
    .nth(1)
    .expect("layout should have 3 rectangles, but has no 1 indexed rectangle");

    draw_cli_widget(app.solve_index, &cube, cube_area, frame);

    let [scramble_area, solve_area] =
        Layout::vertical([Constraint::Length(SCRAMBLE_HEIGHT), Constraint::Fill(1)])
            .areas(recon_space);

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
    #[cfg(feature = "debug")]
    let [solve_space, help_space, dbg_area] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(2),
        Constraint::Length(4),
    ])
    .areas(frame.area());

    #[cfg(not(feature = "debug"))]
    let [solve_space, help_space] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(frame.area());

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
        i32::try_from(
            app.step_text()
                .char_indices()
                .filter(|&(_, c)| c == ' ')
                .nth(app.mode.cur_move())
                .map(|(index, _)| index)
                .expect("move out of range"),
        )
        .expect("overflowed")
        .strict_add(1),
        i32::try_from(app.mode.cur_step())
            .expect("overflowed")
            .strict_add(1),
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

    #[cfg(feature = "debug")]
    {
        let mut dbg_rows = dbg_area.rows();

        frame.render_widget(
            Line::from(format!("Step: {}", app.mode.cur_step())),
            dbg_rows
                .next()
                .expect("dbg_rows should have at least 3 rows"),
        );
        frame.render_widget(
            Line::from(format!("Move: {}", app.mode.cur_move())),
            dbg_rows
                .next()
                .expect("dbg_rows should have at least 3 rows"),
        );
        frame.render_widget(
            Line::from(format!(
                "Total moves: {}",
                crate::app::total_moves_in_cur_step(app)
            )),
            dbg_rows
                .next()
                .expect("dbg_rows should have at least 3 rows"),
        );
    }
}
