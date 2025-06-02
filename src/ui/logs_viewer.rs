use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;
use crate::logs::{ComponentLogs, LogLevel};

pub fn draw_logs_viewer(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(5), // Header with stats
            Constraint::Min(0),    // Log entries
            Constraint::Length(3), // Controls
        ])
        .split(f.size());

    // Header with component info and stats
    draw_log_header(f, chunks[0], app);

    // Log entries
    draw_log_entries(f, chunks[1], app);

    // Controls
    draw_log_controls(f, chunks[2], app);
}

fn draw_log_header(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Component info (left side)
    let component_info = if let Some(ref logs) = app.current_logs {
        vec![
            Line::from(vec![
                Span::styled("Component: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    &logs.component_name,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Type: ", Style::default().fg(Color::Cyan)),
                Span::styled(&logs.component_type, Style::default().fg(Color::Yellow)),
                Span::styled(" | Namespace: ", Style::default().fg(Color::Cyan)),
                Span::styled(&logs.namespace, Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("Total Entries: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    logs.total_entries.to_string(),
                    Style::default().fg(Color::White),
                ),
            ]),
        ]
    } else {
        vec![Line::from("No logs loaded")]
    };

    let info_widget = Paragraph::new(component_info)
        .block(Block::default().borders(Borders::ALL).title("Log Info"));
    f.render_widget(info_widget, header_chunks[0]);

    // Stats (right side)
    if let Some(ref logs) = app.current_logs {
        draw_log_stats(f, header_chunks[1], logs);
    }
}

fn draw_log_stats(f: &mut Frame, area: ratatui::layout::Rect, logs: &ComponentLogs) {
    let stats_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    let error_count = logs.get_error_count();
    let warning_count = logs.get_warning_count();
    let total = logs.total_entries;

    // Error gauge
    let error_ratio = if total > 0 {
        error_count as f64 / total as f64
    } else {
        0.0
    };
    let error_gauge = Gauge::default()
        .block(Block::default().title(format!("🔴 Errors: {}", error_count)))
        .gauge_style(Style::default().fg(Color::Red))
        .ratio(error_ratio);
    f.render_widget(error_gauge, stats_chunks[0]);

    // Warning gauge
    let warning_ratio = if total > 0 {
        warning_count as f64 / total as f64
    } else {
        0.0
    };
    let warning_gauge = Gauge::default()
        .block(Block::default().title(format!("🟡 Warnings: {}", warning_count)))
        .gauge_style(Style::default().fg(Color::Yellow))
        .ratio(warning_ratio);
    f.render_widget(warning_gauge, stats_chunks[1]);

    // Info/Debug gauge
    let info_count = total - error_count - warning_count;
    let info_ratio = if total > 0 {
        info_count as f64 / total as f64
    } else {
        0.0
    };
    let info_gauge = Gauge::default()
        .block(Block::default().title(format!("🔵 Info/Debug: {}", info_count)))
        .gauge_style(Style::default().fg(Color::Blue))
        .ratio(info_ratio);
    f.render_widget(info_gauge, stats_chunks[2]);
}

fn draw_log_entries(f: &mut Frame, area: ratatui::layout::Rect, app: &mut App) {
    if let Some(ref logs) = app.current_logs {
        // Get filtered logs based on current filter
        let displayed_logs: Vec<&crate::logs::LogEntry> = match &app.log_filter {
            Some(filter) => match filter.as_str() {
                "error" => logs
                    .entries
                    .iter()
                    .filter(|e| matches!(e.level, LogLevel::Error))
                    .collect(),
                "warning" => logs
                    .entries
                    .iter()
                    .filter(|e| matches!(e.level, LogLevel::Warning))
                    .collect(),
                "info" => logs
                    .entries
                    .iter()
                    .filter(|e| matches!(e.level, LogLevel::Info))
                    .collect(),
                "debug" => logs
                    .entries
                    .iter()
                    .filter(|e| matches!(e.level, LogLevel::Debug))
                    .collect(),
                _ => logs.entries.iter().collect(),
            },
            None => logs.entries.iter().collect(),
        };

        let items: Vec<ListItem> = displayed_logs
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let level_color = match entry.level {
                    LogLevel::Error => Color::Red,
                    LogLevel::Warning => Color::Yellow,
                    LogLevel::Info => Color::Blue,
                    LogLevel::Debug => Color::Gray,
                };

                let style = if Some(i) == app.logs_scroll_state.selected() {
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let log_line = format!(
                    "{} [{}] [{}] {}",
                    entry.level.color_code(),
                    truncate_timestamp(&entry.timestamp),
                    entry.source,
                    truncate_message(&entry.message, 100)
                );

                ListItem::new(log_line).style(style.fg(level_color))
            })
            .collect();

        let title = format!("Logs ({} entries)", displayed_logs.len());
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_stateful_widget(list, area, &mut app.logs_scroll_state);
    } else {
        let empty = Paragraph::new("No logs available")
            .block(Block::default().borders(Borders::ALL).title("Logs"))
            .alignment(Alignment::Center);
        f.render_widget(empty, area);
    }
}

fn draw_log_controls(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let filter_info = match &app.log_filter {
        Some(filter) => format!("Filter: {} ", filter),
        None => "Filter: all ".to_string(),
    };

    let controls = format!(
        "{}| ↑↓: Scroll | f: Filter (e/w/i/d/a) | /: Search | ESC: Back | q: Quit",
        filter_info
    );

    let instructions = Paragraph::new(controls)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, area);
}

fn truncate_timestamp(timestamp: &str) -> String {
    if timestamp.len() > 19 {
        // Extract just the date and time part for ISO timestamps
        if let Some(t_pos) = timestamp.find('T') {
            let date_part = &timestamp[..std::cmp::min(10, timestamp.len())];
            let time_part = &timestamp[t_pos + 1..std::cmp::min(t_pos + 9, timestamp.len())];
            format!("{} {}", date_part, time_part)
        } else {
            timestamp[..std::cmp::min(19, timestamp.len())].to_string()
        }
    } else {
        timestamp.to_string()
    }
}

fn truncate_message(message: &str, max_len: usize) -> String {
    if message.len() > max_len {
        format!("{}...", &message[..max_len - 3])
    } else {
        message.to_string()
    }
}
