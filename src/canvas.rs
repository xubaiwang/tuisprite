use ratatui::{
    style::{Color, Style},
    widgets::Widget,
};

use crate::drawing::Drawing;

/// 在終端中展示畫作。
pub struct Canvas<'a> {
    drawing: &'a Drawing,
}

impl<'a> Canvas<'a> {
    pub fn new(drawing: &'a Drawing) -> Self {
        Self { drawing }
    }
}

impl<'a> Widget for Canvas<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let drawing = self.drawing;
        let half_height_ceil = drawing.height.div_ceil(2);

        buf.set_style(area, Style::default().bg(Color::Black));

        for i in 0..drawing.width {
            for half_j in 0..half_height_ceil {
                let Some(cell) = buf.cell_mut((i as u16 + area.x, half_j as u16 + area.y)) else {
                    return;
                };

                let top_j = 2 * half_j;
                let bottom_j = top_j + 1;

                let top_color = drawing.pixel(i, top_j).unwrap_or_default();
                let bottom_color = drawing.pixel(i, bottom_j).unwrap_or_default();

                // 分成兩半：
                // - 上下都無 empty
                // - 上下都有 up
                // - 只有上 up
                // - 只有下 down

                match (top_color, bottom_color) {
                    (Color::Reset, Color::Reset) => {
                        cell.set_style(Style::default());
                    }
                    (Color::Reset, c) => {
                        cell.set_char('▄');
                        cell.fg = c;
                    }
                    (c, Color::Reset) => {
                        cell.set_char('▀');
                        cell.fg = c;
                    }
                    (c1, c2) => {
                        cell.set_char('▀');
                        cell.fg = c1;
                        cell.bg = c2;
                    }
                }
            }
        }
    }
}
