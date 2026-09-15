#![allow(dead_code)]

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
    Frame,
};

use super::theme::Theme;

pub struct Header<'a> {
    title: &'a str,
    version: &'a str,
    theme: &'a Theme,
}

impl<'a> Header<'a> {
    pub fn new(title: &'a str, version: &'a str, theme: &'a Theme) -> Self {
        Self {
            title,
            version,
            theme,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let header = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let title = Paragraph::new(vec![Line::from(vec![
            Span::styled("🚀 ", self.theme.style_accent()),
            Span::styled(self.title, self.theme.style_bold()),
        ])]);

        let version = Paragraph::new(vec![Line::from(vec![
            Span::styled("v", self.theme.style_dim()),
            Span::styled(self.version, self.theme.style_primary()),
            Span::styled(" | ", self.theme.style_dim()),
            Span::styled("⚡ AI Mode: ON", self.theme.style_success()),
        ])])
        .alignment(ratatui::layout::Alignment::Right);

        frame.render_widget(title, header[0]);
        frame.render_widget(version, header[1]);
    }
}

pub struct StatusBar<'a> {
    items: Vec<StatusItem<'a>>,
    theme: &'a Theme,
}

pub struct StatusItem<'a> {
    pub label: &'a str,
    pub value: &'a str,
    pub style: Style,
}

impl<'a> StatusBar<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        Self {
            items: Vec::new(),
            theme,
        }
    }

    pub fn add_item(mut self, label: &'a str, value: &'a str, style: Style) -> Self {
        self.items.push(StatusItem {
            label,
            value,
            style,
        });
        self
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let mut spans = Vec::new();

        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled(" │ ", self.theme.style_dim()));
            }
            spans.push(Span::styled(
                format!("{}: ", item.label),
                self.theme.style_dim(),
            ));
            spans.push(Span::styled(item.value.to_string(), item.style));
        }

        let status = Paragraph::new(Line::from(spans));
        frame.render_widget(status, area);
    }
}

pub struct InfoBox<'a> {
    title: &'a str,
    content: Vec<Line<'a>>,
    theme: &'a Theme,
    border_color: Color,
}

impl<'a> InfoBox<'a> {
    pub fn new(title: &'a str, theme: &'a Theme) -> Self {
        Self {
            title,
            content: Vec::new(),
            theme,
            border_color: theme.primary,
        }
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    pub fn add_line(mut self, line: Line<'a>) -> Self {
        self.content.push(line);
        self
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(Span::styled(
                format!(" {} ", self.title),
                Style::default()
                    .fg(self.border_color)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.border_color));

        let paragraph = Paragraph::new(self.content.clone())
            .block(block)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }
}

pub struct TabsWidget<'a> {
    titles: Vec<&'a str>,
    selected: usize,
    theme: &'a Theme,
}

impl<'a> TabsWidget<'a> {
    pub fn new(titles: Vec<&'a str>, selected: usize, theme: &'a Theme) -> Self {
        Self {
            titles,
            selected,
            theme,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let tabs = Tabs::new(self.titles.clone())
            .block(Block::default().borders(Borders::BOTTOM))
            .select(self.selected)
            .style(self.theme.style_dim())
            .highlight_style(
                Style::default()
                    .fg(self.theme.accent)
                    .add_modifier(Modifier::BOLD),
            )
            .divider("│");

        frame.render_widget(tabs, area);
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
