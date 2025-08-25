use ratatui::style::Color;

pub struct Drawing {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Color>,
}

impl Drawing {
    pub fn pixel(&self, x: usize, y: usize) -> Option<Color> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = y * self.width + x;
        self.pixels.get(index).copied()
    }

    pub fn pixel_mut(&mut self, x: usize, y: usize) -> Option<&mut Color> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = y * self.width + x;
        self.pixels.get_mut(index)
    }
}

impl Default for Drawing {
    fn default() -> Self {
        let mut pixels = vec![Color::Reset; 64];
        pixels[0] = Color::Red;
        pixels[1] = Color::Green;
        pixels[2] = Color::Blue;

        Self {
            width: 8,
            height: 8,
            pixels,
        }
    }
}
