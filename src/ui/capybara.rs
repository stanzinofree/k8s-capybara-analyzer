use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn draw_capybara(f: &mut Frame, _app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Min(0)])
        .split(f.size());

    let capybara_art = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "🎩 CAPYBARA HACKER 🐹",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("           ░░░░░░░░░░░░░░░░░░░░░░"),
        Line::from("         ░░                    ░░"),
        Line::from("       ░░  ●●              ●●    ░░"),
        Line::from("     ░░      ┌────────────┐        ░░"),
        Line::from("   ░░        │ > kubectl  │          ░░"),
        Line::from("   ░░        │ > analyze  │          ░░"),
        Line::from("   ░░        │ > hack...  │          ░░"),
        Line::from("   ░░        └────────────┘          ░░"),
        Line::from("     ░░                            ░░"),
        Line::from("       ░░  ∩━━━━━━━━━━━━━━━━━━━━∩  ░░"),
        Line::from("         ░░                      ░░"),
        Line::from("           ░░░░░░░░░░░░░░░░░░░░░░"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "\"In the world of containers and clusters,",
            Style::default().fg(Color::Yellow),
        )]),
        Line::from(vec![Span::styled(
            " even the most complex Kubernetes issues",
            Style::default().fg(Color::Yellow),
        )]),
        Line::from(vec![Span::styled(
            " can be solved with zen-like calm...\"",
            Style::default().fg(Color::Yellow),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "                    - Master Capybara 🧘‍♂️",
            Style::default().fg(Color::Green),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Fun Fact: Capybaras are the world's largest rodents",
            Style::default().fg(Color::Magenta),
        )]),
        Line::from(vec![Span::styled(
            "and are excellent swimmers... just like this tool",
            Style::default().fg(Color::Magenta),
        )]),
        Line::from(vec![Span::styled(
            "navigates through your Kubernetes clusters! 🏊‍♂️",
            Style::default().fg(Color::Magenta),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press ESC to return to the main menu",
            Style::default().fg(Color::Cyan),
        )]),
    ];

    let capybara_widget = Paragraph::new(capybara_art)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🐹 Easter Egg: The Zen of Capybara Hacking 🐹")
                .border_style(Style::default().fg(Color::Yellow)),
        );

    f.render_widget(capybara_widget, chunks[0]);
}
