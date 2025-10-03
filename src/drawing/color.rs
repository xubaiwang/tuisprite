//! Extensions to `csscolorparser::Color`.

use csscolorparser::Color;

pub trait ColorExt {
    fn to_ratatui(&self) -> ratatui::style::Color;
    fn grayscale(&self) -> u8;
    /// Which fg (black or white) to use when use self as background.
    fn calculate_fg(&self) -> Color;
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
        ((0.299 * self.r + 0.587 * self.g + 0.114 * self.b) * 255.) as u8
    }

    fn calculate_fg(&self) -> Color {
        let grayscale = self.grayscale();
        if grayscale > 128 {
            Color::from_rgba8(0, 0, 0, 255)
        } else {
            Color::from_rgba8(255, 255, 255, 255)
        }
    }
}

#[cfg(test)]
mod test {
    use csscolorparser::Color;

    use super::*;

    #[test]
    fn test_grayscale() {
        let white = Color::new(1., 1., 1., 1.);
        let grayscale = white.grayscale();
        assert_eq!(grayscale, 255);
    }
}
