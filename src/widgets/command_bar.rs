use ratatui::widgets::{StatefulWidget, Widget};
use unicode_width::UnicodeWidthStr;

use crate::app::config::{Config, mode::Mode};

pub struct CommandBar<'a> {
    config: &'a Config,
}

impl<'a> CommandBar<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }
}

impl<'a> StatefulWidget for CommandBar<'a> {
    type State = Option<(u16, u16)>;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) where
        Self: Sized,
    {
        match &*self.config.mode.borrow() {
            Mode::Normal => {}
            Mode::Command(command) => {
                let command = command.borrow();
                format!(":{}", command).render(area, buf);
                *state = Some((1 + command.width() as u16, area.y));
            }
        }
    }
}
