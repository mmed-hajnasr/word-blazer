use crate::{matcher::Matcher, settings::Settings};
use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};
use ratatui::style::Color;
use std::collections::{BTreeSet, VecDeque};

const ALPHABET: [char; 26] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];
const DIRECTIONS: [(i32, i32); 8] = [
    (-1, 1),
    (0, 1),
    (1, 1),
    (1, 0),
    (1, -1),
    (0, -1),
    (-1, -1),
    (-1, 0),
];
const POWERUPS: [PowerUP; 5] = [
    PowerUP::AriadneThread,
    PowerUP::HeliosTorch,
    // PowerUP::ProteusGift,
    PowerUP::OdinDraupnir,
    PowerUP::ThorMjolnir,
    PowerUP::BifrostBridge,
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PowerUP {
    AriadneThread,
    HeliosTorch,
    // ProteusGift,
    OdinDraupnir,
    ThorMjolnir,
    BifrostBridge,
}

impl PowerUP {
    pub fn description(&self) -> &str {
        match self {
            PowerUP::AriadneThread => "Ariadne's thread : Magical thread that guides you through the maze, as it guided Theseus through the Labyrinth.",
            PowerUP::HeliosTorch => "The torch of helios : Illuminates dark areas with the brilliant light of the sun god's torch.",
            // PowerUP::ProteusGift => "Proteus's Gift : Transforms into the character you need most, channeling Proteus' shapeshifting abilities.",
            PowerUP::OdinDraupnir => "Draupnir : Multiplies by 8 your score with the power of Odin's self-replicating ring.",
            PowerUP::ThorMjolnir => "Thor's hammer : Destroys all walls within 3 cells radius, channeling Thor's mighty hammer Mjolnir.",
            PowerUP::BifrostBridge => "The BifrostBridge : Teleports you to a random position in the maze, using the power of the rainbow bridge that connects realms.",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            PowerUP::AriadneThread => Color::Yellow,
            PowerUP::HeliosTorch => Color::Red,
            // PowerUP::ProteusGift => Color::Green,
            PowerUP::OdinDraupnir => Color::Green,
            PowerUP::ThorMjolnir => Color::Blue,
            PowerUP::BifrostBridge => Color::Magenta,
        }
    }
}

#[derive(Default, Clone)]
pub struct MazeCell {
    pub value: char,
    pub power_up: Option<PowerUP>,
    pub wall: bool,
    pub visited: bool,
    pub exit: bool,
}

impl MazeCell {
    /// use the new() function to get walled cell.
    pub fn wall() -> Self {
        Self {
            wall: true,
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct Maze {
    pub cells: Vec<Vec<MazeCell>>,
    pub player_location: (usize, usize),
    pub height: usize,
    pub width: usize,
}

impl Maze {
    fn valid_coordenates(
        &self,
        coordenates: (usize, usize),
        direction: usize,
    ) -> Option<(usize, usize)> {
        let (di, dj) = DIRECTIONS[direction];
        let new_i = coordenates.0 as i32 + di;
        let new_j = coordenates.1 as i32 + dj;
        if new_i < 0 || new_j < 0 {
            return None;
        }
        let i = new_i as usize;
        let j = new_j as usize;
        if i < self.height && j < self.width && !self.cells[i][j].wall {
            return Some((new_i as usize, new_j as usize));
        }
        None
    }

    fn fill_maze_characters(
        &mut self,
        coordenates: (usize, usize),
        mut state: usize,
        word_prob: f64,
        word_builder: &Matcher,
        rng: &mut StdRng,
    ) {
        let mut shuffled_directions: Vec<usize> = (0..8).collect();
        shuffled_directions.shuffle(rng);
        if word_builder.options(state).is_empty() {
            state = 0
        }

        for &direction in shuffled_directions.iter() {
            if let Some(next) = self.valid_coordenates(coordenates, direction) {
                if self.cells[next.0][next.1].value != char::default() {
                    continue;
                }
                let next_char: char = if rng.gen::<f64>() < word_prob {
                    *word_builder
                        .options(state)
                        .choose(rng)
                        .unwrap_or(ALPHABET.choose(rng).unwrap())
                } else {
                    *ALPHABET.choose(rng).unwrap()
                };
                self.cells[next.0][next.1].value = next_char;
                self.fill_maze_characters(
                    next,
                    word_builder.next_state(state, next_char),
                    word_prob,
                    word_builder,
                    rng,
                );
            };
        }
    }

    fn make_wall(&mut self, i: usize, j: usize, mut direction: usize, rng: &mut StdRng) {
        self.cells[i][j].wall = true;
        if rng.gen::<f64>() < 0.5 {
            if rng.gen::<f64>() < 0.5 {
                direction += 7;
            } else {
                direction += 1;
            }
            direction %= 8;
        }
        if let Some((new_i, new_j)) = self.valid_coordenates((i, j), direction) {
            self.make_wall(new_i, new_j, direction, rng);
        }
    }

    pub fn shortest_route(&self) -> Option<BTreeSet<(i32, i32)>> {
        // we are using bfs to find the shortest path.
        let mut vis: Vec<Vec<bool>> = vec![vec![false; self.width]; self.height];
        let mut next_direction: Vec<Vec<Option<usize>>> = vec![vec![None; self.width]; self.height];
        let mut stack: VecDeque<(usize, usize)> =
            VecDeque::from([(self.player_location.0, self.player_location.1)]);
        let mut exit: Option<(usize, usize)> = None;
        while let Some((x, y)) = stack.pop_front() {
            if vis[x][y] {
                continue;
            }
            vis[x][y] = true;
            if self.cells[x][y].exit {
                exit = Some((x, y));
                break;
            }
            for d in 0..8 {
                if let Some((next_x, next_y)) = self.valid_coordenates((x, y), d) {
                    if vis[next_x][next_y] || next_direction[next_x][next_y].is_some() {
                        continue;
                    }
                    next_direction[next_x][next_y] = Some((d + 4) % 8);
                    stack.push_back((next_x, next_y));
                }
            }
        }
        let (mut x, mut y) = exit?;
        let mut ans: BTreeSet<(i32, i32)> = BTreeSet::from([(x as i32, y as i32)]);
        while let Some(d) = next_direction[x][y] {
            (x, y) = self.valid_coordenates((x, y), d)?;
            ans.insert((x as i32, y as i32));
        }
        ans.remove(&(self.player_location.0 as i32, self.player_location.1 as i32));
        // debug!(
        //     "{} : {}",
        //     ans.iter()
        //         .map(|(x, y)| format!("({},{})", x, y))
        //         .collect::<Vec<String>>()
        //         .join(" -> "),
        //     ans.len()
        // );
        Some(ans)
    }

    pub fn new(settings: &Settings) -> Self {
        let (n, m): (usize, usize) = (settings.height, settings.width);
        let word_builder: Matcher = Matcher::new(settings.words.clone());
        let mut rng = StdRng::seed_from_u64(settings.seed);

        // first we fill the characters of the maze using dfs and rand.
        let mut maze = Self {
            height: n,
            width: m,
            cells: vec![vec![MazeCell::default(); m]; n],
            player_location: (0, 0),
        };
        maze.fill_maze_characters(
            (n / 2, m / 2),
            0,
            settings.word_porb,
            &word_builder,
            &mut rng,
        );

        for _ in 0..settings.wall_nodes {
            let i: usize = rng.gen_range(0..maze.height);
            let j: usize = rng.gen_range(0..maze.width);
            let direction: usize = rng.gen_range(0..8);
            maze.make_wall(i, j, direction, &mut rng);
            maze.make_wall(i, j, (direction + 4) % 8, &mut rng);
        }

        let i: usize = rng.gen_range(0..maze.height);
        let j: usize = rng.gen_range(0..maze.width);
        maze.cells[i][j].wall = false;
        maze.cells[i][j].exit = true;

        let mut x = rng.gen_range(0..maze.height);
        let mut y = rng.gen_range(0..maze.width);
        maze.player_location = (x, y);
        while {
            let route = maze.shortest_route();
            maze.cells[x][y].exit || route.is_none() || route.unwrap().len() > (n + m) / 3
        } {
            x = rng.gen_range(0..maze.height);
            y = rng.gen_range(0..maze.width);
            maze.player_location = (x, y);
        }
        maze.cells[x][y].visited = true;
        maze.cells[x][y].wall = false;

        for _ in 0..settings.nb_power_ups {
            let mut to_up: (usize, usize) =
                (rng.gen_range(0..maze.height), rng.gen_range(0..maze.width));
            while maze.cells[to_up.0][to_up.1].wall
                || maze.cells[to_up.0][to_up.1].exit
                || maze.cells[to_up.0][to_up.1].power_up.is_some()
                || to_up == maze.player_location
            {
                to_up = (rng.gen_range(0..maze.height), rng.gen_range(0..maze.width));
            }
            maze.cells[to_up.0][to_up.1].power_up = Some(*POWERUPS.choose(&mut rng).unwrap());
        }
        maze
    }
}
