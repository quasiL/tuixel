use ratatui::{
    buffer::Buffer,
    crossterm::event::{MouseEvent, MouseEventKind},
    layout::{Alignment, Constraint, Layout, Rect},
    prelude::Frame,
    style::{palette::tailwind, Color, Stylize},
    symbols,
    text::{Line, Text},
    widgets::{Block, Borders, List, ListState, Padding, Paragraph, Tabs, Widget},
};

use crate::components::mysql::SelectedTab;

impl SelectedTab {
    pub fn render_general(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Tab 1: General")
            .block(self.block())
            .render(area, buf);
    }
}
