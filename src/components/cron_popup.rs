use color_eyre::Result;
use ratatui::{
    crossterm::event::{MouseEvent, MouseEventKind},
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    prelude::Frame,
    text::Text,
    widgets::{Block, Borders, Clear, List, ListState, Paragraph},
};
use tokio::sync::mpsc::UnboundedSender;
use tui_big_text::{BigText, PixelSize};

use super::Component;
use crate::{
    action::{Action, Module},
    config::Config,
    draw::Drawable,
    style::EditWindowStyles,
};

#[derive(Default)]
pub struct CronPopup {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    enabled: bool,
    cron: String,
}

impl Drawable for CronPopup {}

impl CronPopup {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Component for CronPopup {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::ChangeMode(Module::CronPopup) = action {
            self.enabled = true;
        }
        if let Action::SendData(ref _notation) = action {
            self.cron = _notation.clone();
        }
        if self.enabled {
            match action {
                Action::ChangeMode(Module::Cron) => {
                    self.enabled = false;
                }
                _ => {}
            }
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        if self.enabled {
            let area = center(
                frame.area(),
                Constraint::Percentage(70),
                Constraint::Length(19),
            );
            frame.render_widget(Clear, area);

            let layout = Layout::vertical([Constraint::Length(17), Constraint::Length(2)])
                .flex(Flex::SpaceBetween);
            let [main_area, footer_area] = layout.areas(area);

            let main = Layout::vertical([
                Constraint::Length(4),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .margin(2)
            .flex(Flex::Start);
            let [title, cron_notation, job, description] = main.areas(main_area);

            let test = Paragraph::new(Text::from(self.cron.clone()));

            let footer = Layout::vertical([Constraint::Length(3)]);
            let [help] = footer.areas(footer_area);

            frame.render_widget(test, title);
        }

        Ok(())
    }
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}
