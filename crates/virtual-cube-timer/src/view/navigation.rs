use crate::{
    app::navigation::{NavigationApp, NavigationMode},
    view::{CUBE_HEIGHT, CUBE_LENGTH, SCRAMBLE_HEIGHT, help::HelpWidget},
};

use std::io::stdout;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Offset, Rect},
    text::Line,
    widgets::{Block, Paragraph},
};
use rubiks_cube::{ColorScheme3x3, Cube3x3};
use termprofile::{DetectorSettings, TermProfile};

pub fn render_navigation(app: &NavigationApp, frame: &mut Frame) {
    let scramble = app.scramble().clone();

    let cube = Cube3x3::default() * scramble;

    match app.nav_mode {
        NavigationMode::Solves => view_browse(app, frame, cube),
        NavigationMode::Inspect(_) => view_inspect(app, frame, cube),
    }
}

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

/// Draws the cube and the reconstruction, returning (`cube_area`, `scramble_area`, `recon_area`)
fn draw_base(
    app: &NavigationApp,
    frame: &mut Frame,
    solve_space: Rect,
    cube: Cube3x3,
) -> (Rect, Rect, Rect) {
    let scramble_text = app.scramble().to_string();

    let solve_text = app.solution().to_string();

    let scramble_widget =
        Paragraph::new(scramble_text.clone()).block(Block::bordered().title("Scramble"));

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

    (cube_area, scramble_area, solve_area)
}

fn view_browse(app: &NavigationApp, frame: &mut Frame, cube: Cube3x3) {
    let [app_space, help_space] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(frame.area());

    draw_base(app, frame, app_space, cube);

    let help = HelpWidget::new([
        "Inspect solve: Enter",
        "Next solve: N",
        "Previous solve: P",
        "Exit: Esc",
    ]);

    frame.render_widget(help, help_space);
}

fn view_inspect(app: &NavigationApp, frame: &mut Frame, mut cube: Cube3x3) {
    #[cfg(feature = "debug")]
    let [solve_space, help_space, dbg_area] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(2),
        Constraint::Length(2),
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

    let cursor_position_in_recon = Offset::new(
        i32::try_from(
            app.step()
                .expect("invalid step")
                .to_string()
                .char_indices()
                .filter(|&(_, c)| c == ' ')
                .nth(app.unchecked_inspect_options().move_index)
                .map(|(index, _)| index)
                .expect("move out of range"),
        )
        .expect("overflowed")
        .strict_add(1),
        i32::try_from(app.unchecked_inspect_options().step_index)
            .expect("overflowed")
            .strict_add(1),
    );

    for (step, line) in app.solve_iter().enumerate() {
        if step >= app.unchecked_inspect_options().step_index {
            break;
        }
        cube = cube * line;
    }

    cube = app
        .step()
        .expect("step not found")
        .clone()
        .into_iter()
        .zip(0..cursor_position_in_recon.x)
        .fold(cube, |c, (m, _)| c * m);

    let (_cube_area, _, solve_area) = draw_base(app, frame, solve_space, cube);

    frame.set_cursor_position(solve_area.as_position() + cursor_position_in_recon);

    let help_areas =
        Layout::horizontal(help_widgets.iter().map(|_| Constraint::Fill(1))).split(help_space);

    for (widget, area) in help_widgets.iter().zip(help_areas.iter()) {
        frame.render_widget(widget, *area);
    }

    #[cfg(feature = "debug")]
    {
        let print_info = [
            format!("Step: {}", app.step().unwrap()),
            format!("Step: {}", app.step().unwrap()),
            format!("Total moves: {}", app.cur_step_move_count()),
        ];

        let mut dbg_rows = dbg_area
            .layout_vec(&Layout::new(
                ratatui::layout::Direction::Horizontal,
                (0..print_info.len()).map(|_| Constraint::Fill(1)),
            ))
            .into_iter();

        print_info.into_iter().for_each(|info| {
            frame.render_widget(
                Line::from(info),
                dbg_rows
                    .next()
                    .expect("dbg_rows should have at least 3 rows"),
            )
        });
    }
}
