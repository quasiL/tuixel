use color_eyre::Result;
use ratatui::{
    layout::Rect,
    prelude::Frame,
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph},
};

pub trait Drawable {
    fn draw_footer(
        &self,
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
}
