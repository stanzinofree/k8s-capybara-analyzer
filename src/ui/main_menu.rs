use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;

pub fn draw_main_menu(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(8),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("🐹 ", Style::default().fg(Color::Yellow)),
            Span::styled(
                "CAPYBARA HACKER",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![Span::styled(
            "Kubernetes Cluster Analyzer",
            Style::default().fg(Color::Green),
        )]),
        Line::from(vec![Span::styled(
            "Created with ❤️  by Alessandro Middei",
            Style::default().fg(Color::Yellow),
        )]),
        Line::from(vec![Span::styled(
            "Version 1.0 - Rust Edition",
            Style::default().fg(Color::Magenta),
        )]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(title, chunks[0]);

    // Menu items
    let menu_items = app.get_main_menu_items();
    let items: Vec<ListItem> = menu_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if Some(i) == app.list_state.selected() {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(*item).style(style)
        })
        .collect();

    let menu = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Main Menu"))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        );
    f.render_stateful_widget(menu, chunks[1], &mut app.list_state);

    // Instructions
    let instructions =
        Paragraph::new("Use ↑↓ to navigate, Enter to select, ESC to go back, q to quit")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, chunks[2]);
}
