use ratatui::widgets::Widget;

pub struct CommandBar<'a> {
    command: Option<&'a str>,
}

impl<'a> CommandBar<'a> {
    pub fn new(command: Option<&'a str>) -> Self {
        Self { command }
    }
}

impl<'a> Widget for CommandBar<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if let Some(command) = self.command {
            format!(":{}", command).render(area, buf);
        }
    }
}
