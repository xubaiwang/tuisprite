use ratatui::{
    layout::{Margin, Rect},
    widgets::StatefulWidget,
};

use crate::{drawing::Drawing, widgets::canvas::Canvas};

pub struct Workspace<'a> {
    drawing: &'a Drawing,
}

impl<'a> Workspace<'a> {
    pub fn new(drawing: &'a Drawing) -> Self {
        Self { drawing }
    }
}

impl<'a> StatefulWidget for Workspace<'a> {
    type State = Option<Rect>;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) where
        Self: Sized,
    {
        let margin_x = (area.width - self.drawing.width as u16) / 2;
        // NOTE: two drawing cell take one height
        let margin_y = (area.height - self.drawing.height as u16 / 2) / 2;

        let canvas_area = area.inner(Margin::new(margin_x, margin_y));

        Canvas::new(self.drawing).render(canvas_area, buf, state);
    }
}
