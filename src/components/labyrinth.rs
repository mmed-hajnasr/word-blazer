use std::{cell, collections::BTreeSet};

use super::Component;
use crate::{
    action::Action,
    config::Config,
    matcher::Matcher,
    maze::{Maze, MazeCell, PowerUP},
    settings::Settings,
};
use color_eyre::{eyre::Ok, Result};
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

const LOST_MESSAGE: &str = include_str!("../../resources/lost_message.txt");

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
}

impl Labyrinth {
    pub fn new(settings: Settings) -> Self {
        let mut ans = Self {
            maze: Maze::new(&settings),
            matcher: Matcher::new(settings.words.clone()),
            notification: (Color::Reset, "".to_string()),
            notif_backup: "Welcome to the maze".to_string(),
            steps: settings.steps,
            words: settings.words,
            ..Default::default()
        };
        ans.update_visual();
        ans
    }

    fn show_selected(&mut self) {
        if let Some(power) = self.visible.get_powerup() {
            self.notification.0 = power.color();
            self.notification.1 = power.description().to_string();
        } else {
            self.notification.0 = Color::Reset;
            self.notification.1 = self.notif_backup.clone();
        }
    }

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
        let current_cell: &mut MazeCell = &mut self.maze.cells[x][y];
        if current_cell.visited {
            // check if the player lost.
            self.steps -= 1;
            if self.steps == 0 {
                self.lost = true;
            }
            return;
        }
        if let Some(power) = current_cell.power_up {
            self.player_state.push((5, power));
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
            let mut added_score: usize = found_words.iter().map(|s| s.len() * s.len()).sum();
            added_score *= draupnir_bonus;
            self.notif_backup = "Congrats You found the word: \"".to_string()
                + &found_words.join("\", \"")
                + "\" and that gave you "
                + &added_score.to_string()
                + " steps.";
            self.score += added_score;
            self.steps += added_score;
        }

        // check if the player lost.
        self.steps -= 1;
        if self.steps == 0 {
            self.lost = true;
        }
    }

    fn update_visual(&mut self) {
        // Calculate sight radius based on Helios Torch power-ups
        let sight_radius = 3 + self
            .player_state
            .iter()
            .filter(|(_, power)| power == &PowerUP::HeliosTorch)
            .count();

        let (player_row, player_col) = self.maze.player_location;
        let (height, width) = (self.maze.height, self.maze.width);

        // Calculate visible bounds
        let row_start = player_row.saturating_sub(sight_radius);
        let row_end = (player_row + sight_radius).min(height - 1);
        let col_start = player_col.saturating_sub(sight_radius);
        let col_end = (player_col + sight_radius).min(width - 1);
        let x_indent = sight_radius.saturating_sub(player_row);
        let y_indent = sight_radius.saturating_sub(player_col);

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
            thread: vec![],
            offset: (
                player_row as i32 - sight_radius as i32,
                player_col as i32 - sight_radius as i32,
            ),
        }
    }
}

impl From<&VisibleArea> for Table<'_> {
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
            let vy: i32 = y - visible.offset.0;
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
        match action {
            Action::GoUp => {
                self.visible.selected.0 = self.visible.selected.0.saturating_sub(1);
            }
            Action::GoDown => {
                self.visible.selected.0 =
                    (self.visible.selected.0 + 1).min(self.visible.cells.len() - 1);
            }
            Action::GoLeft => {
                self.visible.selected.1 = self.visible.selected.1.saturating_sub(1);
            }
            Action::GoRight => {
                self.visible.selected.1 =
                    (self.visible.selected.1 + 1).min(self.visible.cells[0].len() - 1);
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
        if self.lost {
            let lost_board = Paragraph::new(LOST_MESSAGE)
                .fg(Color::Red)
                .block(Block::default().borders(Borders::ALL))
                .alignment(Alignment::Center);
            frame.render_widget(lost_board, area);
            return Ok(());
        }
        let [score, notif, maze] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
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

        // Render second bordered text
        let notification_board = Paragraph::new(self.notification.1.clone())
            .fg(self.notification.0)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        frame.render_widget(notification_board, notif);

        // Render table
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

#[derive(Default)]
struct VisibleArea {
    cells: Vec<Vec<MazeCell>>,
    selected: (usize, usize),
    thread: Vec<(i32, i32)>,
    offset: (i32, i32),
}

impl VisibleArea {
    fn get_powerup(&self) -> Option<PowerUP> {
        let (i, j) = self.selected;
        if !self.cells[i][j].visited {
            return None;
        }
        self.cells[i][j].power_up
    }
}
