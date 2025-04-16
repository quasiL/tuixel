pub mod utils;

use color_eyre::Result;
use ratatui::{
    crossterm::event::{MouseEvent, MouseEventKind},
    layout::{Constraint, Layout, Margin, Rect},
    prelude::Frame,
    text::Text,
    widgets::{
        Cell, HighlightSpacing, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
        TableState,
    },
};
use tokio::sync::mpsc::UnboundedSender;
use tracing::error;

use super::Component;
use crate::{
    action::{Action, Module},
    config::Config,
    draw::Drawable,
    style::TableStyles,
};
use utils::{constraint_len_calculator, from_crontab, get_next_execution, save_to_crontab};

impl Drawable for Cron {}
const ITEM_HEIGHT: usize = 3;

#[derive(Default)]
pub struct Cron {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    enabled: bool,
    mouse: bool,
    state: TableState,
    items: Vec<CronJob>,
    longest_item_lens: (u16, u16, u16),
    scroll_state: ScrollbarState,
    styles: TableStyles,
}

#[derive(Default)]
pub struct CronJob {
    pub cron_notation: String,
    pub job: String,
    pub job_description: String,
    pub next_execution: String,
}

impl CronJob {
    const fn ref_array(&self) -> [&String; 3] {
        [
            &self.cron_notation,
            &self.next_execution,
            &self.job_description,
        ]
    }

    pub fn new(cron_job: CronJob) -> Self {
        Self {
            cron_notation: cron_job.cron_notation,
            job: cron_job.job,
            job_description: cron_job.job_description,
            next_execution: cron_job.next_execution,
        }
    }
}

impl Cron {
    pub fn new() -> Self {
        let cron_jobs_vec = vec![];
        let scroll_position = if cron_jobs_vec.is_empty() {
            0
        } else {
            (cron_jobs_vec.len() - 1) * ITEM_HEIGHT
        };
        Self {
            command_tx: None,
            config: Config::default(),
            enabled: false,
            mouse: true,
            state: TableState::default().with_selected(0),
            longest_item_lens: constraint_len_calculator(&cron_jobs_vec),
            scroll_state: ScrollbarState::new(scroll_position),
            styles: TableStyles::new(),
            items: cron_jobs_vec,
        }
    }

    fn draw_table(&mut self, frame: &mut Frame, area: Rect) {
        let header = ["Cron Notation", "Next Execution", "Description"]
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
        if self.longest_item_lens.0 < "Cron Notation".len() as u16 {
            self.longest_item_lens.0 = "Cron Notation".len() as u16;
        }
        let table = Table::new(
            rows,
            [
                Constraint::Length(self.longest_item_lens.0 + 8),
                Constraint::Min(self.longest_item_lens.1 + 1),
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

impl Component for Cron {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        if self.enabled && self.mouse {
            let start_row: u16 = 3;
            let row_height: u16 = ITEM_HEIGHT as u16;

            let table_height = self.items.len();

            if mouse.row >= start_row && mouse.row < start_row + (table_height as u16 * row_height)
            {
                let selected_index = ((mouse.row - start_row) / row_height) as usize;

                if selected_index < self.items.len() {
                    self.state.select(Some(selected_index));

                    if let MouseEventKind::Up(_) = mouse.kind {
                        let tx = self.command_tx.clone().unwrap();
                        tx.send(Action::Select).unwrap();
                        self.mouse = false;
                    }
                }
            }
        }

        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::ChangeMode(Module::Cron) = action {
            self.mouse = true;
            let cron_jobs_vec =
                from_crontab(&self.config.settings.cron.timezone).unwrap_or_else(|err| {
                    tracing::error!("Error reading crontab: {}", err);
                    vec![CronJob {
                        cron_notation: format!("Error: {}", err),
                        job: String::new(),
                        job_description: String::new(),
                        next_execution: String::new(),
                    }]
                });
            let scroll_position = if cron_jobs_vec.is_empty() {
                0
            } else {
                (cron_jobs_vec.len() - 1) * ITEM_HEIGHT
            };
            self.items = cron_jobs_vec;
            self.scroll_state = ScrollbarState::new(scroll_position);
            self.enabled = true;
        }
        if let Action::PassData(ref cron) = action {
            if !cron.is_empty() {
                let index: i32 = cron[0].parse().unwrap();
                if index == -1 {
                    self.items.push(CronJob::new(CronJob {
                        cron_notation: cron[1].clone(),
                        job: cron[2].clone(),
                        job_description: cron[3].clone(),
                        next_execution: get_next_execution(
                            &cron[1],
                            &self.config.settings.cron.timezone,
                        ),
                    }));
                    save_to_crontab(&self.items).unwrap_or_else(|err| {
                        error!("Error saving to crontab: {}", err);
                    });
                } else {
                    self.items[index as usize].cron_notation = cron[1].clone();
                    self.items[index as usize].job = cron[2].clone();
                    self.items[index as usize].job_description = cron[3].clone();
                    self.items[index as usize].next_execution =
                        get_next_execution(&cron[1], &self.config.settings.cron.timezone);
                    save_to_crontab(&self.items).unwrap_or_else(|err| {
                        error!("Error saving to crontab: {}", err);
                    });
                }
            }
        }
        if self.enabled {
            let tx = self.command_tx.clone().unwrap();
            match action {
                Action::ChangeMode(Module::Home) => {
                    self.enabled = false;
                    return Ok(Some(Action::ClearScreen));
                }
                Action::NewRecord => {
                    tx.send(Action::PassData(vec![])).unwrap();
                    self.mouse = false;
                    return Ok(Some(Action::ChangeMode(Module::CronPopup)));
                }
                Action::DeleteRecord => {
                    let index = self.state.selected().unwrap();
                    self.items.remove(index);
                    save_to_crontab(&self.items).unwrap_or_else(|err| {
                        error!("Error saving to crontab: {}", err);
                    });
                }
                Action::Select => {
                    if !self.items.is_empty() {
                        tx.send(Action::PassData(vec![
                            self.state.selected().unwrap().to_string(),
                            self.items[self.state.selected().unwrap()]
                                .cron_notation
                                .to_string(),
                            self.items[self.state.selected().unwrap()].job.to_string(),
                            self.items[self.state.selected().unwrap()]
                                .job_description
                                .to_string(),
                        ]))
                        .unwrap();
                        self.mouse = false;
                        return Ok(Some(Action::ChangeMode(Module::CronPopup)));
                    }
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
                vec![
                    ("<Esc>", "Quit"),
                    ("<Enter>", "Edit selected cron"),
                    ("<↓↑>", "Move up and down"),
                    ("<d>", "Delete selected cron"),
                    ("<n>", "Add new cron"),
                ],
            )?;
        }
        Ok(())
    }
}
