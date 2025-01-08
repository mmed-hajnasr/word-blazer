use crate::{matcher::Matcher, settings::Settings};
use ratatui::style::Color;

const ALPHABET: [char; 26] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];

#[derive(Clone)]
enum PowerUP {
    AriadneThread,
    HeliosTorch,
    ProteusGift,
    OdinDraupnir,
    ThorMjolnir,
    BifrostBridge,
}

impl PowerUP {
    pub fn description(&self) -> &str {
        match self {
            PowerUP::AriadneThread => "Magical thread that guides you through the maze, as it guided Theseus through the Labyrinth.",
            PowerUP::HeliosTorch => "Illuminates dark areas with the brilliant light of the sun god's torch.",
            PowerUP::ProteusGift => "Transforms into the character you need most, channeling Proteus' shapeshifting abilities.",
            PowerUP::OdinDraupnir => "Multiplies by 8 your score with the power of Odin's self-replicating ring.",
            PowerUP::ThorMjolnir => "Destroys all walls within 3 cells radius, channeling Thor's mighty hammer Mjolnir.",
            PowerUP::BifrostBridge => "Teleports you to a random position in the maze, using the power of the rainbow bridge that connects realms.",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            PowerUP::AriadneThread => Color::Yellow,
            PowerUP::HeliosTorch => Color::Red,
            PowerUP::ProteusGift => Color::Green,
            PowerUP::OdinDraupnir => Color::Magenta,
            PowerUP::ThorMjolnir => Color::Blue,
            PowerUP::BifrostBridge => Color::Cyan,
        }
    }
}

#[derive(Default, Clone)]
struct Cell {
    value: char,
    power_up: Option<PowerUP>,
    wall: bool,
    visited: bool,
    exit: bool,
}

pub struct Maze {
    cells: Vec<Vec<Cell>>,
    player_location: (usize, usize),
}

impl Maze {
    pub fn new(settings: Settings) -> Self {
        let (n, m): (usize, usize) = (settings.height, settings.width);
        let mut cells: Vec<Vec<Cell>> = vec![vec![Cell::default(); m]; n];
        let mut word_builder: Matcher = Matcher::new(settings.words.clone());
        Self {
            cells,
            player_location: (0, 0),
        }
    }
}
