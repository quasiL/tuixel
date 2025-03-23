use color_eyre::Result;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::Frame,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
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

    fn draw_settings(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .style(self.styles.background_style)
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(self.styles.border_style);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(inner_area);

        let title = Paragraph::new("Settings")
            .style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, layout[0]);

        let settings_data = vec![
            (
                "Timezone:",
                if self.config.settings.cron.timezone.is_empty() {
                    "Not Set"
                } else {
                    &self.config.settings.cron.timezone
                },
            ),
            (
                "Document Root:",
                if self.config.settings.users.docroot.is_empty() {
                    "Not Set"
                } else {
                    &self.config.settings.users.docroot
                },
            ),
        ];

        let settings_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                std::iter::repeat(Constraint::Length(2))
                    .take(settings_data.len())
                    .collect::<Vec<_>>(),
            )
            .margin(1)
            .split(layout[1]);

        for (i, (title, value)) in settings_data.iter().enumerate() {
            let row = Paragraph::new(format!("{:<20} {}", title, value))
                .style(Style::default().fg(Color::White));

            frame.render_widget(row, settings_layout[i]);
        }
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
            let [settings_area, footer_area] =
                Layout::vertical([Constraint::Min(1), Constraint::Length(2)]).areas(area);

            self.draw_settings(frame, settings_area);
            self.draw_footer(frame, footer_area, vec![("<Esc>", "Quit")])?;
        }
        Ok(())
    }
}
