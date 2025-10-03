use std::cell::RefCell;

#[derive(Default, Clone, Debug)]
pub enum Mode {
    #[default]
    Normal,
    /// Colon and input command.
    Command(RefCell<String>),
}
