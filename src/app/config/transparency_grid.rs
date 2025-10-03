/// How to display transparency background.
pub struct TransparencyGrid {
    /// Width/Height of each grid cell.
    pub size: usize,
    /// Color of darker grid cell.
    pub dark: [u8; 3],
    /// Color of lighter grid cell.
    pub light: [u8; 3],
}

// NOTE: the aseprite looking
impl Default for TransparencyGrid {
    fn default() -> Self {
        Self {
            size: 8,
            dark: [217, 217, 217],
            light: [240, 240, 240],
        }
    }
}
