use anyhow::Result;
use clap::Parser;

use crate::{app::App, cli::Args};

mod app;
mod cli;
mod drawing;
mod utils;
mod widgets;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let app = App::new(args.path)?;

    let mut terminal = ratatui::init();
    app.run(&mut terminal).await?;
    ratatui::restore();

    Ok(())
}
