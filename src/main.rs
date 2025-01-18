use app::App;
use clap::Parser;
use cli::Cli;
use color_eyre::Result;
use settings::Settings;

mod action;
mod app;
mod cli;
mod components;
mod config;
mod errors;
mod logging;
mod matcher;
mod maze;
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
