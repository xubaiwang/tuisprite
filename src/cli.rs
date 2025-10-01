use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(arg_required_else_help = true)]
pub struct Args {
    // TODO: currently force use a path, may be optional for new file
    /// The file path to load and save.
    pub path: PathBuf,
}
