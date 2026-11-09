use ratatui::Frame;

use crate::app::untimed::UntimedApp;

pub fn render_untimed(app: &UntimedApp, frame: &mut Frame) {
    frame.render_widget(app.cube.widget(), frame.area());
}
