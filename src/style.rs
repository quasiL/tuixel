use ratatui::style::{self, Color, Modifier, Style};
use style::palette::tailwind;

#[derive(Default)]
pub struct MenuStyles {
    pub header_style: Style,
    pub header_border_style: Style,
    pub menu_background_style: Style,
    pub selected_row_style: Style,
}

impl MenuStyles {
    pub fn new() -> MenuStyles {
        MenuStyles {
            header_style: Style::new()
                .fg(tailwind::SLATE.c100)
                .bg(tailwind::SLATE.c900),
            header_border_style: Style::new()
                .fg(tailwind::GRAY.c400)
                .bg(tailwind::SLATE.c900),
            menu_background_style: Style::new()
                .fg(tailwind::SLATE.c100)
                .bg(tailwind::SLATE.c900),
            selected_row_style: Style::new()
                .fg(tailwind::SLATE.c100)
                .bg(tailwind::SLATE.c800)
                .add_modifier(Modifier::BOLD),
        }
    }
}

#[derive(Default)]
pub struct TableStyles {
    pub header_style: Style,
    pub selected_row_style: Style,
    pub row_style: Style,
    pub normal_row_color: Color,
    pub alt_row_color: Color,
    pub scrollbar_style: Style,
}

impl TableStyles {
    pub const fn new() -> Self {
        Self {
            header_style: Style::new()
                .fg(tailwind::GRAY.c300)
                .bg(tailwind::SLATE.c800)
                .add_modifier(Modifier::BOLD),
            selected_row_style: Style::new().fg(tailwind::GRAY.c300).bg(tailwind::SKY.c950),
            row_style: Style::new().fg(tailwind::GRAY.c200),
            normal_row_color: tailwind::SLATE.c700,
            alt_row_color: tailwind::SLATE.c600,
            scrollbar_style: Style::new()
                .fg(tailwind::GRAY.c300)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::REVERSED),
        }
    }
}

#[derive(Default)]
pub struct EditWindowStyles {
    pub window_style: Style,
    pub window_border_style: Style,
    pub title_style: Style,
    pub title_border_style: Style,
    pub unselected_input_border_style: Style,
    pub selected_input_border_style: Style,
    pub valid_input_style: Style,
    pub valid_cursor_style: Style,
    pub invalid_input_style: Style,
    pub invalid_cursor_style: Style,
    pub cursor_style: Style,
}

impl EditWindowStyles {
    pub const fn new() -> Self {
        Self {
            window_style: Style::new().bg(tailwind::SLATE.c800),
            window_border_style: Style::new()
                .fg(tailwind::GRAY.c400)
                .bg(tailwind::SLATE.c800),
            title_style: Style::new().fg(tailwind::GRAY.c300),
            title_border_style: Style::new()
                .fg(tailwind::GRAY.c400)
                .bg(tailwind::SLATE.c800),
            unselected_input_border_style: Style::new().fg(Color::Gray),
            selected_input_border_style: Style::new().fg(tailwind::SKY.c600),
            valid_input_style: Style::new().fg(Color::LightGreen),
            valid_cursor_style: Style::new().bg(Color::LightGreen),
            invalid_input_style: Style::new().fg(Color::LightRed),
            invalid_cursor_style: Style::new().bg(Color::LightRed),
            cursor_style: Style::new().bg(Color::White),
        }
    }
}

#[derive(Default)]
pub struct MysqlUsers {
    pub items_style: Style,
    pub items_border_style: Style,
    pub info_style: Style,
    pub info_border_style: Style,
    pub selected_item_style: Style,
    pub scrollbar_style: Style,
}

impl MysqlUsers {
    pub const fn new() -> Self {
        Self {
            items_style: Style::new()
                .fg(tailwind::GRAY.c300)
                .bg(tailwind::SLATE.c800),
            items_border_style: Style::new()
                .fg(tailwind::GRAY.c400)
                .bg(tailwind::SLATE.c800),
            info_style: Style::new()
                .fg(tailwind::GRAY.c300)
                .bg(tailwind::SLATE.c800),
            info_border_style: Style::new()
                .fg(tailwind::GRAY.c400)
                .bg(tailwind::SLATE.c800),
            selected_item_style: Style::new()
                .fg(tailwind::SLATE.c900)
                .bg(tailwind::GRAY.c400),
            scrollbar_style: Style::new()
                .fg(Color::White)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::REVERSED),
        }
    }
}

#[derive(Default)]
pub struct WebserverStyles {
    pub background_style: Style,
    pub border_style: Style,
}

impl WebserverStyles {
    pub const fn new() -> Self {
        Self {
            background_style: Style::new().bg(tailwind::SLATE.c800),
            border_style: Style::new()
                .fg(tailwind::GRAY.c400)
                .bg(tailwind::SLATE.c800),
        }
    }
}
