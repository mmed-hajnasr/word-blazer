use super::Component;
use crate::{
    action::Action,
    config::Config,
    matcher::Matcher,
    maze::{Maze, MazeCell, PowerUP},
    settings::Settings,
};
use color_eyre::Result;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Default)]
pub struct Labyrinth {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    settings: Settings,
    maze: Maze,
    player_state: Vec<(usize, PowerUP)>,
    matcher: Matcher,
}

impl Labyrinth {
    pub fn new(settings: Settings) -> Self {
        Self {
            maze: Maze::new(&settings),
            matcher: Matcher::new(settings.words.clone()),
            settings,
            ..Default::default()
        }
    }
}

impl From<&MazeCell> for Cell<'_> {
    fn from(cell: &MazeCell) -> Self {
        if cell.exit {
            return Cell::new("★").fg(Color::Magenta);
        }
        if cell.wall {
            return Cell::new("").bg(Color::White);
        }
        if cell.visited {
            return Cell::new("☐").fg(Color::White);
        }
        let cell_color = if let Some(power) = cell.power_up {
            power.color()
        } else {
            Color::White
        };
        Cell::new(cell.value.to_string()).fg(cell_color)
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
            Action::Tick => {
                // add any logic here that should run on every tick
            }
            Action::Render => {
                // add any logic here that should run on every render
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let table: Table = self
            .maze
            .cells
            .iter()
            .map(|row| row.iter().map(Cell::from).collect::<Row>())
            .collect();
        frame.render_widget(table, area);
        Ok(())
    }
}
