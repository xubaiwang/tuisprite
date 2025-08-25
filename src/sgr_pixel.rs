use ratatui::crossterm::Command;

/// 啟動像素座標模式。
pub struct EnableSgrPixel;

impl Command for EnableSgrPixel {
    fn write_ansi(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        f.write_str("\x1B[?1016h")
    }
}
