#[derive(Default, Clone, Debug)]

pub enum Mode {
    #[default]
    Normal,
    /// Colon and input command.
    Command(String),
}
