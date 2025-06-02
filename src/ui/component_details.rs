use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;
use crate::kubernetes::{DeploymentInfo, PodInfo};

pub fn draw_component_details(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Details content
            Constraint::Length(3), // Controls
        ])
        .split(f.size());

    // Title
    let title = if let Some((ref name, ref comp_type)) = app.selected_component {
        format!("📋 {} Details: {}", comp_type, name)
    } else {
        "Component Details".to_string()
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

    // Details content
    if let Some((ref name, ref comp_type)) = app.selected_component {
        match comp_type.as_str() {
            "Pod" => draw_pod_details(f, chunks[1], app, name),
            "Deployment" => draw_deployment_details(f, chunks[1], app, name),
            _ => draw_generic_details(f, chunks[1], name, comp_type),
        }
    } else {
        let empty = Paragraph::new("No component selected")
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        f.render_widget(empty, chunks[1]);
    }

    // Controls
    let instructions = Paragraph::new("↑↓: Scroll | l: View Logs | ESC: Back | q: Quit")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray));
    f.render_widget(instructions, chunks[2]);
}

fn draw_pod_details(f: &mut Frame, area: ratatui::layout::Rect, app: &App, pod_name: &str) {
    // First try to find in the loaded pods (from namespace browsing)
    if let Some(pod) = app.pods.iter().find(|p| p.name == pod_name) {
        let namespace = app.selected_namespace.as_deref().unwrap_or("unknown");
        let details = create_pod_detail_lines(pod, namespace);
        let mut scroll_state = ratatui::widgets::ListState::default();
        draw_scrollable_details(f, area, details, &mut scroll_state);
        return;
    }

    // Fallback: try to find in cluster analysis data
    if let Some(analysis) = &app.cluster_analysis {
        for namespace_analysis in &analysis.namespaces {
            if let Some(pod) = namespace_analysis.pods.iter().find(|p| p.name == pod_name) {
                let details = create_pod_detail_lines(pod, &namespace_analysis.name);
                let mut scroll_state = ratatui::widgets::ListState::default();
                draw_scrollable_details(f, area, details, &mut scroll_state);
                return;
            }
        }
    }

    // Final fallback
    let details = vec![
        "Pod not found in loaded data".to_string(),
        "".to_string(),
        format!("Searched for pod: {}", pod_name),
        format!("Current namespace: {:?}", app.selected_namespace),
        format!("Loaded pods count: {}", app.pods.len()),
    ];
    let mut scroll_state = ratatui::widgets::ListState::default();
    draw_scrollable_details(f, area, details, &mut scroll_state);
}

fn draw_deployment_details(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    app: &App,
    deployment_name: &str,
) {
    // First try to find in the loaded deployments (from namespace browsing)
    if let Some(deployment) = app.deployments.iter().find(|d| d.name == deployment_name) {
        let namespace = app.selected_namespace.as_deref().unwrap_or("unknown");
        let details = create_deployment_detail_lines(deployment, namespace);
        let mut scroll_state = ratatui::widgets::ListState::default();
        draw_scrollable_details(f, area, details, &mut scroll_state);
        return;
    }

    // Fallback: try to find in cluster analysis data
    if let Some(analysis) = &app.cluster_analysis {
        for namespace_analysis in &analysis.namespaces {
            if let Some(deployment) = namespace_analysis
                .deployments
                .iter()
                .find(|d| d.name == deployment_name)
            {
                let details = create_deployment_detail_lines(deployment, &namespace_analysis.name);
                let mut scroll_state = ratatui::widgets::ListState::default();
                draw_scrollable_details(f, area, details, &mut scroll_state);
                return;
            }
        }
    }

    // Final fallback
    let details = vec![
        "Deployment not found in loaded data".to_string(),
        "".to_string(),
        format!("Searched for deployment: {}", deployment_name),
        format!("Current namespace: {:?}", app.selected_namespace),
        format!("Loaded deployments count: {}", app.deployments.len()),
    ];
    let mut scroll_state = ratatui::widgets::ListState::default();
    draw_scrollable_details(f, area, details, &mut scroll_state);
}

fn draw_generic_details(f: &mut Frame, area: ratatui::layout::Rect, name: &str, comp_type: &str) {
    let details = vec![
        format!("Component Type: {}", comp_type),
        format!("Name: {}", name),
        "".to_string(),
        "No detailed information available for this component type.".to_string(),
    ];

    draw_scrollable_details(
        f,
        area,
        details,
        &mut ratatui::widgets::ListState::default(),
    );
}

fn create_pod_detail_lines(pod: &PodInfo, namespace: &str) -> Vec<String> {
    let mut lines = Vec::new();

    // Header
    lines.push("📦 POD INFORMATION".to_string());
    lines.push("".to_string());

    // Basic info
    lines.push(format!("Name: {}", pod.name));
    lines.push(format!("Namespace: {}", namespace));

    // Status
    let status_icon = if pod.status == "Running" && pod.ready {
        "🟢"
    } else {
        "🔴"
    };

    lines.push(format!("Status: {} {}", status_icon, pod.status));
    lines.push(format!(
        "Ready: {}",
        if pod.ready { "✅ Yes" } else { "❌ No" }
    ));

    // Resource info
    lines.push("".to_string());
    lines.push("🔧 RESOURCE INFORMATION".to_string());

    if let Some(ref cpu) = pod.cpu_usage {
        lines.push(format!("CPU Usage: {}", cpu));
    } else {
        lines.push("CPU Usage: Not available".to_string());
    }

    if let Some(ref memory) = pod.memory_usage {
        lines.push(format!("Memory Usage: {}", memory));
    } else {
        lines.push("Memory Usage: Not available".to_string());
    }

    if let Some(ref restart_count) = pod.restart_count {
        let count: i32 = restart_count.parse().unwrap_or(0);
        let restart_icon = if count > 0 { "⚠️" } else { "✅" };
        lines.push(format!("Restart Count: {} {}", restart_icon, restart_count));
    } else {
        lines.push("Restart Count: Not available".to_string());
    }

    // Container info
    lines.push("".to_string());
    lines.push("📦 CONTAINER INFORMATION".to_string());

    if let Some(ref image) = pod.image {
        lines.push(format!("Image: {}", image));
    } else {
        lines.push("Image: Not available".to_string());
    }

    // Health info
    lines.push("".to_string());
    lines.push("❤️ HEALTH STATUS".to_string());

    if pod.ready && pod.status == "Running" {
        lines.push("Overall Health: 🟢 Healthy".to_string());
        lines.push("Status: Pod is running and ready to serve traffic".to_string());
    } else {
        lines.push("Overall Health: 🔴 Unhealthy".to_string());
        if !pod.ready {
            lines.push("Issue: Pod is not ready to serve traffic".to_string());
        }
        if pod.status != "Running" {
            lines.push(format!(
                "Issue: Pod status is '{}' instead of 'Running'",
                pod.status
            ));
        }
    }

    // Additional debug info
    lines.push("".to_string());
    lines.push("🔍 DEBUG INFORMATION".to_string());
    lines.push(format!("Pod object loaded from namespace: {}", namespace));

    lines
}

fn create_deployment_detail_lines(deployment: &DeploymentInfo, namespace: &str) -> Vec<String> {
    let mut lines = Vec::new();

    // Header
    lines.push("🚀 DEPLOYMENT INFORMATION".to_string());
    lines.push("".to_string());

    // Basic info
    lines.push(format!("Name: {}", deployment.name));
    lines.push(format!("Namespace: {}", namespace));

    // Replica status
    let replica_emoji = if deployment.ready_replicas == deployment.desired_replicas
        && deployment.desired_replicas > 0
    {
        "🟢"
    } else if deployment.desired_replicas == 0 {
        "⚪"
    } else {
        "🔴"
    };

    lines.push(format!(
        "Replicas: {} {}/{}",
        replica_emoji, deployment.ready_replicas, deployment.desired_replicas
    ));

    let availability = if deployment.desired_replicas > 0 {
        let percentage =
            (deployment.ready_replicas as f64 / deployment.desired_replicas as f64) * 100.0;
        format!("Availability: {:.1}%", percentage)
    } else {
        "Availability: N/A (scaled to 0)".to_string()
    };
    lines.push(availability);

    // Strategy
    lines.push("".to_string());
    lines.push("📋 DEPLOYMENT STRATEGY".to_string());

    if let Some(ref strategy) = deployment.strategy {
        lines.push(format!("Update Strategy: {}", strategy));
    } else {
        lines.push("Update Strategy: Not available".to_string());
    }

    // Image info
    lines.push("".to_string());
    lines.push("📦 CONTAINER IMAGE".to_string());
    if let Some(ref image) = deployment.image {
        lines.push(format!("Image: {}", image));

        // Extract image name and tag for better readability
        if let Some(last_slash) = image.rfind('/') {
            let image_name_tag = &image[last_slash + 1..];
            if let Some(colon) = image_name_tag.find(':') {
                let name = &image_name_tag[..colon];
                let tag = &image_name_tag[colon + 1..];
                lines.push(format!("  Image Name: {}", name));
                lines.push(format!("  Tag: {}", tag));
            }
        }
    } else {
        lines.push("Image: Not available".to_string());
    }

    // Health status
    lines.push("".to_string());
    lines.push("❤️ HEALTH STATUS".to_string());

    if deployment.ready_replicas == deployment.desired_replicas && deployment.desired_replicas > 0 {
        lines.push("Overall Health: 🟢 Healthy".to_string());
        lines.push("Status: All replicas are ready and available".to_string());
    } else if deployment.desired_replicas == 0 {
        lines.push("Overall Health: ⚪ Scaled to Zero".to_string());
        lines.push("Status: Deployment is intentionally scaled to 0 replicas".to_string());
    } else {
        lines.push("Overall Health: 🔴 Unhealthy".to_string());
        if deployment.ready_replicas == 0 {
            lines.push("Issue: No replicas are ready (complete outage)".to_string());
        } else {
            lines.push(format!(
                "Issue: Only {}/{} replicas are ready (partial outage)",
                deployment.ready_replicas, deployment.desired_replicas
            ));
        }
    }

    // Additional debug info
    lines.push("".to_string());
    lines.push("🔍 DEBUG INFORMATION".to_string());
    lines.push(format!(
        "Deployment object loaded from namespace: {}",
        namespace
    ));

    lines
}

fn draw_scrollable_details(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    lines: Vec<String>,
    scroll_state: &mut ratatui::widgets::ListState,
) {
    let items: Vec<ListItem> = lines.into_iter().map(|line| ListItem::new(line)).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Details"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_stateful_widget(list, area, scroll_state);
}
