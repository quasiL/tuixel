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

// pub mod databases;
// pub mod general;
// pub mod users;

// use color_eyre::Result;
// use ratatui::{
//     buffer::Buffer,
//     layout::{Constraint, Layout, Rect},
//     prelude::Frame,
//     style::{palette::tailwind, Color, Style, Stylize},
//     symbols,
//     text::Line,
//     widgets::{Block, Padding, Paragraph, Tabs, Widget},
// };
// use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
// use tokio::sync::mpsc::UnboundedSender;

// use super::Component;
// use crate::{
//     action::{Action, Module},
//     components::mysql::users::UsersView,
//     config::Config,
//     draw::Drawable,
// };

// #[derive(Default, Clone, Copy, Display, FromRepr, EnumIter)]
// enum SelectedTab {
//     #[default]
//     General,
//     Users,
//     Databases,
//     Logs,
// }

// impl SelectedTab {
//     fn title(self) -> Line<'static> {
//         format!("  {self}  ")
//             .fg(tailwind::SLATE.c200)
//             .bg(tailwind::GRAY.c600)
//             .into()
//     }

//     fn next(self) -> Self {
//         let tabs_count = SelectedTab::iter().count();
//         let current_index = self as usize;
//         let next_index = (current_index + 1) % tabs_count;
//         Self::from_repr(next_index).unwrap_or(self)
//     }

//     fn block(self) -> Block<'static> {
//         Block::bordered()
//             .border_set(symbols::border::PROPORTIONAL_TALL)
//             .padding(Padding::horizontal(1))
//             .border_style(tailwind::SLATE.c700)
//     }

//     fn render_tab4(self, area: Rect, buf: &mut Buffer) {
//         Paragraph::new("Tab 4: Logs")
//             .block(self.block())
//             .render(area, buf);
//     }

//     fn footer_info(self) -> Vec<(&'static str, &'static str)> {
//         match self {
//             Self::General => vec![("<Esc>", "Quit"), ("<Tab>", "Next Tab")],
//             Self::Users => vec![
//                 ("<Esc>", "Quit"),
//                 ("<↓↑>", "Move up and down"),
//                 ("<Tab>", "Next Tab"),
//             ],
//             Self::Databases => vec![
//                 ("<Esc>", "Quit"),
//                 ("<↓↑>", "Move up and down"),
//                 ("<Tab>", "Next Tab"),
//             ],
//             Self::Logs => vec![("<Esc>", "Quit"), ("<Tab>", "First Tab")],
//         }
//     }
// }

// #[derive(Default)]
// pub struct MySql {
//     command_tx: Option<UnboundedSender<Action>>,
//     config: Config,
//     enabled: bool,
//     selected_tab: SelectedTab,
//     users_view: UsersView,
// }

// impl Drawable for MySql {}

// impl MySql {
//     pub fn new() -> Self {
//         Self {
//             command_tx: None,
//             config: Config::default(),
//             enabled: false,
//             selected_tab: SelectedTab::default(),
//             users_view: UsersView::new(),
//         }
//     }

//     fn render_tabs(&self, area: Rect, frame: &mut Frame) {
//         let titles = SelectedTab::iter().map(SelectedTab::title);
//         let highlight_style = (Color::default(), tailwind::SLATE.c700);
//         let selected_tab_index = self.selected_tab as usize;

//         Tabs::new(titles)
//             .highlight_style(highlight_style)
//             .style(Style::new().bg(tailwind::SLATE.c800))
//             .select(selected_tab_index)
//             .padding("", "")
//             .divider("")
//             .render(area, frame.buffer_mut());
//     }

//     pub fn next_tab(&mut self) {
//         self.selected_tab = self.selected_tab.next();
//     }
// }

// impl Component for MySql {
//     fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
//         self.command_tx = Some(tx);
//         Ok(())
//     }

//     fn register_config_handler(&mut self, config: Config) -> Result<()> {
//         self.config = config;
//         Ok(())
//     }

//     fn update(&mut self, action: Action) -> Result<Option<Action>> {
//         if let Action::ChangeMode(Module::MySql) = action {
//             self.enabled = true;
//         }
//         if self.enabled {
//             match action {
//                 Action::ChangeMode(Module::Home) => {
//                     self.enabled = false;
//                     return Ok(Some(Action::ClearScreen));
//                 }
//                 Action::SwtichElement => {
//                     self.next_tab();
//                 }
//                 _ => {}
//             }
//             if let SelectedTab::Users = self.selected_tab {
//                 match action {
//                     Action::MoveDown => {
//                         self.users_view.next_user();
//                     }
//                     Action::MoveUp => {
//                         self.users_view.previous_user();
//                     }
//                     _ => {}
//                 }
//             }
//         }
//         Ok(None)
//     }

//     fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
//         if self.enabled {
//             use Constraint::{Length, Min};
//             let vertical = Layout::vertical([Length(1), Min(0), Length(2)]);
//             let [header_area, inner_area, footer_area] = vertical.areas(area);

//             self.render_tabs(header_area, frame);

//             //self.selected_tab.render(inner_area, frame.buffer_mut());
//             match self.selected_tab {
//                 SelectedTab::General => self
//                     .selected_tab
//                     .render_general(inner_area, frame.buffer_mut()),
//                 SelectedTab::Users => self.selected_tab.render_users(
//                     inner_area,
//                     frame.buffer_mut(),
//                     &mut self.users_view,
//                 ),
//                 SelectedTab::Databases => self
//                     .selected_tab
//                     .render_databases(inner_area, frame.buffer_mut()),
//                 SelectedTab::Logs => self
//                     .selected_tab
//                     .render_tab4(inner_area, frame.buffer_mut()),
//             }

//             self.draw_footer(frame, footer_area, self.selected_tab.footer_info())?;
//         }
//         Ok(())
//     }
// }
