use ratatui::{
    style::Stylize,
    widgets::{Block, Widget},
};

pub struct StatusBar;

impl Widget for StatusBar {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        // reversed background color
        Block::new().reversed().render(area, buf);
        " status bar: 1=red, 2=green, 3=blue, 4=black"
            .reversed()
            .render(area, buf);
    }
}
