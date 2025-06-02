use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;

pub fn draw_pods_list(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.size());

    // Title with namespace info
    let title = if let Some(ref ns) = app.selected_namespace {
        format!("📦 Pods in Namespace: {}", ns)
    } else {
        "📦 Pods List".to_string()
    };

    let title_widget = Paragraph::new(title)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(title_widget, chunks[0]);

    // Pods list
    if app.pods.is_empty() {
        let empty_message = Paragraph::new("No pods found in this namespace")
            .block(Block::default().borders(Borders::ALL).title("Pods"))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(empty_message, chunks[1]);
    } else {
        let items: Vec<ListItem> = app
            .pods
            .iter()
            .enumerate()
            .map(|(i, pod)| {
                let status_icon = if pod.ready && pod.status == "Running" {
                    "🟢"
                } else if pod.status == "Running" {
                    "🟡"
                } else {
                    "🔴"
                };

                let restart_info = if let Some(ref restart_count) = pod.restart_count {
                    let count: i32 = restart_count.parse().unwrap_or(0);
                    if count > 0 {
                        format!(" (↻ {})", count)
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                let style = if Some(i) == app.list_state.selected() {
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    let color = if pod.ready && pod.status == "Running" {
                        Color::Green
                    } else if pod.status == "Running" {
                        Color::Yellow
                    } else {
                        Color::Red
                    };
                    Style::default().fg(color)
                };

                let pod_info = format!(
                    "{} {} | Status: {} | Ready: {}{}",
                    status_icon,
                    pod.name,
                    pod.status,
                    if pod.ready { "✅" } else { "❌" },
                    restart_info
                );

                ListItem::new(pod_info).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Select Pod to view details"),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_stateful_widget(list, chunks[1], &mut app.list_state);
    }

    // Instructions
    let instructions = Paragraph::new("↑↓ Navigate | Enter: View Details | ESC: Back | q: Quit")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray));
    f.render_widget(instructions, chunks[2]);
}
