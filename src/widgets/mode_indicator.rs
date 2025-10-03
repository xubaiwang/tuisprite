use ratatui::widgets::Widget;

use crate::app::config::{Config, mode::Mode};

pub struct ModeIndicator<'a> {
    config: &'a Config,
}

// impl<'a> Widget for ModeIndicator<'a> {
//     fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
//     where
//         Self: Sized,
//     {
//         let text = match self.config.mode.borrow() {
//             Mode::Normal => "NORMAL",
//             Mode::Command(_) => "COMMAND",
//         };
//         format!(" {} ", text).render(area, buf);
//     }
// }
