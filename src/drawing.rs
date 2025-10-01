//! Definition of drawing data.

use serde::{Deserialize, Serialize};

/// Color is defined as RGBA color tuple.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Color([u8; 4]);

impl Color {
    pub const RESET: Color = Color([0; 4]);
    pub const RED: Color = Color([255, 0, 0, 255]);
    pub const GREEN: Color = Color([0, 255, 0, 255]);
    pub const BLUE: Color = Color([0, 0, 255, 255]);
    pub const BLACK: Color = Color([0, 0, 0, 255]);

    pub fn to_ratatui(&self, bg: [u8; 3]) -> ratatui::style::Color {
        let [r, g, b, a] = self.0;
        let [bg_r, bg_g, bg_b] = bg;

        let blend = |x, bg_x| {
            let a = a as f32 / 255.;
            (a * x as f32 + (1. - a) * bg_x as f32) as u8
        };

        ratatui::style::Color::Rgb(blend(r, bg_r), blend(g, bg_g), blend(b, bg_b))
    }
}

#[derive(Serialize, Deserialize)]
pub struct Drawing {
    pub width: usize,
    pub height: usize,
    pub background: [u8; 3],
    #[serde(default)]
    pub pixels: Vec<Color>,
}

impl Drawing {
    pub fn validate(&mut self) -> bool {
        if self.pixels.len() == 0 {
            self.pixels = vec![Color::RESET; self.width * self.height];
        }
        self.pixels.len() == self.width * self.height
    }
}

impl Drawing {
    pub fn pixel(&self, x: usize, y: usize) -> Option<&Color> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = y * self.width + x;
        self.pixels.get(index)
    }

    pub fn pixel_mut(&mut self, x: usize, y: usize) -> Option<&mut Color> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = y * self.width + x;
        self.pixels.get_mut(index)
    }
}

const DEFAULT_SIZE: usize = 8;

impl Default for Drawing {
    fn default() -> Self {
        let pixels = vec![Color::RESET; DEFAULT_SIZE * DEFAULT_SIZE];

        Self {
            background: [0, 0, 0],
            width: DEFAULT_SIZE,
            height: DEFAULT_SIZE,
            pixels,
        }
    }
}
