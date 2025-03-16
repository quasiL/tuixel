use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    prelude::{Buffer, StatefulWidget, Widget},
    text::Line,
    widgets::{
        Block, BorderType, Borders, List, ListItem, ListState, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
};

use crate::components::mysql::SelectedTab;
use crate::style::MysqlUsers;

const ITEM_HEIGHT: usize = 1;

impl SelectedTab {
    pub fn render_users(self, area: Rect, buf: &mut Buffer, users_view: &mut UsersView) {
        let main_area = area;

        let [user_list_area, info_area] =
            Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
                .areas(main_area);

        users_view.render_users_list(user_list_area, buf);
        users_view.render_users_scrollbar(user_list_area, buf);
        users_view.render_users_info(info_area, buf);
    }
}

struct User {
    username: String,
}

impl User {
    fn new(username: &str) -> Self {
        Self {
            username: username.to_string(),
        }
    }
}

#[derive(Default)]
pub struct UsersView {
    styles: MysqlUsers,
    list_state: ListState,
    scroll_state: ScrollbarState,
    items: Vec<User>,
}

impl UsersView {
    pub fn new() -> Self {
        let mysql_users = vec![User::new("root"), User::new("admin"), User::new("guest")];
        let state = ListState::default().with_selected(Some(0));
        let scroll_position = if mysql_users.is_empty() {
            0
        } else {
            (mysql_users.len() - 1) * 1
        };

        Self {
            styles: MysqlUsers::new(),
            list_state: state,
            scroll_state: ScrollbarState::new(scroll_position),
            items: mysql_users,
        }
    }

    pub fn next_user(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) if i < self.items.len() - 1 => i + 1,
            _ => 0,
        };
        self.list_state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn previous_user(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) if i > 0 => i - 1,
            _ => self.items.len().saturating_sub(1),
        };
        self.list_state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    fn render_users_list(&mut self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|user| ListItem::new(vec![Line::from(user.username.as_str()).centered()]))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.styles.items_border_style)
                    .border_type(BorderType::Thick)
                    .title("Users")
                    .style(self.styles.items_style),
            )
            .highlight_style(self.styles.selected_item_style);

        StatefulWidget::render(list, area, buf, &mut self.list_state);
    }

    fn render_users_info(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.styles.info_border_style)
            .border_type(BorderType::Thick)
            .style(self.styles.info_style);

        Widget::render(block, area, buf);
    }

    fn render_users_scrollbar(&mut self, area: Rect, buf: &mut Buffer) {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .style(self.styles.scrollbar_style);

        StatefulWidget::render(
            scrollbar,
            area.inner(Margin {
                vertical: 1,
                horizontal: 1,
            }),
            buf,
            &mut self.scroll_state,
        );
    }
}
