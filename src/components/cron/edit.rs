use ratatui::{
    crossterm::event::{self, KeyCode},
    layout::{Constraint, Flex, Layout, Rect},
    prelude::{Buffer, Frame, Widget},
    style::{self, Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, TableState},
};
use style::palette::tailwind;
use tui_textarea::{CursorMove, TextArea};

use crate::components::cron::utils::{
    get_human_readable_cron, get_next_execution, save_to_crontab,
};
use crate::components::cron::CronJob;
use crate::{draw::Drawable, style::EditWindowStyles};

impl Drawable for Inputs {}

#[derive(Default)]
enum ActiveInput {
    #[default]
    CronNotation,
    Job,
    JobDescription,
}

impl ActiveInput {
    pub fn next(&self) -> Self {
        match self {
            ActiveInput::CronNotation => ActiveInput::Job,
            ActiveInput::Job => ActiveInput::JobDescription,
            ActiveInput::JobDescription => ActiveInput::CronNotation,
        }
    }
}

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

impl Default for Inputs {
    fn default() -> Self {
        Self {
            styles: EditWindowStyles::new(),
            cron_notation: TextArea::default(),
            job: TextArea::default(),
            job_description: TextArea::default(),
            current_input: ActiveInput::CronNotation,
            cron_notation_value: String::new(),
            job_value: String::new(),
            job_description_value: String::new(),
            is_new: true,
        }
    }
}

impl Inputs {
    pub fn handle_inputs(
        &mut self,
        key: event::KeyEvent,
        show_popup: &mut bool,
        cron_jobs: &mut Vec<CronJob>,
        table_state: &mut TableState,
    ) {
        let ctrl_pressed = key.modifiers.contains(event::KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Tab => {
                self.current_input = self.current_input.next();
            }
            KeyCode::Esc => {
                *show_popup = false;
                self.flash_inputs();
                self.flash_values();
                self.current_input = ActiveInput::CronNotation;
            }
            KeyCode::Char('s') => match validate(&mut self.cron_notation) {
                Ok(_) => {
                    if self.is_new {
                        cron_jobs.push(self.create_new_cron());
                        table_state.select(Some(cron_jobs.len() - 1));
                    } else {
                        self.update_selected_cron(&mut cron_jobs[table_state.selected().unwrap()]);
                    }
                    save_to_crontab(cron_jobs).unwrap_or_else(|err| {
                        eprint!("Error saving to crontab: {}", err);
                    });

                    *show_popup = false;
                }
                Err(ValidationError::InvalidCronExpression(_)) => {}
            },
            // KeyCode::Char('v') if ctrl_pressed => {
            //     self.handle_paste();
            // }
            _ => match self.current_input {
                ActiveInput::CronNotation => {
                    let cron_input = &mut self.cron_notation;
                    let cron_value = &mut self.cron_notation_value;
                    if cron_input.input(key) {
                        cron_value.clear();
                        if let Some(first_line) = cron_input.lines().get(0) {
                            cron_value.push_str(first_line);
                        }
                    }
                }
                ActiveInput::Job => {
                    let job_input = &mut self.job;
                    let job_value = &mut self.job_value;
                    if job_input.input(key) {
                        job_value.clear();
                        if let Some(first_line) = job_input.lines().get(0) {
                            job_value.push_str(first_line);
                        }
                    }
                }
                ActiveInput::JobDescription => {
                    let job_description_input = &mut self.job_description;
                    let job_description_value = &mut self.job_description_value;
                    if job_description_input.input(key) {
                        job_description_value.clear();
                        if let Some(first_line) = job_description_input.lines().get(0) {
                            job_description_value.push_str(first_line);
                        }
                    }
                }
            },
        }
    }

    fn flash_inputs(&mut self) {
        self.cron_notation.delete_line_by_head();
        self.cron_notation.delete_line_by_end();
        self.job.delete_line_by_head();
        self.job.delete_line_by_end();
        self.job_description.delete_line_by_head();
        self.job_description.delete_line_by_end();
    }

    fn flash_values(&mut self) {
        self.cron_notation_value.clear();
        self.job_value.clear();
        self.job_description_value.clear();
    }

    pub fn init_empty(&mut self) {
        self.is_new = true;
        self.flash_inputs();
        self.flash_values();
        self.current_input = ActiveInput::CronNotation;
        self.initial_render();
    }

    pub fn init(&mut self, cron_jobs: &mut Vec<CronJob>, table_state: &mut TableState) {
        self.flash_inputs();
        self.flash_values();
        self.current_input = ActiveInput::CronNotation;
        self.initial_render();

        if !self.is_new {
            let selected_cron = &mut cron_jobs[table_state.selected().unwrap()];
            self.cron_notation.insert_str(&selected_cron.cron_notation);
            self.job.insert_str(&selected_cron.job);
            self.job_description
                .insert_str(&selected_cron.job_description);

            self.cron_notation_value = selected_cron.cron_notation.to_string();
            self.job_value = selected_cron.job.to_string();
            self.job_description_value = selected_cron.job_description.to_string();
        }
    }

    fn create_new_cron(&mut self) -> CronJob {
        CronJob::new({
            CronJob {
                cron_notation: format!("{}", self.cron_notation_value),
                job: format!("{}", self.job_value),
                job_description: format!("{}", self.job_description_value),
                next_execution: get_next_execution(&self.cron_notation_value),
            }
        })
    }

    fn update_selected_cron(&mut self, selected_cron: &mut CronJob) {
        selected_cron.cron_notation = format!("{}", self.cron_notation_value);
        selected_cron.job = format!("{}", self.job_value);
        selected_cron.job_description = format!("{}", self.job_description_value);
        selected_cron.next_execution = get_next_execution(&self.cron_notation_value);
    }

    fn initial_render(&mut self) {
        let cron_input = &mut self.cron_notation;
        let job_input = &mut self.job;
        let description_input = &mut self.job_description;

        cron_input.set_placeholder_text("Enter a cron notation");
        cron_input.set_cursor_line_style(Style::default());
        cron_input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(self.styles.selected_input_border_style),
        );

        job_input.set_placeholder_text("Enter a job");
        job_input.set_cursor_line_style(Style::default());

        description_input.set_placeholder_text("Enter a description");
        description_input.set_cursor_line_style(Style::default());
    }

    pub fn draw_inputs(&mut self, frame: &mut Frame, area: Rect) {
        let area = popup_area(area, 70);
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
        let [title_area, cron_notation_area, job_area, description_area] = main.areas(main_area);

        let footer = Layout::vertical([Constraint::Length(3)]);
        let [info_area] = footer.areas(footer_area);

        let selected_cron_notation = get_human_readable_cron(self.cron_notation_value.as_str())
            .unwrap_or_else(|e| format!("{}", e));

        let wrapped_text: Vec<Line> = selected_cron_notation
            .chars()
            .collect::<Vec<_>>()
            .chunks(100)
            .map(|chunk| {
                Line::from(Span::styled(
                    chunk.iter().collect::<String>(),
                    self.styles.title_style,
                ))
            })
            .collect();

        let title = Paragraph::new(Text::from(wrapped_text))
            .style(self.styles.title_style)
            .centered()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick)
                    .border_style(self.styles.title_border_style),
            );
        frame.render_widget(title, title_area);

        self.draw_footer(
            frame,
            info_area,
            vec![
                ("<Esc>", "Close without saving"),
                ("<Tab>", "Move to the next field"),
                ("<Enter>", "Save and close"),
            ],
        )
        .unwrap();

        let cron_input: &mut TextArea<'_> = &mut self.cron_notation;
        let job_input = &mut self.job;
        let description_input = &mut self.job_description;

        match self.current_input {
            ActiveInput::CronNotation => {
                match validate(cron_input) {
                    Ok(_) => {
                        cron_input.set_block(
                            Block::default()
                                .borders(Borders::ALL)
                                .border_style(self.styles.valid_input_style)
                                .title("Cron notation* (OK)"),
                        );
                        cron_input.set_cursor_style(self.styles.valid_cursor_style);
                    }
                    Err(ValidationError::InvalidCronExpression(message)) => {
                        cron_input.set_block(
                            Block::default()
                                .borders(Borders::ALL)
                                .border_style(self.styles.invalid_input_style)
                                .title(format!("Cron notation* ({})", message)),
                        );
                        cron_input.set_cursor_style(self.styles.invalid_cursor_style);
                    }
                }
                frame.render_widget(cron_input.widget(), cron_notation_area);

                job_input.set_cursor_style(Style::default());
                job_input.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(self.styles.unselected_input_border_style)
                        .title("Job"),
                );
                frame.render_widget(job_input.widget(), job_area);

                description_input.set_cursor_style(Style::default());
                description_input.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(self.styles.unselected_input_border_style)
                        .title("Description"),
                );
                frame.render_widget(description_input.widget(), description_area);
            }
            ActiveInput::Job => {
                job_input.set_cursor_style(self.styles.cursor_style);
                job_input.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(self.styles.selected_input_border_style)
                        .title("Job"),
                );
                frame.render_widget(job_input.widget(), job_area);

                cron_input.set_cursor_style(Style::default());
                cron_input.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(self.styles.unselected_input_border_style)
                        .title("Cron notation*"),
                );
                frame.render_widget(cron_input.widget(), cron_notation_area);

                description_input.set_cursor_style(Style::default());
                description_input.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(self.styles.unselected_input_border_style)
                        .title("Description"),
                );
                frame.render_widget(description_input.widget(), description_area);
            }
            ActiveInput::JobDescription => {
                description_input.set_cursor_style(self.styles.cursor_style);
                description_input.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(self.styles.selected_input_border_style)
                        .title("Description"),
                );
                frame.render_widget(description_input.widget(), description_area);

                cron_input.set_cursor_style(Style::default());
                cron_input.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(self.styles.unselected_input_border_style)
                        .title("Cron notation*"),
                );
                frame.render_widget(cron_input.widget(), cron_notation_area);

                job_input.set_cursor_style(Style::default());
                job_input.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(self.styles.unselected_input_border_style)
                        .title("Job"),
                );
                frame.render_widget(job_input.widget(), job_area);
            }
        }
    }
}

fn popup_area(area: Rect, percent_x: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Max(19)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidCronExpression(String),
}

fn validate(textarea: &mut TextArea) -> Result<(), ValidationError> {
    use chrono::Utc;
    use cron_parser::parse;

    let input = textarea
        .lines()
        .get(0)
        .map(|s| s.as_str())
        .unwrap_or("")
        .trim();

    let now = Utc::now();

    match parse(input, &now) {
        Ok(_) => {
            textarea.set_style(Style::default().fg(Color::LightGreen));
            Ok(())
        }
        Err(_) => {
            textarea.set_style(Style::default().fg(Color::LightRed));
            Err(ValidationError::InvalidCronExpression(
                "Invalid cron expression".to_string(),
            ))
        }
    }
}
