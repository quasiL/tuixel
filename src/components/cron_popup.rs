use color_eyre::Result;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Flex, Layout, Rect},
    prelude::Frame,
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};
use tokio::sync::mpsc::UnboundedSender;
use tui_textarea::TextArea;

use super::Component;
use crate::components::cron::utils::get_human_readable_cron;
use crate::{
    action::{Action, Module},
    config::Config,
    draw::Drawable,
    style::EditWindowStyles,
};

#[derive(Default, PartialEq)]
enum ActiveInput {
    #[default]
    CronNotation,
    Job,
    JobDescription,
    AIQuestion,
}

impl ActiveInput {
    pub fn next(&self) -> Self {
        match self {
            ActiveInput::CronNotation => ActiveInput::Job,
            ActiveInput::Job => ActiveInput::JobDescription,
            ActiveInput::JobDescription => ActiveInput::AIQuestion,
            ActiveInput::AIQuestion => ActiveInput::CronNotation,
        }
    }
}

pub struct CronPopup {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    enabled: bool,
    is_new: bool,
    styles: EditWindowStyles,
    index: i32,
    cron_notation: TextArea<'static>,
    job: TextArea<'static>,
    job_description: TextArea<'static>,
    ai_question: TextArea<'static>,
    current_input: ActiveInput,
    cron_notation_value: String,
    job_value: String,
    job_description_value: String,
    ai_question_value: String,
}

impl Drawable for CronPopup {}

impl Default for CronPopup {
    fn default() -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            enabled: false,
            index: -1,
            styles: EditWindowStyles::new(),
            cron_notation: TextArea::default(),
            job: TextArea::default(),
            job_description: TextArea::default(),
            ai_question: TextArea::default(),
            current_input: ActiveInput::CronNotation,
            cron_notation_value: String::new(),
            job_value: String::new(),
            job_description_value: String::new(),
            ai_question_value: String::new(),
            is_new: true,
        }
    }
}

impl CronPopup {
    pub fn new() -> Self {
        Self::default()
    }

    fn flash_inputs(&mut self) {
        self.cron_notation.delete_line_by_head();
        self.cron_notation.delete_line_by_end();
        self.job.delete_line_by_head();
        self.job.delete_line_by_end();
        self.job_description.delete_line_by_head();
        self.job_description.delete_line_by_end();
        self.ai_question.delete_line_by_head();
        self.ai_question.delete_line_by_end();
    }

    fn flash_values(&mut self) {
        self.cron_notation_value.clear();
        self.job_value.clear();
        self.job_description_value.clear();
        self.ai_question_value.clear();
    }

    fn initial_render(&mut self) {
        self.flash_inputs();
        self.flash_values();
        self.current_input = ActiveInput::CronNotation;

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

        self.ai_question
            .set_placeholder_text("Ask AI to convert human readable into cron notation");
        self.ai_question.set_cursor_line_style(Style::default());
    }

    fn make_request_to_ai(&mut self) -> String {
        let question = self.ai_question_value.clone();
        let response = format!("0 0 1 1 *: {}:", question);
        response
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

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if self.enabled {
            match key.code {
                KeyCode::Tab => {}
                KeyCode::Enter => {}
                KeyCode::Up => {
                    if self.current_input == ActiveInput::AIQuestion {
                        self.cron_notation_value = self.make_request_to_ai();
                        self.cron_notation.delete_line_by_head();
                        self.cron_notation.delete_line_by_end();
                        self.cron_notation.insert_str(&self.cron_notation_value);
                    }
                }
                _ => match self.current_input {
                    ActiveInput::CronNotation => {
                        let cron_input = &mut self.cron_notation;
                        let cron_value = &mut self.cron_notation_value;
                        if cron_input.input(key) {
                            cron_value.clear();
                            if let Some(first_line) = cron_input.lines().first() {
                                cron_value.push_str(first_line);
                            }
                        }
                    }
                    ActiveInput::Job => {
                        let job_input = &mut self.job;
                        let job_value = &mut self.job_value;
                        if job_input.input(key) {
                            job_value.clear();
                            if let Some(first_line) = job_input.lines().first() {
                                job_value.push_str(first_line);
                            }
                        }
                    }
                    ActiveInput::JobDescription => {
                        let job_description_input = &mut self.job_description;
                        let job_description_value = &mut self.job_description_value;
                        if job_description_input.input(key) {
                            job_description_value.clear();
                            if let Some(first_line) = job_description_input.lines().first() {
                                job_description_value.push_str(first_line);
                            }
                        }
                    }
                    ActiveInput::AIQuestion => {
                        let ai_question_input = &mut self.ai_question;
                        let ai_question_value = &mut self.ai_question_value;
                        if ai_question_input.input(key) {
                            ai_question_value.clear();
                            if let Some(first_line) = ai_question_input.lines().first() {
                                ai_question_value.push_str(first_line);
                            }
                        }
                    }
                },
            }
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::ChangeMode(Module::CronPopup) = action {
            self.enabled = true;
        }
        if let Action::PassData(ref cron) = action {
            self.initial_render();
            if !cron.is_empty() {
                self.is_new = false;
                self.index = cron[0].parse().unwrap();
                self.cron_notation_value = cron[1].clone();
                self.job_value = cron[2].clone();
                self.job_description_value = cron[3].clone();

                self.cron_notation.insert_str(&self.cron_notation_value);
                self.job.insert_str(&self.job_value);
                self.job_description.insert_str(&self.job_description_value);
            } else {
                self.is_new = true;
            }
        }
        if self.enabled {
            match action {
                Action::ChangeMode(Module::Cron) => {
                    self.flash_inputs();
                    self.flash_values();
                    self.enabled = false;
                }
                Action::SwitchElement => {
                    self.current_input = self.current_input.next();
                }
                Action::Confirm => match validate(&mut self.cron_notation) {
                    Ok(_) => {
                        let tx = self.command_tx.clone().unwrap();
                        tx.send(Action::PassData(vec![
                            self.index.to_string(),
                            self.cron_notation_value.clone(),
                            self.job_value.clone(),
                            self.job_description_value.clone(),
                        ]))
                        .unwrap();
                        self.enabled = false;
                        return Ok(Some(Action::ChangeMode(Module::Cron)));
                    }
                    Err(ValidationError::InvalidCronExpression(_)) => {
                        self.current_input = ActiveInput::CronNotation;
                    }
                },
                _ => {}
            }
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, _area: Rect) -> Result<()> {
        if self.enabled {
            let area = center(
                frame.area(),
                Constraint::Percentage(70),
                Constraint::Length(22),
            );
            frame.render_widget(Clear, area);

            let layout = Layout::vertical([Constraint::Length(20), Constraint::Length(2)])
                .flex(Flex::SpaceBetween);
            let [main_area, footer_area] = layout.areas(area);

            let main_block = Block::default()
                .style(self.styles.window_style)
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .border_style(self.styles.window_border_style);

            frame.render_widget(main_block, main_area);

            let main = Layout::vertical([
                Constraint::Length(4),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .margin(2)
            .flex(Flex::Start);
            let [title, cron_notation, job, description, ai] = main.areas(main_area);

            let footer = Layout::vertical([Constraint::Length(3)]);
            let [help] = footer.areas(footer_area);

            let selected_cron_notation = get_human_readable_cron(self.cron_notation_value.as_str())
                .unwrap_or_else(|e| e.to_string());

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

            let title_widget = Paragraph::new(Text::from(wrapped_text))
                .style(self.styles.title_style)
                .centered()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(self.styles.title_border_style),
                );
            frame.render_widget(title_widget, title);

            self.draw_footer(
                frame,
                help,
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
            let ai_question_input = &mut self.ai_question;

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
                    frame.render_widget(&*cron_input, cron_notation);

                    job_input.set_cursor_style(Style::default());
                    job_input.set_block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(self.styles.unselected_input_border_style)
                            .title("Job"),
                    );
                    frame.render_widget(&*job_input, job);

                    description_input.set_cursor_style(Style::default());
                    description_input.set_block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(self.styles.unselected_input_border_style)
                            .title("Description"),
                    );
                    frame.render_widget(&*description_input, description);

                    ai_question_input.set_cursor_style(Style::default());
                    ai_question_input.set_block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(self.styles.unselected_input_border_style)
                            .title("Ask AI"),
                    );
                    frame.render_widget(&*ai_question_input, ai);
                }
                ActiveInput::Job => {
                    job_input.set_cursor_style(self.styles.cursor_style);
                    job_input.set_block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(self.styles.selected_input_border_style)
                            .title("Job"),
                    );
                    frame.render_widget(&*job_input, job);

                    cron_input.set_cursor_style(Style::default());
                    cron_input.set_block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(self.styles.unselected_input_border_style)
                            .title("Cron notation*"),
                    );
                    frame.render_widget(&*cron_input, cron_notation);

                    description_input.set_cursor_style(Style::default());
                    description_input.set_block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(self.styles.unselected_input_border_style)
                            .title("Description"),
                    );
                    frame.render_widget(&*description_input, description);

                    ai_question_input.set_cursor_style(Style::default());
                    ai_question_input.set_block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(self.styles.unselected_input_border_style)
                            .title("Ask AI"),
                    );
                    frame.render_widget(&*ai_question_input, ai);
                }
                ActiveInput::JobDescription => {
                    description_input.set_cursor_style(self.styles.cursor_style);
                    description_input.set_block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(self.styles.selected_input_border_style)
                            .title("Description"),
                    );
                    frame.render_widget(&*description_input, description);

                    cron_input.set_cursor_style(Style::default());
                    cron_input.set_block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(self.styles.unselected_input_border_style)
                            .title("Cron notation*"),
                    );
                    frame.render_widget(&*cron_input, cron_notation);

                    job_input.set_cursor_style(Style::default());
                    job_input.set_block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(self.styles.unselected_input_border_style)
                            .title("Job"),
                    );
                    frame.render_widget(&*job_input, job);

                    ai_question_input.set_cursor_style(Style::default());
                    ai_question_input.set_block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(self.styles.unselected_input_border_style)
                            .title("Ask AI"),
                    );
                    frame.render_widget(&*ai_question_input, ai);
                }
                ActiveInput::AIQuestion => {
                    ai_question_input.set_cursor_style(self.styles.cursor_style);
                    ai_question_input.set_block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(self.styles.selected_input_border_style)
                            .title("Ask AI"),
                    );
                    frame.render_widget(&*ai_question_input, ai);

                    cron_input.set_cursor_style(Style::default());
                    cron_input.set_block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(self.styles.unselected_input_border_style)
                            .title("Cron notation*"),
                    );
                    frame.render_widget(&*cron_input, cron_notation);

                    job_input.set_cursor_style(Style::default());
                    job_input.set_block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(self.styles.unselected_input_border_style)
                            .title("Job"),
                    );
                    frame.render_widget(&*job_input, job);

                    description_input.set_cursor_style(Style::default());
                    description_input.set_block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(self.styles.unselected_input_border_style)
                            .title("Description"),
                    );
                    frame.render_widget(&*description_input, description);
                }
            }
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

pub enum ValidationError {
    InvalidCronExpression(String),
}

fn validate(textarea: &mut TextArea) -> Result<(), ValidationError> {
    use chrono::Utc;
    use cron_parser::parse;

    let input = textarea
        .lines()
        .first()
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
