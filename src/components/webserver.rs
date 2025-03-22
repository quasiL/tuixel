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
pub struct Webserver {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    enabled: bool,
    styles: WebserverStyles,
    apache_uptime: Option<String>,
    apache_status: Option<String>,
    nginx_uptime: Option<String>,
    nginx_status: Option<String>,
}

impl Drawable for Webserver {}

impl Webserver {
    pub fn new() -> Self {
        let (apache_installed, nginx_installed) = Self::is_webserver_installed();
        let apache_uptime = if apache_installed {
            Some(Self::get_uptime("apache2"))
        } else {
            None
        };
        let apache_status = if apache_installed {
            Self::get_service_status("apache2")
        } else {
            None
        };
        let nginx_uptime = if nginx_installed {
            Some(Self::get_uptime("nginx"))
        } else {
            None
        };
        let nginx_status = if nginx_installed {
            Self::get_service_status("nginx")
        } else {
            None
        };

        Self {
            command_tx: None,
            config: Config::default(),
            enabled: false,
            styles: WebserverStyles::new(),
            apache_uptime,
            apache_status,
            nginx_uptime,
            nginx_status,
        }
    }

    fn is_webserver_installed() -> (bool, bool) {
        let apache_installed = std::path::Path::new("/etc/apache2").exists();
        let nginx_installed = std::path::Path::new("/etc/nginx").exists();
        (apache_installed, nginx_installed)
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

        let (apache_installed, nginx_installed) = Webserver::is_webserver_installed();

        let sections = if apache_installed && nginx_installed {
            2
        } else {
            1
        };

        let constraints: Vec<Constraint> = (0..sections)
            .map(|_| Constraint::Percentage(100 / sections as u16))
            .collect();
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .margin(2)
            .split(area);

        if apache_installed {
            self.draw_webserver_info(frame, chunks[0], "Apache", "apache2");
        }
        if nginx_installed {
            let index = if apache_installed { 1 } else { 0 };
            self.draw_webserver_info(frame, chunks[index], "Nginx", "nginx");
        }
    }

    fn draw_webserver_info(&mut self, frame: &mut Frame, area: Rect, title: &str, service: &str) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3), // Status
                    Constraint::Length(3), // Uptime
                                           //Constraint::Length(3), // Listening Ports
                                           //Constraint::Length(3), // Worker Processes
                                           //Constraint::Min(6),    // Logs
                ]
                .as_ref(),
            )
            .split(area);

        let status_text = if service == "apache2" {
            self.apache_status
                .clone()
                .unwrap_or_else(|| "Unknown".to_string())
        } else if service == "nginx" {
            self.nginx_status
                .clone()
                .unwrap_or_else(|| "Unknown".to_string())
        } else {
            "Unknown".to_string()
        };

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

        if service == "apache2" {
            if let Some(ref uptime) = self.apache_uptime {
                let apache_uptime = Paragraph::new(uptime.clone()).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(self.styles.border_style)
                        .title("Apache Uptime"),
                );
                frame.render_widget(apache_uptime, chunks[1]);
            }
        } else if service == "nginx" {
            if let Some(ref uptime) = self.nginx_uptime {
                let nginx_uptime = Paragraph::new(uptime.clone()).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(self.styles.border_style)
                        .title("Nginx Uptime"),
                );
                frame.render_widget(nginx_uptime, chunks[1]);
            }
        }

        // let ports = Paragraph::new("Listening on: 80, 443").block(
        //     Block::default()
        //         .borders(Borders::ALL)
        //         .border_style(self.styles.border_style)
        //         .title("Ports"),
        // );
        // frame.render_widget(ports, chunks[2]);

        // let workers = Paragraph::new("Worker Processes: 5").block(
        //     Block::default()
        //         .borders(Borders::ALL)
        //         .border_style(self.styles.border_style)
        //         .title("Workers"),
        // );
        // frame.render_widget(workers, chunks[3]);

        // let logs = Table::new(
        //     vec![
        //         Row::new(vec![Span::raw("[2025-03-20 12:32:45] GET /index")]),
        //         Row::new(vec![Span::raw("[2025-03-20 12:33:01] POST /login")]),
        //     ],
        //     &[Constraint::Percentage(100)],
        // )
        // .block(
        //     Block::default()
        //         .borders(Borders::ALL)
        //         .border_style(self.styles.border_style)
        //         .title("Last Log Entries"),
        // );
        // frame.render_widget(logs, chunks[4]);
    }
}

impl Component for Webserver {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::ChangeMode(Module::Webserver) = action {
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
