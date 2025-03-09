use ratatui::{
    crossterm::event::{self, KeyCode},
    layout::{Constraint, Flex, Layout, Rect},
    prelude::{Buffer, Widget},
    style::{self, Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, TableState},
};
use style::palette::tailwind;
use tui_textarea::{CursorMove, TextArea};

use crate::{draw::Drawable, style::EditWindowStyles};

#[derive(Default)]
enum ActiveInput {
    #[default]
    CronNotation,
    Job,
    JobDescription,
}

#[derive(Default)]
pub struct Inputs {
    styles: EditWindowStyles,
    pub cron_notation: TextArea<'static>,
    pub job: TextArea<'static>,
    pub job_description: TextArea<'static>,
    pub current_input: ActiveInput,
    pub cron_notation_value: String,
    pub job_value: String,
    pub job_description_value: String,
    pub is_new: bool,
}
