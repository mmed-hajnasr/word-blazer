use clap::Parser;
use cli::Cli;
use color_eyre::Result;
use settings::Settings;

use crate::app::App;

mod action;
mod maze;
mod app;
mod cli;
mod components;
mod config;
mod errors;
mod logging;
mod matcher;
mod settings;
mod tui;

#[tokio::main]
async fn main() -> Result<()> {
    crate::errors::init()?;
    crate::logging::init()?;

    let args = Cli::parse();
    let mut app = App::new(Settings::build(args))?;
    app.run().await?;
    Ok(())
}
