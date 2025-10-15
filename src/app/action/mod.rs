use std::path::PathBuf;

use csscolorparser::Color;
use either::Either;

#[derive(Debug, Clone)]
pub enum Action {
    /// Quit the application
    Quit,
    Save(Option<PathBuf>),
    EnterCommandMode,
    EnterNormalMode,
    CommandPush(char),
    CommandPop,
    /// Resize the drawing.
    Resize(usize, usize),
    /// Erase the drawing.
    Erase,
    SetColor(Either<Color, u8>),
    /// Execute JavaScript.
    Execute(String),
}
