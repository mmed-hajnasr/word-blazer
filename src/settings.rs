use crate::cli::Cli;
use clap::ValueEnum;
use rand::random;
use tracing::debug;
use std::fs::read_to_string;

const DEFAULT_WORDS: &str = include_str!("../resources/all-words-5200.txt");
const SIMPLE_WORDS: &str = include_str!("../resources/words-simple-1000.txt");
const HARD_WORD: &str = include_str!("../resources/long-words-5000.txt");

pub fn parse_words(contents: &str) -> Vec<String> {
    contents.lines().map(|line| line.to_string()).collect()
}

#[derive(Default)]
pub struct Settings {
    pub height: usize,
    pub width: usize,
    pub words: Vec<String>,
    pub seed: u64,
    pub word_porb: f64,
    pub wall_nodes: usize,
    pub nb_power_ups: usize,
    pub steps: usize,
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
        debug!("the seed is {}",seed);
        match difficulty {
            Difficulty::Easy => Self {
                height: 10,
                width: 10,
                words: parse_words(SIMPLE_WORDS),
                steps: 10,
                seed,
                word_porb: 1.0,
                wall_nodes: 3,
                nb_power_ups: 10,
            },
            Difficulty::Normal => Self {
                height: 50,
                width: 50,
                steps: 20,
                words: parse_words(DEFAULT_WORDS),
                seed,
                word_porb: 1.0,
                wall_nodes: 10,
                nb_power_ups: 40,
            },
            Difficulty::Hard => Self {
                height: 80,
                width: 80,
                steps: 20,
                words: parse_words(HARD_WORD),
                seed,
                word_porb: 0.9,
                wall_nodes: 20,
                nb_power_ups: 50,
            },
        }
    }

    pub fn build(args: Cli) -> Self {
        let mut settings: Settings = Settings::new(args.difficulty.unwrap_or(Difficulty::Normal));
        if let Some(s) = args.steps {
            settings.steps = s;
        }
        if let Some(h) = args.height {
            settings.height = h;
        }
        if let Some(w) = args.width {
            settings.width = w;
        }
        if let Some(seed) = args.seed {
            settings.seed = seed;
        }
        if let Some(path) = args.path {
            settings.words = parse_words(
                &read_to_string(path.clone())
                    .unwrap_or_else(|_| panic!("Failed to read file: {:?}", path)),
            );
        }
        settings
    }
}
