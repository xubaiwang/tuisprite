use anyhow::Result;
use std::path::Path;

use crate::drawing::Drawing;

pub async fn load_drawing_from_file(path: &Path) -> Result<Drawing> {
    let text = tokio::fs::read_to_string(path).await?;
    let mut drawing = serde_json::from_str::<Drawing>(&text)?;
    drawing.validate();
    Ok(drawing)
}
