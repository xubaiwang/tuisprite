use ratatui::{layout::Rect, style::Style, widgets::StatefulWidget};

use crate::{
    app::config::Config,
    drawing::{Drawing, color::ColorExt},
};

const UPPER_HALF_BLOCK: &str = "▀";
const LOWER_HALF_BLOCK: &str = "▄";

pub struct Canvas<'a> {
    config: &'a Config,
    drawing: &'a Drawing,
}

impl<'a> Canvas<'a> {
    pub fn new(config: &'a Config, drawing: &'a Drawing) -> Self {
        Self { config, drawing }
    }
}

impl<'a> StatefulWidget for Canvas<'a> {
    type State = Option<Rect>;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) where
        Self: Sized,
    {
        // pass state back
        *state = Some(area);

        // NOTE: how to render?
        // Iterate over each two row of the drawing and set pixel.
        // Four cases:
        // 1. both have color => upper block
        // 2. only upper has color => upper block
        // 3. only lower has color => lower block
        // 4. none have color => empty
        for r in 0..self.drawing.height.div_ceil(2) {
            for c in 0..self.drawing.width {
                let bg = {
                    let col = c / self.config.transparency_grid.size;
                    let row = 2 * r / self.config.transparency_grid.size;
                    if (col + row).is_multiple_of(2) {
                        self.config.transparency_grid.dark
                    } else {
                        self.config.transparency_grid.light
                    }
                };

                let upper = self.drawing.pixel(c, 2 * r);
                let lower = self.drawing.pixel(c, 2 * r + 1);

                match (upper, lower) {
                    (None, None) => {}
                    (None, Some(lower)) => {
                        buf.set_string(
                            area.x + c as u16,
                            area.y + r as u16,
                            LOWER_HALF_BLOCK,
                            Style::default().fg(lower.to_ratatui(bg)),
                        );
                    }
                    (Some(upper), None) => {
                        buf.set_string(
                            area.x + c as u16,
                            area.y + r as u16,
                            UPPER_HALF_BLOCK,
                            Style::default().fg(upper.to_ratatui(bg)),
                        );
                    }
                    (Some(upper), Some(lower)) => {
                        buf.set_string(
                            area.x + c as u16,
                            area.y + r as u16,
                            UPPER_HALF_BLOCK,
                            Style::default()
                                .fg(upper.to_ratatui(bg))
                                .bg(lower.to_ratatui(bg)),
                        );
                    }
                }
            }
        }
    }
}
