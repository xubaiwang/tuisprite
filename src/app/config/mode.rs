use std::{cell::RefCell, rc::Rc};

#[derive(Default, Clone, Debug)]
pub enum Mode {
    #[default]
    Normal,
    /// Colon and input command.
    Command(Rc<RefCell<String>>),
}
