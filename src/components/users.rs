pub mod utils;

use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    prelude::Frame,
    text::Text,
    widgets::{
        Cell, HighlightSpacing, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
        TableState,
    },
};
use tokio::sync::mpsc::UnboundedSender;

use super::{
    users::utils::constraint_len_calculator, users::utils::get_users_from_passwd, Component,
};
use crate::{
    action::{Action, Module},
    config::Config,
    draw::Drawable,
    style::TableStyles,
};

impl Drawable for Users {}
const ITEM_HEIGHT: usize = 3;

#[derive(Default)]
pub struct Users {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    enabled: bool,
    state: TableState,
    items: Vec<User>,
    longest_item_lens: (u16, u16, u16),
    scroll_state: ScrollbarState,
    styles: TableStyles,
}

#[derive(Default)]
pub struct User {
    pub username: String,
    pub docroot: String,
    pub shell: String,
}

impl User {
    const fn ref_array(&self) -> [&String; 3] {
        [&self.username, &self.docroot, &self.shell]
    }
}

impl Users {
    pub fn new() -> Self {
        let users = vec![];
        let scroll_position = if users.is_empty() {
            0
        } else {
            (users.len() - 1) * ITEM_HEIGHT
        };
        Self {
            command_tx: None,
            config: Config::default(),
            enabled: false,
            state: TableState::default().with_selected(0),
            longest_item_lens: constraint_len_calculator(&users),
            scroll_state: ScrollbarState::new(scroll_position),
            styles: TableStyles::new(),
            items: users,
        }
    }

    fn draw_table(&mut self, frame: &mut Frame, area: Rect) {
        let header = ["Username", "Document Root", "Shell"]
            .into_iter()
            .map(|title| Cell::from(Text::from(format!("\n{}\n", title))))
            .collect::<Row>()
            .style(self.styles.header_style)
            .height(3);

        let rows = self.items.iter().enumerate().map(|(i, data)| {
            let color = if i % 2 == 0 {
                self.styles.normal_row_color
            } else {
                self.styles.alt_row_color
            };
            let item = data.ref_array();
            item.into_iter()
                .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
                .collect::<Row>()
                .style(self.styles.row_style.bg(color))
                .height(ITEM_HEIGHT.try_into().unwrap())
        });

        let bar = " ▌ ";
        self.longest_item_lens = constraint_len_calculator(&self.items);
        if self.longest_item_lens.0 < "Username".len() as u16 {
            self.longest_item_lens.0 = "Username".len() as u16;
        }
        let table = Table::new(
            rows,
            [
                Constraint::Length(self.longest_item_lens.0 + 2),
                Constraint::Min(self.longest_item_lens.1 + 2),
                Constraint::Min(self.longest_item_lens.2),
            ],
        )
        .header(header)
        .row_highlight_style(self.styles.selected_row_style)
        .style(
            self.styles
                .row_style
                .bg(if (self.items.len() + 1) % 2 == 0 {
                    self.styles.alt_row_color
                } else {
                    self.styles.normal_row_color
                }),
        )
        .highlight_symbol(Text::from(vec!["".into(), bar.into(), "".into()]))
        .highlight_spacing(HighlightSpacing::Always);

        frame.render_stateful_widget(table, area, &mut self.state);
    }

    fn draw_scrollbar(&mut self, frame: &mut Frame, area: Rect) {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .style(self.styles.scrollbar_style);

        frame.render_stateful_widget(
            scrollbar,
            area.inner(Margin {
                vertical: 1,
                horizontal: 1,
            }),
            &mut self.scroll_state,
        );
    }

    fn next_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    fn previous_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    fn first_row(&mut self) {
        self.state.select(Some(0));
        self.scroll_state = self.scroll_state.position(0);
    }

    fn last_row(&mut self) {
        if !self.items.is_empty() {
            let last_index = self.items.len() - 1;
            self.state.select(Some(last_index));
            self.scroll_state = self.scroll_state.position(last_index * ITEM_HEIGHT);
        }
    }
}

impl Component for Users {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::ChangeMode(Module::Users) = action {
            self.items = get_users_from_passwd(&self.config.settings.users.docroot);
            let scroll_position = if self.items.is_empty() {
                0
            } else {
                (self.items.len() - 1) * ITEM_HEIGHT
            };
            self.scroll_state = ScrollbarState::new(scroll_position);
            self.enabled = true;
        }
        if self.enabled {
            match action {
                Action::ChangeMode(Module::Home) => {
                    self.enabled = false;
                    return Ok(Some(Action::ClearScreen));
                }
                Action::MoveUp => {
                    self.previous_row();
                }
                Action::MoveDown => {
                    self.next_row();
                }
                Action::MoveToTheFirst => {
                    self.first_row();
                }
                Action::MoveToTheLast => {
                    self.last_row();
                }
                _ => {}
            }
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        if self.enabled {
            let vertical = Layout::vertical([Constraint::Min(1), Constraint::Length(2)]);
            let rects = vertical.split(area);

            self.draw_table(frame, rects[0]);
            self.draw_scrollbar(frame, rects[0]);
            self.draw_footer(
                frame,
                rects[1],
                vec![("<Esc>", "Quit"), ("<↓↑>", "Move up and down")],
            )?;
        }
        Ok(())
    }
}
