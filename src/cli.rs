use std::path::PathBuf;

use clap::Parser;

use crate::config::{get_config_dir, get_data_dir};
use crate::settings::Difficulty;

#[derive(Parser, Debug)]
#[command(author, version = version(), about)]
pub struct Cli {
    /// Difficulty of the game, this will affect the size of maze and the scarcity of the words.
    #[arg(short, long, value_enum)]
    pub difficulty: Option<Difficulty>,

    /// The number of steps given initially to the player.
    #[arg(long)]
    pub steps: Option<usize>,

    /// Height of the maze.
    #[arg(long)]
    pub height: Option<usize>,

    /// Width of the maze.
    #[arg(long)]
    pub width: Option<usize>,

    /// File path for the custom words.
    #[arg(long, short)]
    pub path: Option<PathBuf>,

    /// Seed this will help reproduce mazes.
    #[arg(long)]
    pub seed: Option<u64>,
}

const VERSION_MESSAGE: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "-",
    env!("VERGEN_GIT_DESCRIBE"),
    " (",
    env!("VERGEN_BUILD_DATE"),
    ")"
);

pub fn version() -> String {
    let author = clap::crate_authors!();

    // let current_exe_path = PathBuf::from(clap::crate_name!()).display().to_string();
    let config_dir_path = get_config_dir().display().to_string();
    let data_dir_path = get_data_dir().display().to_string();

    format!(
        "\
{VERSION_MESSAGE}

Authors: {author}

Config directory: {config_dir_path}
Data directory: {data_dir_path}"
    )
}
