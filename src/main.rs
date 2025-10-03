use anyhow::Result;
use clap::Parser;

use crate::{app::App, cli::Args};

mod app;
mod cli;
mod drawing;
mod utils;
mod widgets;

fn main() -> Result<()> {
    let args = Args::parse();
    let app = App::new(args.path)?;
    ratatui::run(|terminal| app.run(terminal))?;
    Ok(())
}
