use ratatui::widgets::{StatefulWidget, Widget};
use unicode_width::UnicodeWidthStr;

use crate::app::config::{Config, mode::Mode};

pub struct CommandBar<'a> {
    config: &'a Config,
    message: Option<&'a str>,
}

impl<'a> CommandBar<'a> {
    pub fn new(config: &'a Config, message: Option<&'a str>) -> Self {
        Self { config, message }
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
        match &self.config.mode {
            Mode::Normal => {
                if let Some(message) = self.message {
                    format!("-- {} --", message).render(area, buf);
                }
            }
            Mode::Command(command) => {
                format!(":{}", command).render(area, buf);
                *state = Some((1 + command.width() as u16, area.y));
            }
        }
    }
}
