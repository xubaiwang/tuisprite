use std::io;

use anyhow::Result;
use ratatui::crossterm::{
    Command,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
};

/// Mouse tracking SGR Pixel mode (1016).
///
/// See:
///
/// - [XTerm](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Mouse-Tracking)
/// - [Foot](https://man.archlinux.org/man/extra/foot/foot-ctlseqs.7.en#Private_Modes)
/// - [Windows Terminal](https://github.com/microsoft/terminal/issues/18591) (not yet)
pub struct EnableSgrPixel;

impl Command for EnableSgrPixel {
    fn write_ansi(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        f.write_str("\x1B[?1016h")
    }
}

/// Enable mouse and SGR Pixel mode.
pub fn enable_mouse() -> Result<()> {
    execute!(io::stdout(), EnableMouseCapture, EnableSgrPixel)?;
    Ok(())
}

/// Disable mouse and SGR Pixel mode.
pub fn disable_mouse() -> Result<()> {
    execute!(io::stdout(), DisableMouseCapture)?;
    Ok(())
}
