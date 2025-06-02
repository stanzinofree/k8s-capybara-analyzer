use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;

pub fn draw_cluster_analysis(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Analysis content
            Constraint::Length(3), // Instructions
        ])
        .split(f.size());

    // Title
    let title_widget = Paragraph::new("🔍 Cluster Analysis")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(title_widget, chunks[0]);

    // Analysis content
    match &app.cluster_analysis {
        Some(analysis) => {
            let mut display_items = Vec::new();
            let mut selectable_items = Vec::new();

            // Create display items and track selectable ones
            for ns_analysis in &analysis.namespaces {
                // Namespace header (not selectable)
                display_items.push((
                    format!(
                        "📁 Namespace: {} ({} pods, {} deployments)",
                        ns_analysis.name,
                        ns_analysis.pods.len(),
                        ns_analysis.deployments.len()
                    ),
                    false, // not selectable
                ));

                // Pod entries (selectable)
                for pod in &ns_analysis.pods {
                    let status_icon = if pod.ready && pod.status == "Running" {
                        "🟢"
                    } else {
                        "🔴"
                    };
                    display_items.push((
                        format!("  {} Pod: {} ({})", status_icon, pod.name, pod.status),
                        true, // selectable
                    ));
                    selectable_items.push((pod.name.clone(), "Pod".to_string()));
                }

                // Deployment entries (selectable)
                for deployment in &ns_analysis.deployments {
                    let status_icon = if deployment.ready_replicas == deployment.desired_replicas {
                        "🟢"
                    } else {
                        "🟡"
                    };
                    display_items.push((
                        format!(
                            "  {} Deployment: {} ({}/{})",
                            status_icon,
                            deployment.name,
                            deployment.ready_replicas,
                            deployment.desired_replicas
                        ),
                        true, // selectable
                    ));
                    selectable_items.push((deployment.name.clone(), "Deployment".to_string()));
                }

                // Add empty line between namespaces (not selectable)
                if !ns_analysis.pods.is_empty() || !ns_analysis.deployments.is_empty() {
                    display_items.push(("".to_string(), false));
                }
            }

            // Remove last empty line
            if display_items
                .last()
                .map(|(s, _)| s.is_empty())
                .unwrap_or(false)
            {
                display_items.pop();
            }

            // Get current selection
            let selected_index = app.list_state.selected().unwrap_or(0);

            // Create list items with proper highlighting
            let list_items: Vec<ListItem> = display_items
                .iter()
                .enumerate()
                .map(|(display_idx, (item, is_selectable))| {
                    let style = if *is_selectable {
                        // Calculate which selectable item this is
                        let selectable_index =
                            calculate_selectable_index(&display_items, display_idx);

                        if selectable_index == selected_index {
                            // This is the selected item
                            Style::default()
                                .bg(Color::Blue)
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            // This is a selectable but not selected item
                            Style::default().fg(Color::White)
                        }
                    } else {
                        // Namespace headers and empty lines
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::ITALIC)
                    };
                    ListItem::new(item.as_str()).style(style)
                })
                .collect();

            let list = List::new(list_items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Cluster Components (Select to view details)"),
            );

            f.render_widget(list, chunks[1]);
        }
        None => {
            let loading = Paragraph::new("Loading cluster analysis...")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Cluster Analysis"),
                )
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Yellow));
            f.render_widget(loading, chunks[1]);
        }
    }

    // Instructions
    let instructions = Paragraph::new("↑↓ Navigate | Enter: View Details | ESC: Back | q: Quit")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray));
    f.render_widget(instructions, chunks[2]);
}

// Helper function to calculate which selectable item corresponds to a display index
fn calculate_selectable_index(
    display_items: &[(String, bool)],
    target_display_index: usize,
) -> usize {
    let mut selectable_count = 0;
    for (i, (_, is_selectable)) in display_items.iter().enumerate() {
        if i == target_display_index {
            return selectable_count;
        }
        if *is_selectable {
            selectable_count += 1;
        }
    }
    0
}
