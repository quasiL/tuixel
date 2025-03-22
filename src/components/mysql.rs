use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Frame,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};
use std::process::Command;
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{
    action::{Action, Module},
    config::Config,
    draw::Drawable,
    style::WebserverStyles,
};

#[derive(Default)]
pub struct MySql {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    enabled: bool,
    styles: WebserverStyles,
    mysql_uptime: Option<String>,
    mysql_status: Option<String>,
}

impl Drawable for MySql {}

impl MySql {
    pub fn new() -> Self {
        let mysql_installed = Self::is_mysql_installed();
        let mysql_uptime = if mysql_installed {
            Some(Self::get_uptime("mysql"))
        } else {
            None
        };
        let mysql_status = if mysql_installed {
            Self::get_service_status("mysql")
        } else {
            None
        };

        Self {
            command_tx: None,
            config: Config::default(),
            enabled: false,
            styles: WebserverStyles::new(),
            mysql_uptime,
            mysql_status,
        }
    }

    fn is_mysql_installed() -> bool {
        std::path::Path::new("/etc/mysql").exists()
    }

    fn get_service_status(service: &str) -> Option<String> {
        Command::new("systemctl")
            .arg("is-active")
            .arg(service)
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    }

    fn get_uptime(service: &str) -> String {
        let output = Command::new("systemctl")
            .arg("show")
            .arg(service)
            .arg("--property=ActiveEnterTimestamp")
            .output()
            .expect("Failed to execute systemctl command");

        let output_str = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = output_str.lines().next() {
            line.replace("ActiveEnterTimestamp=", "")
        } else {
            "Unknown".to_string()
        }
    }

    fn draw_info(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::new()
            .borders(Borders::ALL)
            .border_style(self.styles.border_style)
            .style(self.styles.background_style)
            .border_type(BorderType::Thick);

        frame.render_widget(block, area);

        let mysql_installed = Self::is_mysql_installed();

        let sections = if mysql_installed { 1 } else { 0 };

        let constraints: Vec<Constraint> = (0..sections)
            .map(|_| Constraint::Percentage(100 / sections as u16))
            .collect();
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .margin(2)
            .split(area);

        if mysql_installed {
            self.draw_mysql_info(frame, chunks[0], "MySQL");
        }
    }

    fn draw_mysql_info(&mut self, frame: &mut Frame, area: Rect, title: &str) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3), // Status
                    Constraint::Length(3), // Uptime
                ]
                .as_ref(),
            )
            .split(area);

        let status_text = self
            .mysql_status
            .clone()
            .unwrap_or_else(|| "Unknown".to_string());

        let status_color = if status_text == "active" {
            Color::LightGreen
        } else {
            Color::LightRed
        };

        let status = Paragraph::new(format!("Status: {}", status_text))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("{} Status", title)),
            )
            .style(Style::default().fg(status_color));

        frame.render_widget(status, chunks[0]);

        if let Some(ref uptime) = self.mysql_uptime {
            let mysql_uptime = Paragraph::new(uptime.clone()).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.styles.border_style)
                    .title("MySQL Uptime"),
            );
            frame.render_widget(mysql_uptime, chunks[1]);
        }
    }
}

impl Component for MySql {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::ChangeMode(Module::MySql) = action {
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
