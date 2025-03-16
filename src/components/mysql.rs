pub mod databases;
pub mod general;
pub mod users;

use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    prelude::Frame,
    style::{palette::tailwind, Color, Style, Stylize},
    symbols,
    text::Line,
    widgets::{Block, Padding, Paragraph, Tabs, Widget},
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{
    action::{Action, Module},
    components::mysql::users::UsersView,
    config::Config,
    draw::Drawable,
};

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter)]
enum SelectedTab {
    #[default]
    General,
    Users,
    Databases,
    Logs,
}

impl SelectedTab {
    fn title(self) -> Line<'static> {
        format!("  {self}  ")
            .fg(tailwind::SLATE.c200)
            .bg(tailwind::GRAY.c600)
            .into()
    }

    fn next(self) -> Self {
        let tabs_count = SelectedTab::iter().count();
        let current_index = self as usize;
        let next_index = (current_index + 1) % tabs_count;
        Self::from_repr(next_index).unwrap_or(self)
    }

    fn block(self) -> Block<'static> {
        Block::bordered()
            .border_set(symbols::border::PROPORTIONAL_TALL)
            .padding(Padding::horizontal(1))
            .border_style(tailwind::SLATE.c700)
    }

    fn render_tab4(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Tab 4: Logs")
            .block(self.block())
            .render(area, buf);
    }

    fn footer_info(self) -> Vec<(&'static str, &'static str)> {
        match self {
            Self::General => vec![("<Esc>", "Quit"), ("<Tab>", "Next Tab")],
            Self::Users => vec![
                ("<Esc>", "Quit"),
                ("<↓↑>", "Move up and down"),
                ("<Tab>", "Next Tab"),
            ],
            Self::Databases => vec![
                ("<Esc>", "Quit"),
                ("<↓↑>", "Move up and down"),
                ("<Tab>", "Next Tab"),
            ],
            Self::Logs => vec![("<Esc>", "Quit"), ("<Tab>", "First Tab")],
        }
    }
}

#[derive(Default)]
pub struct MySql {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    enabled: bool,
    selected_tab: SelectedTab,
    users_view: UsersView,
}

impl Drawable for MySql {}

impl MySql {
    pub fn new() -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            enabled: false,
            selected_tab: SelectedTab::default(),
            users_view: UsersView::new(),
        }
    }

    fn render_tabs(&self, area: Rect, frame: &mut Frame) {
        let titles = SelectedTab::iter().map(SelectedTab::title);
        let highlight_style = (Color::default(), tailwind::SLATE.c700);
        let selected_tab_index = self.selected_tab as usize;

        Tabs::new(titles)
            .highlight_style(highlight_style)
            .style(Style::new().bg(tailwind::SLATE.c800))
            .select(selected_tab_index)
            .padding("", "")
            .divider("")
            .render(area, frame.buffer_mut());
    }

    pub fn next_tab(&mut self) {
        self.selected_tab = self.selected_tab.next();
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
        if self.enabled {
            match action {
                Action::ChangeMode(Module::Home) => {
                    self.enabled = false;
                    return Ok(Some(Action::ClearScreen));
                }
                Action::SwtichElement => {
                    self.next_tab();
                }
                _ => {}
            }
            match self.selected_tab {
                SelectedTab::Users => match action {
                    Action::MoveDown => {
                        self.users_view.next_user();
                    }
                    Action::MoveUp => {
                        self.users_view.previous_user();
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        if self.enabled {
            use Constraint::{Length, Min};
            let vertical = Layout::vertical([Length(1), Min(0), Length(2)]);
            let [header_area, inner_area, footer_area] = vertical.areas(area);

            self.render_tabs(header_area, frame);

            //self.selected_tab.render(inner_area, frame.buffer_mut());
            match self.selected_tab {
                SelectedTab::General => self
                    .selected_tab
                    .render_general(inner_area, frame.buffer_mut()),
                SelectedTab::Users => self.selected_tab.render_users(
                    inner_area,
                    frame.buffer_mut(),
                    &mut self.users_view,
                ),
                SelectedTab::Databases => self
                    .selected_tab
                    .render_databases(inner_area, frame.buffer_mut()),
                SelectedTab::Logs => self
                    .selected_tab
                    .render_tab4(inner_area, frame.buffer_mut()),
            }

            self.draw_footer(frame, footer_area, self.selected_tab.footer_info())?;
        }
        Ok(())
    }
}
