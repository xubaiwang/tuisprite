use std::collections::VecDeque;

use csscolorparser::Color;

use crate::app::config::{mode::Mode, transparency_grid::TransparencyGrid};

pub mod mode;
pub mod transparency_grid;

/// App runtime config.
pub struct Config {
    /// Current color.
    pub color: Color,
    /// Previously used colors.
    pub color_history: VecDeque<Color>,
    pub transparency_grid: TransparencyGrid,
    pub mode: Mode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            color: Color::from_rgba8(0, 0, 0, 255),
            color_history: {
                let mut v = VecDeque::new();
                v.push_back(Color::from_rgba8(255, 255, 255, 255));
                v.push_back(Color::from_rgba8(255, 0, 0, 255));
                v.push_back(Color::from_rgba8(0, 255, 0, 255));
                v.push_back(Color::from_rgba8(0, 0, 255, 255));
                v.push_back(Color::from_rgba8(0, 255, 255, 255));
                v.push_back(Color::from_rgba8(255, 255, 0, 255));
                v
            },
            transparency_grid: Default::default(),
            mode: Default::default(),
        }
    }
}

impl Config {
    pub fn set_color(&mut self, color: Color) {
        let color_history = &mut self.color_history;

        let old_color = std::mem::replace(&mut self.color, color);
        if !color_history.contains(&old_color) {
            color_history.push_back(old_color);
        }
        if color_history.len() > 10 {
            color_history.pop_front();
        }
    }
}
