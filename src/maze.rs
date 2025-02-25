use crate::{matcher::Matcher, settings::Settings};
use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};
use ratatui::{prelude::*, widgets::*};
use std::collections::{BTreeSet, VecDeque};

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
const ALPHABET: [char; 26] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
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
    /// get description of the power_up.
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

    /// get the color associated with the power_up.
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

/// represents a single cell in the maze.
#[derive(Default, Clone)]
pub struct MazeCell {
    pub value: char,
    pub power_up: Option<PowerUP>,
    pub wall: bool,
    pub visited: bool,
    pub exit: bool,
}

impl MazeCell {
    /// Returns an empty walled cell.
    pub fn wall() -> Self {
        Self {
            wall: true,
            ..Default::default()
        }
    }
}

/// represents the entire maze using a matrix of cells and player location.
#[derive(Default)]
pub struct Maze {
    pub cells: Vec<Vec<MazeCell>>,
    pub player_location: (usize, usize),
    pub height: usize,
    pub width: usize,
}

impl Maze {
    /// checks if it's possible to go in a certain direction from coordenates.
    ///
    /// # Arguments
    ///
    /// * `coordenates` - the current coordenates of the player.
    /// * `direction` - the index of direction that you want to move in.
    ///
    /// # Returns
    ///
    /// None if it's not possible to go in that direction due to the edge of the maze or a wall.
    /// Some(new_coordinates) the new coordenates after moving in that direction.
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

    /// recursive function that uses rng and dfs to fill the maze using a matcher.
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

    /// recursively creates linear walls in the maze.
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

    /// checks if it's possible to go the end of maze from the player location.
    ///
    /// # Returns
    ///
    /// None if it's not possible to get the end.
    /// Some(path) the set of coordenates that are on the path from the player to the end.
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
        Some(ans)
    }

    /// generate the maze.
    pub fn new(settings: &Settings) -> Self {
        let (n, m): (usize, usize) = (settings.height, settings.width);
        let word_builder: Matcher = Matcher::new(settings.words.clone());
        // generate the rng from the seed.
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

        // generate the walls.
        for _ in 0..settings.wall_nodes {
            let i: usize = rng.gen_range(0..maze.height);
            let j: usize = rng.gen_range(0..maze.width);
            let direction: usize = rng.gen_range(0..8);
            maze.make_wall(i, j, direction, &mut rng);
            maze.make_wall(i, j, (direction + 4) % 8, &mut rng);
        }

        // pick the exit.
        let i: usize = rng.gen_range(0..maze.height);
        let j: usize = rng.gen_range(0..maze.width);
        maze.cells[i][j].wall = false;
        maze.cells[i][j].exit = true;

        // pick the initial player location and check if it has a path to the exit.
        let mut x = rng.gen_range(0..maze.height);
        let mut y = rng.gen_range(0..maze.width);
        maze.player_location = (x, y);
        while {
            let route = maze.shortest_route();
            maze.cells[x][y].exit || route.is_none() || route.unwrap().len() <= (n + m) / 3
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

#[derive(Default)]
pub struct VisibleArea {
    pub cells: Vec<Vec<MazeCell>>,
    pub selected: (usize, usize),
    pub thread: BTreeSet<(i32, i32)>,
    pub offset: (i32, i32),
}

impl VisibleArea {
    /// get the power up in the current location.
    /// if the cell is already visited returns None.
    pub fn get_powerup(&self) -> Option<PowerUP> {
        let (i, j) = self.selected;
        if self.cells[i][j].visited {
            return None;
        }
        self.cells[i][j].power_up
    }
}

impl From<&VisibleArea> for Table<'_> {
    /// Transforms the visible area into an easy to present table.
    fn from(visible: &VisibleArea) -> Self {
        let width: usize = visible.cells[0].len();
        let widths = vec![Constraint::Length(3); width];
        let mid: usize = visible.cells.len() / 2;
        let n: i32 = visible.cells.len() as i32;
        let m: i32 = visible.cells[0].len() as i32;

        let mut cells: Vec<Vec<Cell>> = visible
            .cells
            .iter()
            .map(|row| row.iter().map(Cell::from).collect::<Vec<Cell>>())
            .collect();
        cells[mid][mid] = Cell::new(" ◎ ");
        cells[visible.selected.0][visible.selected.1] = cells[visible.selected.0]
            [visible.selected.1]
            .clone()
            .reversed();

        for &(x, y) in visible.thread.iter() {
            let vx: i32 = x - visible.offset.0;
            let vy: i32 = y - visible.offset.1;
            if vx >= 0 && vx < n && vy >= 0 && vy < m {
                let vx: usize = vx as usize;
                let vy: usize = vy as usize;
                cells[vx][vy] = cells[vx][vy].clone().bg(Color::Yellow)
            }
        }
        let table: Table = cells.iter().map(|row| Row::new(row.clone())).collect();
        table.column_spacing(0).widths(widths)
    }
}

impl From<&MazeCell> for Cell<'_> {
    fn from(cell: &MazeCell) -> Self {
        if cell.exit {
            return Cell::new(" ★ ").fg(Color::Magenta);
        }
        if cell.wall {
            return Cell::new("").bg(Color::White);
        }
        if cell.visited {
            return Cell::new(" ☐ ");
        }
        if let Some(power) = cell.power_up {
            Cell::new(Text::from(cell.value.to_string()).centered()).fg(power.color())
        } else {
            Cell::new(Text::from(cell.value.to_string()).centered())
        }
    }
}
