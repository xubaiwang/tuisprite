use csscolorparser::Color;
use ratatui::{
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Widget},
};

use crate::{app::Setting, drawing::ColorExt};

pub struct StatusBar<'a> {
    setting: &'a Setting,
}

impl<'a> StatusBar<'a> {
    pub fn new(setting: &'a Setting) -> Self {
        Self { setting }
    }
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        // reversed background color
        Block::new().on_gray().render(area, buf);

        let bg = self.setting.color.to_ratatui();
        let grayscale = self.setting.color.grayscale();
        let fg = if grayscale > 128 {
            Color::from_rgba8(0, 0, 0, 255)
        } else {
            Color::from_rgba8(255, 255, 255, 255)
        }
        .to_ratatui();

        let mut spans = vec![
            Span::raw(" "),
            Span::raw("NORMAL").bold(),
            Span::raw(" "),
            Span::styled(
                format!(" {} ", self.setting.color.to_css_hex()),
                Style::default().bg(bg).fg(fg),
            )
            .bold(),
        ];

        for (idx, color) in self.setting.color_history.iter().rev().enumerate() {
            let grayscale = color.grayscale();
            let fg = if grayscale > 128 {
                Color::from_rgba8(0, 0, 0, 255)
            } else {
                Color::from_rgba8(255, 255, 255, 255)
            }
            .to_ratatui();
            spans.push(Span::styled(
                format!("{}", idx + 1),
                Style::default().bg(color.to_ratatui()).fg(fg),
            ));
        }

        Line::from(spans).black().render(area, buf);
    }
}
