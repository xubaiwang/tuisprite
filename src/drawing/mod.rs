//! Definition of drawing data.

use csscolorparser::Color;
use serde::{Deserialize, Serialize};

pub mod color;
pub mod io;

#[derive(Serialize, Deserialize)]
pub struct Drawing {
    pub width: usize,
    pub height: usize,
    #[serde(default)]
    pub pixels: Vec<Color>,
}

/// Creation.
impl Drawing {
    /// Create new empty drawing with given size.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![Color::from_rgba8(0, 0, 0, 0); width * height],
        }
    }

    pub fn validate(&mut self) -> bool {
        // XXX: should not modify
        if self.pixels.is_empty() {
            self.pixels = vec![Color::from_rgba8(0, 0, 0, 0); self.width * self.height];
        }
        self.pixels.len() == self.width * self.height
    }
}

impl Default for Drawing {
    fn default() -> Self {
        const DEFAULT_SIZE: usize = 16;

        Self::new(DEFAULT_SIZE, DEFAULT_SIZE)
    }
}

/// Basic ops.
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

    pub fn erase_all(&mut self) {
        self.pixels = vec![Color::from_rgba8(0, 0, 0, 0); self.width * self.height];
    }
}
