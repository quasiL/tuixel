use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    prelude::Frame,
    widgets::{Block, BorderType, Borders},
};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{
    action::{Action, Module},
    config::Config,
    draw::Drawable,
    style::WebserverStyles,
};

#[derive(Default)]
pub struct Settings {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    enabled: bool,
    styles: WebserverStyles,
}

impl Drawable for Settings {}

impl Settings {
    pub fn new() -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            enabled: false,
            styles: WebserverStyles::new(),
        }
    }

    fn draw_info(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::new()
            .borders(Borders::ALL)
            .border_style(self.styles.border_style)
            .style(self.styles.background_style)
            .border_type(BorderType::Thick);

        frame.render_widget(block, area);
    }
}

impl Component for Settings {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::ChangeMode(Module::Settings) = action {
            self.enabled = true;
        }
        if let Action::ChangeMode(Module::Home) = action {
            self.enabled = false;
            return Ok(Some(Action::ClearScreen));
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        if self.enabled {
            let [main_area, footer_area] =
                Layout::vertical([Constraint::Min(1), Constraint::Length(2)]).areas(area);

            self.draw_info(frame, main_area);
            self.draw_footer(frame, footer_area, vec![("<Esc>", "Quit")])?;
        }
        Ok(())
    }
}
