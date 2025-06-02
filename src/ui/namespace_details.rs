use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;

pub fn draw_namespace_details(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(8),      // Menu options
            Constraint::Percentage(45), // Pods
            Constraint::Percentage(45), // Deployments
        ])
        .split(f.size());

    // Title
    let title = if let Some(ref ns) = app.selected_namespace {
        format!("Namespace: {}", ns)
    } else {
        "Namespace Details".to_string()
    };

    let title_widget = Paragraph::new(title)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title_widget, chunks[0]);

    // Menu options
    let menu_items = app.get_namespace_details_items();
    let menu_list_items: Vec<ListItem> = menu_items
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
            ListItem::new(item.as_str()).style(style)
        })
        .collect();

    let menu = List::new(menu_list_items)
        .block(Block::default().borders(Borders::ALL).title("Actions"))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        );
    f.render_stateful_widget(menu, chunks[1], &mut app.list_state);

    // Pods
    let pod_items: Vec<ListItem> = app
        .pods
        .iter()
        .map(|pod| {
            let color = if pod.status == "Running" && pod.ready {
                Color::Green
            } else {
                Color::Yellow
            };
            ListItem::new(format!(
                "Pod: {} | Status: {} | Ready: {}",
                pod.name, pod.status, pod.ready
            ))
            .style(Style::default().fg(color))
        })
        .collect();

    let pods_list =
        List::new(pod_items).block(Block::default().borders(Borders::ALL).title("Pods"));
    f.render_widget(pods_list, chunks[2]);

    // Deployments
    let deployment_items: Vec<ListItem> = app
        .deployments
        .iter()
        .map(|dep| {
            let color = if dep.ready_replicas == dep.desired_replicas {
                Color::Green
            } else {
                Color::Red
            };
            ListItem::new(format!(
                "Deployment: {} | Replicas: {}/{}",
                dep.name, dep.ready_replicas, dep.desired_replicas
            ))
            .style(Style::default().fg(color))
        })
        .collect();

    let deployments_list = List::new(deployment_items)
        .block(Block::default().borders(Borders::ALL).title("Deployments"));
    f.render_widget(deployments_list, chunks[3]);
}
