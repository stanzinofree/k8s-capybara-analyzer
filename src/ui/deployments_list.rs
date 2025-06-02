use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;

pub fn draw_deployments_list(f: &mut Frame, app: &mut App) {
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
        format!("🚀 Deployments in Namespace: {}", ns)
    } else {
        "🚀 Deployments List".to_string()
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

    // Deployments list
    if app.deployments.is_empty() {
        let empty_message = Paragraph::new("No deployments found in this namespace")
            .block(Block::default().borders(Borders::ALL).title("Deployments"))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(empty_message, chunks[1]);
    } else {
        let items: Vec<ListItem> = app
            .deployments
            .iter()
            .enumerate()
            .map(|(i, deployment)| {
                let status_icon = if deployment.ready_replicas == deployment.desired_replicas
                    && deployment.desired_replicas > 0
                {
                    "🟢"
                } else if deployment.desired_replicas == 0 {
                    "⚪"
                } else {
                    "🔴"
                };

                let strategy_info = if let Some(ref strategy) = deployment.strategy {
                    format!(" | Strategy: {}", strategy)
                } else {
                    String::new()
                };

                let style = if Some(i) == app.list_state.selected() {
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    let color = if deployment.ready_replicas == deployment.desired_replicas
                        && deployment.desired_replicas > 0
                    {
                        Color::Green
                    } else if deployment.desired_replicas == 0 {
                        Color::Gray
                    } else {
                        Color::Red
                    };
                    Style::default().fg(color)
                };

                let deployment_info = format!(
                    "{} {} | Replicas: {}/{}{}",
                    status_icon,
                    deployment.name,
                    deployment.ready_replicas,
                    deployment.desired_replicas,
                    strategy_info
                );

                ListItem::new(deployment_info).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Select Deployment to view details"),
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
