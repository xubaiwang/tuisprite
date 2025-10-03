use ratatui::{
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Widget},
};

use crate::{
    app::config::{Config, mode::Mode},
    drawing::color::ColorExt,
};

pub struct StatusBar<'a> {
    config: &'a Config,
}

impl<'a> StatusBar<'a> {
    pub fn new(setting: &'a Config) -> Self {
        Self { config: setting }
    }
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        // bar background color
        Block::new().on_gray().render(area, buf);

        let bg = self.config.color.borrow().to_ratatui([0, 0, 0]);
        let fg = self
            .config
            .color
            .borrow()
            .calculate_fg()
            .to_ratatui([0, 0, 0]);

        let mut spans = vec![
            Span::raw(" "),
            Span::raw(match &*self.config.mode.borrow() {
                Mode::Normal => "NORMAL",
                Mode::Command(_) => "COMMAND",
            })
            .bold(),
            Span::raw(" "),
            Span::styled(
                format!(" {} ", self.config.color.borrow().to_css_hex()),
                Style::default().bg(bg).fg(fg),
            )
            .bold(),
        ];

        for (idx, color) in self.config.color_history.borrow().iter().rev().enumerate() {
            let fg = color.calculate_fg().to_ratatui([0, 0, 0]);
            spans.push(Span::styled(
                format!("{}", to_superscript(idx + 1)),
                Style::default()
                    .bg(color.to_ratatui([0, 0, 0]))
                    .fg(fg)
                    .bold(),
            ));
        }

        Line::from(spans).black().render(area, buf);
    }
}

fn to_superscript(idx: usize) -> char {
    match idx {
        1 => '¹',
        2 => '²',
        3 => '³',
        4 => '⁴',
        5 => '⁵',
        6 => '⁶',
        7 => '⁷',
        8 => '⁸',
        9 => '⁹',
        10 => '⁰',
        _ => panic!("out of superscript"),
    }
}
