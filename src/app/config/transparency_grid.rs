use csscolorparser::Color;

/// How to display transparency background.
pub struct TransparencyGrid {
    /// Width/Height of each grid cell.
    pub size: u16,
    /// Color of darker grid cell.
    pub dark: Color,
    /// Color of lighter grid cell.
    pub light: Color,
}

// NOTE: the aseprite looking
impl Default for TransparencyGrid {
    fn default() -> Self {
        Self {
            size: 8,
            dark: Color::from_rgba8(217, 217, 217, 255),
            light: Color::from_rgba8(240, 240, 240, 255),
        }
    }
}
