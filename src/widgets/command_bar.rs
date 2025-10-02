use ratatui::widgets::Widget;

pub struct CommandBar<'a> {
    command: &'a str,
}

impl<'a> CommandBar<'a> {
    pub fn new(command: Option<&'a str>) -> Self {
        Self {
            command: command.unwrap_or(""),
        }
    }
}

impl<'a> Widget for CommandBar<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        format!(":{}", self.command).render(area, buf);
    }
}
