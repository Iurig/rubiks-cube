use std::io::stdout;

use ratatui::Frame;
use termprofile::{DetectorSettings, TermProfile};

use crate::app::App;

pub fn render_view(app: &App, frame: &mut Frame) {
    frame.render_widget(
        *app.cube
            .widget()
            .on_terminal(TermProfile::detect(&stdout(), DetectorSettings::default())),
        frame.area(),
    );
}
