use color_eyre::Result;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind},
    layout::{Alignment, Constraint, Layout, Rect},
    prelude::Frame,
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, List, ListState, Paragraph},
};
use tokio::sync::mpsc::UnboundedSender;
use tui_big_text::{BigText, PixelSize};

use super::Component;
use crate::{
    action::{Action, Module},
    config::Config,
    style::MenuStyles,
};

#[derive(Default)]
pub struct Home {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    enabled: bool,
    menu_list: MenuList,
    styles: MenuStyles,
    menu_items: Vec<MenuItem>,
}

#[derive(Default)]
struct MenuList {
    items: Vec<String>,
    state: ListState,
}

struct MenuItem {
    label: &'static str,
    action: fn() -> Action,
}

impl Home {
    pub fn new() -> Self {
        let menu_items = vec![
            MenuItem {
                label: "🕗 Cron Jobs",
                action: || Action::ChangeMode(Module::Cron),
            },
            MenuItem {
                label: "👤 FTP",
                action: || Action::Quit,
            },
            MenuItem {
                label: "🐬 MySQL",
                action: || Action::Quit,
            },
            MenuItem {
                label: "🌐 Webserver",
                action: || Action::Quit,
            },
            MenuItem {
                label: "🔧 Settings",
                action: || Action::Quit,
            },
        ];
        Self {
            command_tx: None,
            config: Config::default(),
            enabled: true,
            menu_list: MenuList {
                items: menu_items
                    .iter()
                    .map(|item| item.label.to_string())
                    .collect(),
                state: ListState::default().with_selected(Some(0)),
            },
            styles: MenuStyles::new(),
            menu_items,
        }
    }

    fn draw_title(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let block = Block::default().style(self.styles.header_border_style);
        let inner_area = block.inner(area);

        let [_top_padding, text_area, _bottom_padding] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .areas(inner_area);

        let big_text = BigText::builder()
            .pixel_size(PixelSize::Sextant)
            .alignment(Alignment::Center)
            .style(self.styles.header_style)
            .lines(vec!["TUIxel".into()])
            .build();

        frame.render_widget(block, area); // Render the block
        frame.render_widget(big_text, text_area); // Render the text

        Ok(())
    }

    fn draw_menu(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let block = Block::new().style(self.styles.menu_background_style);

        let items: Vec<Text> = self
            .menu_list
            .items
            .iter()
            .map(|item| Text::from_iter(["", item.as_str(), ""]).centered())
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(self.styles.selected_row_style);

        frame.render_stateful_widget(list, area, &mut self.menu_list.state);

        Ok(())
    }

    fn draw_footer(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        keybinds: Vec<(&str, &str)>,
    ) -> Result<()> {
        let mut spans: Vec<Span> = Vec::new();
        let mut lines: Vec<Line> = Vec::new();

        let mut current_width = 0;
        let max_width = area.width.saturating_sub(4);

        for (key, desc) in &keybinds {
            let key_span = Span::styled(*key, Style::default().fg(Color::Gray));
            let desc_span = Span::styled(*desc, Style::default().fg(Color::DarkGray));
            let spacing = Span::raw("  ");

            let pair_width = key.len() as u16 + 1 + desc.len() as u16 + 2;

            if current_width + pair_width > max_width && !spans.is_empty() {
                lines.push(Line::from(spans));
                spans = Vec::new();
                current_width = 0;
            }

            spans.push(key_span);
            spans.push(Span::raw(" "));
            spans.push(desc_span);
            spans.push(spacing);

            current_width += pair_width;
        }

        if !spans.is_empty() {
            lines.push(Line::from(spans));
        }

        let info_footer = Paragraph::new(Text::from(lines))
            .style(Style::default().bg(Color::Rgb(30, 41, 59)))
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default());

        frame.render_widget(info_footer, area);

        Ok(())
    }

    fn process_select(&mut self) -> Result<Option<Action>> {
        let action = self
            .menu_list
            .state
            .selected()
            .and_then(|index| self.menu_items.get(index))
            .map(|item| (item.action)());

        Ok(action)
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
                KeyCode::Enter => {
                    self.enabled = false;
                    return self.process_select();
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.menu_list.state.select_next();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.menu_list.state.select_previous();
                }
                KeyCode::Home => {
                    self.menu_list.state.select_first();
                }
                KeyCode::End => {
                    self.menu_list.state.select_last();
                }
                _ => {}
            }
        }
        Ok(None)
    }

    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        match mouse.kind {
            MouseEventKind::Down(_) => {
                let menu_start_row = 6;
                let menu_height = self.menu_list.items.len();
                let item_vertical_span: usize = 3;

                if mouse.row >= menu_start_row
                    && mouse.row < menu_start_row + menu_height as u16 * item_vertical_span as u16
                {
                    let selected_index =
                        (mouse.row as usize - menu_start_row as usize) / item_vertical_span;

                    self.menu_list.state.select(Some(selected_index));
                    self.enabled = false;
                    return self.process_select();
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::ChangeMode(Module::Home) = action {
            self.enabled = true;
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        if self.enabled {
            let [main_area, footer_area] =
                Layout::vertical([Constraint::Min(1), Constraint::Length(2)]).areas(area);

            let [title_area, menu_area] =
                Layout::vertical([Constraint::Length(5), Constraint::Min(1)]).areas(main_area);

            self.draw_menu(frame, menu_area)?;
            self.draw_title(frame, title_area)?;
            self.draw_footer(
                frame,
                footer_area,
                vec![
                    ("<Esc/q>", "Quit"),
                    ("<Enter>", "Select"),
                    ("<↓↑>", "Move up and down"),
                ],
            )?;
        }
        Ok(())
    }
}
