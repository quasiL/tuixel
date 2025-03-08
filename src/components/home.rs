use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{
    action::{Action, Module},
    config::Config,
};

#[derive(Default)]
pub struct Home {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    enabled: bool,
}

impl Home {
    pub fn new() -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            enabled: true,
        }
    }
}

impl Component for Home {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if self.enabled {
            match key.code {
                KeyCode::Char('c') => {
                    self.enabled = false;
                    return Ok(Some(Action::ChangeMode(Module::Cron)));
                }
                _ => {}
            }
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::ChangeMode(Module::Home) => {
                self.enabled = true;
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        if self.enabled {
            frame.render_widget(Paragraph::new("home"), area);
        }
        Ok(())
    }
}
