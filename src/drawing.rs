//! Definition of drawing data.

use csscolorparser::Color;
use serde::{Deserialize, Serialize};

pub trait ColorExt {
    fn to_ratatui(&self) -> ratatui::style::Color;
    fn grayscale(&self) -> u8;
}

impl ColorExt for Color {
    fn to_ratatui(&self) -> ratatui::style::Color {
        let [r, g, b, a] = self.to_rgba8();
        let [bg_r, bg_g, bg_b] = [255, 255, 255];

        let blend = |x, bg_x| {
            let a = a as f32 / 255.;
            (a * x as f32 + (1. - a) * bg_x as f32) as u8
        };

        ratatui::style::Color::Rgb(blend(r, bg_r), blend(g, bg_g), blend(b, bg_b))
    }

    fn grayscale(&self) -> u8 {
        (0.299 * self.r + 0.587 * self.g + 0.114 * self.b) as u8
    }
}

#[derive(Serialize, Deserialize)]
pub struct Drawing {
    pub width: usize,
    pub height: usize,
    #[serde(default)]
    pub pixels: Vec<Color>,
}

impl Drawing {
    pub fn validate(&mut self) -> bool {
        if self.pixels.len() == 0 {
            self.pixels = vec![Color::from_rgba8(0, 0, 0, 0); self.width * self.height];
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

    pub fn resize(&mut self, width: usize, height: usize) {
        let old_width = self.width;
        let old_height = self.height;
        let mut new_pixels = Vec::with_capacity(width * height);

        for y in 0..height {
            for x in 0..width {
                let color = if y < old_height && x < old_width {
                    let old_index = y * old_width + x;
                    self.pixels[old_index].clone()
                } else {
                    Color::from_rgba8(0, 0, 0, 0)
                };
                new_pixels.push(color);
            }
        }

        // update
        self.width = width;
        self.height = height;
        self.pixels = new_pixels;
    }
}

const DEFAULT_SIZE: usize = 8;

impl Default for Drawing {
    fn default() -> Self {
        let pixels = vec![Color::from_rgba8(0, 0, 0, 0); DEFAULT_SIZE * DEFAULT_SIZE];

        Self {
            width: DEFAULT_SIZE,
            height: DEFAULT_SIZE,
            pixels,
        }
    }
}
