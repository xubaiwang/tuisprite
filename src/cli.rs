use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
pub struct Args {
    // TODO: currently force use a path, may be optional for new file
    /// The file path to load and save.
    pub path: Option<PathBuf>,
}
