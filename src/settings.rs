use crate::cli::Cli;
use clap::ValueEnum;
use std::fs::read_to_string;
use rand::random;

const DEFAULT_WORDS: &str = include_str!("../resources/words-simple-1000.txt");

pub fn parse_words(contents: &str) -> Vec<String> {
    contents.lines().map(|line| line.to_string()).collect()
}

#[derive(Default)]
pub struct Settings {
    pub difficulty: Difficulty,
    pub height: usize,
    pub width: usize,
    pub words: Vec<String>,
    pub seed: u64,
}

#[derive(Default, Debug, Clone, ValueEnum)]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
}

impl Settings {
    fn new(difficulty: Difficulty) -> Self {
        let seed: u64 = random();
        match difficulty {
            Difficulty::Easy => Self {
                difficulty,
                height: 30,
                width: 30,
                words: parse_words(DEFAULT_WORDS),
                seed,
            },
            Difficulty::Normal => Self {
                difficulty,
                height: 50,
                width: 50,
                words: parse_words(DEFAULT_WORDS),
                seed,
            },
            Difficulty::Hard => Self {
                difficulty,
                height: 80,
                width: 80,
                words: parse_words(DEFAULT_WORDS),
                seed,
            },
        }
    }

    pub fn build(args: Cli) -> Self {
        let mut settings: Settings = Settings::new(args.difficulty.unwrap_or(Difficulty::Normal));
        if let Some(h) = args.height {
            settings.height = h;
        }
        if let Some(w) = args.width {
            settings.width = w;
        }
        if let Some(seed) = args.seed {
            settings.seed = seed;
        }
        if let Some(path) = args.words_path {
            settings.words = parse_words(
                &read_to_string(path.clone()).unwrap_or_else(|_| panic!("Failed to read file: {:?}", path)),
            );
        }
        settings
    }
}
