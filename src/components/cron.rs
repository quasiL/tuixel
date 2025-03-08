pub mod edit;

use color_eyre::Result;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{
    action::{Action, Module},
    config::Config,
};
use edit::Edit;

#[derive(Default)]
pub struct Cron {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    enabled: bool,
}

impl Cron {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Component for Cron {
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
            Action::ChangeMode(Module::Cron) => {
                self.enabled = true;
            }
            Action::ChangeMode(Module::Home) => {
                self.enabled = false;
                return Ok(Some(Action::ClearScreen));
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        if self.enabled {
            frame.render_widget(Paragraph::new("cron"), area);
        }
        Ok(())
    }
}
