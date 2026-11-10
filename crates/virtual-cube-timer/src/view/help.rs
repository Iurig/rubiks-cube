use ratatui::{
    layout::{Constraint, Layout},
    text::Line,
    widgets::Widget,
};

pub struct HelpWidget {
    messages: Vec<String>,
}

impl Widget for HelpWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let help_areas =
            Layout::horizontal(self.messages.iter().map(|_| Constraint::Fill(1))).split(area);

        for (message, area) in self.messages.into_iter().zip(help_areas.into_iter()) {
            Line::from(message.as_str()).centered().render(*area, buf);
        }
    }
}

impl HelpWidget {
    pub fn new<'a>(messages: impl IntoIterator<Item = &'a str>) -> Self {
        Self {
            messages: messages.into_iter().map(|s| s.to_string()).collect(),
        }
    }
}
