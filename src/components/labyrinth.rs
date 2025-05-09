use super::Component;
use crate::{
    action::Action,
    config::Config,
    matcher::Matcher,
    maze::{Maze, MazeCell, PowerUP, VisibleArea},
    settings::Settings,
};
use color_eyre::{eyre::Ok, Result};
use rand::{thread_rng, Rng};
use ratatui::{prelude::*, widgets::*};
use std::{
    cmp::{max, min},
    collections::BTreeSet,
};
use tokio::sync::mpsc::UnboundedSender;

const LOST_MESSAGE: &str = include_str!("../../resources/lost_message.txt");
const WIN_MESSAGE: &str = include_str!("../../resources/win_message.txt");

#[derive(Default)]
pub struct Labyrinth {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    maze: Maze,
    words: Vec<String>,
    player_state: Vec<(usize, PowerUP)>,
    matcher: Matcher,
    visible: VisibleArea,
    score: usize,
    steps: usize,
    notification: (Color, String),
    notif_backup: String,
    lost: bool,
    won: bool,
}

impl Labyrinth {
    pub fn new(settings: Settings) -> Self {
        let mut ans = Self {
            maze: Maze::new(&settings),
            matcher: Matcher::new(settings.words.clone()),
            notification: (Color::Reset, "".to_string()),
            notif_backup:
                "Welcome to the maze:\n use <wasd> or arrows to move and <enter> to confirm move."
                    .to_string(),
            steps: settings.steps,
            words: settings.words,
            ..Default::default()
        };
        ans.update_visual();
        ans
    }

    /// prints the description of selected power up.
    fn show_selected(&mut self) {
        if let Some(power) = self.visible.get_powerup() {
            self.notification.0 = power.color();
            self.notification.1 = power.description().to_string();
        } else {
            self.notification.0 = Color::Reset;
            self.notification.1 = self.notif_backup.clone();
        }
    }

    /// handles the confirmation of a movement.
    fn confirm(&mut self) {
        let x = self.visible.selected.0;
        let y = self.visible.selected.1;
        let selected_cell: &MazeCell = &self.visible.cells[x][y];
        if selected_cell.wall {
            self.notif_backup = "That's wall buddy, You're not that strong.".to_string();
            return;
        }
        let center: usize = self.visible.cells.len() / 2;
        if center.abs_diff(x) > 1 || center.abs_diff(y) > 1 {
            self.notif_backup = "That's too far try something closer.".to_string();
            return;
        }

        // add the power up.
        let x = self.maze.player_location.0 + x - center;
        let y = self.maze.player_location.1 + y - center;
        self.maze.player_location = (x, y);
        if !self.visible.thread.remove(&(x as i32, y as i32)) {
            self.visible.thread = BTreeSet::new();
        }
        self.player_state = self
            .player_state
            .iter()
            .filter(|&(l, _)| *l > 0)
            .map(|&(l, p)| (l - 1, p))
            .collect();

        if let Some(power) = self.maze.cells[x][y].power_up {
            self.apply_power_up(power);
        }

        let current_cell: &mut MazeCell = &mut self.maze.cells[x][y];
        if current_cell.visited {
            self.matcher.reset();
            // check if the player lost.
            self.steps -= 1;
            if self.steps == 0 {
                self.lost = true;
            }
            return;
        }

        // get the score from words.
        current_cell.visited = true;
        let found_words: Vec<String> = self
            .matcher
            .next(&current_cell.value)
            .iter()
            .map(|&ind| self.words[ind].clone())
            .collect();
        if !found_words.is_empty() {
            let draupnir_bonus: usize = 8_usize.pow(
                self.player_state
                    .iter()
                    .filter(|(_, p)| *p == PowerUP::OdinDraupnir)
                    .count() as u32,
            );
            let mut added_score: usize = found_words.iter().map(|s| Self::score(s)).sum();
            added_score *= draupnir_bonus;
            self.notif_backup = "Congrats You found the word: \"".to_string()
                + &found_words.join("\", \"")
                + "\" and that gave you "
                + &added_score.to_string()
                + " steps.";
            self.score += added_score;
            self.steps += added_score;
        }
        if current_cell.exit {
            self.won = true;
            return;
        }

        // check if the player lost.
        self.steps -= 1;
        if self.steps == 0 {
            self.lost = true;
        }
    }

    /// calculates the score based on the length of the word.
    fn score(s: &str) -> usize {
        let l = s.len() + 2;
        l * (l / 3)
    }

    /// regenerate the visible area based on the new position.
    fn update_visual(&mut self) {
        // Calculate sight radius based on Helios Torch power-ups
        let sight_radius = 3 + self
            .player_state
            .iter()
            .filter(|(_, power)| power == &PowerUP::HeliosTorch)
            .count();

        let (x, y) = self.maze.player_location;
        let (height, width) = (self.maze.height, self.maze.width);

        // Calculate visible bounds
        let row_start = x.saturating_sub(sight_radius);
        let row_end = (x + sight_radius).min(height - 1);
        let col_start = y.saturating_sub(sight_radius);
        let col_end = (y + sight_radius).min(width - 1);
        let x_indent = sight_radius.saturating_sub(x);
        let y_indent = sight_radius.saturating_sub(y);

        let dimention = 2 * sight_radius + 1;
        let mut visibility_grid = vec![vec![MazeCell::wall(); dimention]; dimention];

        for i in row_start..=row_end {
            for j in col_start..=col_end {
                // Convert maze coordinates to visibility grid coordinates
                let grid_row = i.saturating_sub(row_start) + x_indent;
                let grid_col = j.saturating_sub(col_start) + y_indent;

                if i < height && j < width {
                    visibility_grid[grid_row][grid_col] = self.maze.cells[i][j].clone();
                }
            }
        }

        self.visible = VisibleArea {
            cells: visibility_grid,
            selected: (dimention / 2, dimention / 2),
            thread: self.visible.thread.clone(),
            offset: (
                x as i32 - sight_radius as i32,
                y as i32 - sight_radius as i32,
            ),
        }
    }

    fn apply_power_up(&mut self, power: PowerUP) {
        match power {
            PowerUP::AriadneThread => self.visible.thread = self.maze.shortest_route().unwrap(),
            PowerUP::ThorMjolnir => {
                let (x, y) = self.maze.player_location;
                let n = self.maze.height;
                let m = self.maze.width;
                for i in x.saturating_sub(3)..=(x + 3) {
                    if i >= n {
                        break;
                    }
                    for j in y.saturating_sub(3)..=(y + 3) {
                        if j >= m {
                            break;
                        }
                        self.maze.cells[i][j].wall = false;
                    }
                }
            }
            PowerUP::BifrostBridge => {
                let mut rng = thread_rng();
                let mut x = rng.gen_range(0..self.maze.height);
                let mut y = rng.gen_range(0..self.maze.width);
                self.maze.player_location = (x, y);
                while self.maze.cells[x][y].exit || self.maze.shortest_route().is_none() {
                    x = rng.gen_range(0..self.maze.height);
                    y = rng.gen_range(0..self.maze.width);
                    self.maze.player_location = (x, y);
                }
                self.maze.cells[x][y].visited = true;
                self.maze.cells[x][y].wall = false;
            }
            // PowerUP::ProteusGift => {},
            _ => self.player_state.push((5, power)),
        }
    }
}

impl Component for Labyrinth {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        let center: usize = self.visible.cells.len() / 2;
        match action {
            Action::GoUp => self.visible.selected.0 = max(self.visible.selected.0 - 1, center - 1),
            Action::GoDown => {
                self.visible.selected.0 = min(self.visible.selected.0 + 1, center + 1)
            }
            Action::GoLeft => {
                self.visible.selected.1 = max(self.visible.selected.1 - 1, center - 1)
            }
            Action::GoRight => {
                self.visible.selected.1 = min(self.visible.selected.1 + 1, center + 1)
            }
            Action::Confirm => {
                self.confirm();
                self.update_visual();
            }
            _ => {}
        }
        self.show_selected();
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        // show the lose screen.
        if self.lost {
            let lost_board = Paragraph::new(LOST_MESSAGE)
                .fg(Color::Red)
                .block(Block::default().borders(Borders::ALL))
                .alignment(Alignment::Center);
            frame.render_widget(lost_board, area);
            return Ok(());
        }

        // show the win screen.
        if self.won {
            let lost_board = Paragraph::new(
                WIN_MESSAGE.to_owned() + "\nyou're score is " + &self.score.to_string(),
            )
            .fg(Color::Green)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
            frame.render_widget(lost_board, area);
            return Ok(());
        }
        let [score, notif, maze] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Fill(1),
        ])
        .areas(area);

        // Render the score and the remaining steps.
        let [score_area, steps_area] =
            Layout::horizontal(Constraint::from_percentages([50, 50])).areas(score);
        let steps_board = match self.steps {
            1 => Paragraph::new("You have one step remaining. Make it count!!")
                .block(Block::default().borders(Borders::ALL))
                .alignment(Alignment::Center)
                .fg(Color::Red),
            _ => Paragraph::new(format!("{} steps remaining.", self.steps))
                .block(Block::default().borders(Borders::ALL))
                .alignment(Alignment::Center),
        };
        frame.render_widget(steps_board, steps_area);
        let score_board = Paragraph::new(format!("Your score is {}", self.score))
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        frame.render_widget(score_board, score_area);

        // Render the notification board.
        let notification_board = Paragraph::new(self.notification.1.clone())
            .fg(self.notification.0)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        frame.render_widget(notification_board, notif);

        // Render maze
        let diamater: u16 = self.visible.cells.len() as u16;
        let [_, center_vert, _] = Layout::vertical([
            Constraint::Fill(3),
            Constraint::Min(diamater),
            Constraint::Fill(3),
        ])
        .areas(maze);
        let [_, center_horizantal, _] = Layout::horizontal([
            Constraint::Fill(3),
            Constraint::Min(diamater * 3),
            Constraint::Fill(3),
        ])
        .areas(center_vert);
        frame.render_widget(Table::from(&self.visible), center_horizantal);
        Ok(())
    }
}
