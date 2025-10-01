use ratatui::widgets::Widget;

pub struct CommandBar;

impl Widget for CommandBar {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        ":command bar".render(area, buf);
    }
}
