use ratatui::{
    Frame,
    layout::{Constraint, HorizontalAlignment, Layout},
};
use tui_big_text::{BigText, PixelSize};

use crate::{
    app::timing::TimingApp,
    view::{CUBE_HEIGHT, help::HelpWidget},
};

pub fn render_timing(app: &TimingApp, frame: &mut Frame) {
    let [_, cube_area, timer_area, _, help_area] = frame.area().layout(&Layout::new(
        ratatui::layout::Direction::Vertical,
        [
            Constraint::Fill(1),
            Constraint::Length(CUBE_HEIGHT),
            Constraint::Length(5),
            Constraint::Fill(1),
            Constraint::Length(1),
        ],
    ));

    let help = HelpWidget::new(["Timer Start/Stop: Move/Solve the Cube", "Exit: Esc"]);

    frame.render_widget(app.cube.widget(), cube_area);
    frame.render_widget(
        BigText::builder()
            .pixel_size(PixelSize::Quadrant)
            .lines(vec![
                format!(
                    "{:02?}.{:02?}",
                    app.timer_number.as_secs(),
                    (app.timer_number.as_millis() / 10) % 100
                )
                .into(),
            ])
            .alignment(HorizontalAlignment::Center)
            .build(),
        timer_area,
    );
    frame.render_widget(help, help_area);
}
