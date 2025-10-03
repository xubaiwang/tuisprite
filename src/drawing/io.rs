use anyhow::Result;
use std::{fs, path::Path};

use crate::drawing::Drawing;

pub fn load_drawing_from_file(path: &Path) -> Result<Drawing> {
    let text = fs::read_to_string(path)?;
    let mut drawing = serde_json::from_str::<Drawing>(&text)?;
    drawing.validate();
    Ok(drawing)
}
