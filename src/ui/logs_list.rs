use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::fs;

use crate::app::App;

pub fn draw_logs_list(f: &mut Frame, app: &mut App) {
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
        format!("📋 Available Logs in Namespace: {}", ns)
    } else {
        "📋 Available Logs".to_string()
    };

    let title_widget = Paragraph::new(title)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title_widget, chunks[0]);

    // Scan directory for actual log sources
    let mut log_sources = Vec::new();

    if let Some(ref namespace) = app.selected_namespace {
        let namespace_dir = format!("output/{}", namespace);

        // Scan for directories with logs
        if let Ok(entries) = fs::read_dir(&namespace_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let component_name = entry.file_name().to_string_lossy().to_string();
                    let component_path = entry.path();

                    // Check if this directory has logs
                    let has_logs = [
                        component_path.join("logs.txt"),
                        component_path.join("log.txt"),
                        component_path.join("logs.json"),
                    ]
                    .iter()
                    .any(|path| path.exists());

                    if has_logs {
                        // Determine component type based on name patterns
                        let component_type = determine_component_type(&component_name);
                        let status_icon = "📄"; // Default log icon

                        log_sources.push(format!(
                            "{} {}: {}",
                            status_icon, component_type, component_name
                        ));
                    }
                }
            }
        }

        // If no log directories found, check for direct log files
        if log_sources.is_empty() {
            if let Ok(entries) = fs::read_dir(&namespace_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(extension) = path.extension() {
                        if extension == "txt" || extension == "log" || extension == "json" {
                            if let Some(stem) = path.file_stem() {
                                let file_name = stem.to_string_lossy().to_string();
                                log_sources.push(format!("📄 Log File: {}", file_name));
                            }
                        }
                    }
                }
            }
        }
    }

    // If no sources, show helpful message
    if log_sources.is_empty() {
        log_sources.push("No log sources found in this namespace".to_string());
        log_sources.push("".to_string());
        log_sources.push("Expected log structure:".to_string());
        log_sources.push("  output/{namespace}/{component}/logs.txt".to_string());
        log_sources.push("  output/{namespace}/{component}/log.txt".to_string());
        log_sources.push("".to_string());
        if let Some(ref ns) = app.selected_namespace {
            log_sources.push(format!("Checked directory: output/{}/", ns));
        }
    }

    let items: Vec<ListItem> = log_sources
        .iter()
        .enumerate()
        .map(|(i, source)| {
            let style = if Some(i) == app.list_state.selected() {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(source.as_str()).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Select Component to View Logs"),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        );
    f.render_stateful_widget(list, chunks[1], &mut app.list_state);

    // Instructions
    let instructions = Paragraph::new("↑↓ Navigate | Enter: View Logs | ESC: Back | q: Quit")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, chunks[2]);
}

fn determine_component_type(component_name: &str) -> &'static str {
    if component_name.contains("pod")
        || component_name.contains("-") && component_name.matches('-').count() >= 2
    {
        "Pod"
    } else if component_name.contains("deploy") {
        "Deployment"
    } else if component_name.contains("service") || component_name.contains("svc") {
        "Service"
    } else if component_name.contains("manager") {
        "Manager"
    } else if component_name.contains("controller") {
        "Controller"
    } else if component_name.contains("operator") {
        "Operator"
    } else {
        "Component"
    }
}
