use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::Path;

mod app;
mod error;
mod kubernetes;
mod logs;
mod ui;

use app::{App, Screen};
use error::Result;

fn main() -> Result<()> {
    // Check if output directory exists
    if !Path::new("output").exists() {
        eprintln!("❌ Error: 'output' directory not found!");
        eprintln!("Please ensure you have the Kubernetes cluster dump in the 'output' directory.");
        eprintln!("Expected structure:");
        eprintln!("output/");
        eprintln!("├── namespace1/");
        eprintln!("│   ├── pods.json");
        eprintln!("│   ├── deployments.json");
        eprintln!("│   └── ...");
        eprintln!("└── namespace2/");
        std::process::exit(1);
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new()?;

    // Main loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    match result {
        Ok(_) => {
            println!("👋 Thanks for using Capybara Hacker Kubernetes Analyzer!");
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Esc => {
                    if app.current_screen == Screen::MainMenu {
                        break;
                    }
                    app.back();
                }
                KeyCode::Down => {
                    if app.current_screen == Screen::LogsViewer {
                        app.scroll_logs_down();
                    } else if app.current_screen == Screen::ComponentDetails {
                        app.scroll_details_down();
                    } else {
                        app.next();
                    }
                }
                KeyCode::Up => {
                    if app.current_screen == Screen::LogsViewer {
                        app.scroll_logs_up();
                    } else if app.current_screen == Screen::ComponentDetails {
                        app.scroll_details_up();
                    } else {
                        app.previous();
                    }
                }
                KeyCode::Enter => {
                    if let Err(e) = app.select() {
                        if e.to_string() == "exit" {
                            break;
                        }
                    }
                }
                KeyCode::Char('l') => {
                    // Open logs from component details
                    if app.current_screen == Screen::ComponentDetails {
                        if let Some((ref name, ref comp_type)) = app.selected_component.clone() {
                            // Find the namespace for this component
                            if let Some(ref analysis) = app.cluster_analysis {
                                for ns_analysis in &analysis.namespaces {
                                    let found = match comp_type.as_str() {
                                        "Pod" => ns_analysis.pods.iter().any(|p| p.name == *name),
                                        "Deployment" => {
                                            ns_analysis.deployments.iter().any(|d| d.name == *name)
                                        }
                                        _ => false,
                                    };

                                    if found {
                                        // Try to load logs for this component
                                        match crate::logs::load_pod_logs(&ns_analysis.name, name) {
                                            Ok(logs) => {
                                                app.current_logs = Some(logs);
                                                app.current_screen = Screen::LogsViewer;
                                                app.selected_namespace =
                                                    Some(ns_analysis.name.clone());
                                            }
                                            Err(_) => {
                                                // Could show error message, for now just ignore
                                            }
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                // Log filtering shortcuts
                KeyCode::Char('f') => {
                    if app.current_screen == Screen::LogsViewer {
                        // Cycle through filters: all -> error -> warning -> info -> debug -> all
                        match app.log_filter.as_deref() {
                            None => app.toggle_log_filter("error"),
                            Some("error") => app.toggle_log_filter("warning"),
                            Some("warning") => app.toggle_log_filter("info"),
                            Some("info") => app.toggle_log_filter("debug"),
                            Some("debug") => app.log_filter = None,
                            _ => app.log_filter = None,
                        }
                    }
                }
                KeyCode::Char('e') => {
                    if app.current_screen == Screen::LogsViewer {
                        app.toggle_log_filter("error");
                    }
                }
                KeyCode::Char('w') => {
                    if app.current_screen == Screen::LogsViewer {
                        app.toggle_log_filter("warning");
                    }
                }
                KeyCode::Char('i') => {
                    if app.current_screen == Screen::LogsViewer {
                        app.toggle_log_filter("info");
                    }
                }
                KeyCode::Char('d') => {
                    if app.current_screen == Screen::LogsViewer {
                        app.toggle_log_filter("debug");
                    }
                }
                KeyCode::Char('a') => {
                    if app.current_screen == Screen::LogsViewer {
                        app.log_filter = None; // Show all
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}
