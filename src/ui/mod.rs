use crate::app::{App, Screen};
use ratatui::Frame;

mod capybara;
mod cluster_analysis;
mod component_details;
mod deployments_list;
mod logs_list;
mod logs_viewer;
mod main_menu;
mod namespace_details;
mod namespace_list;
mod pods_list;

pub use capybara::draw_capybara;
pub use cluster_analysis::draw_cluster_analysis;
pub use component_details::draw_component_details;
pub use deployments_list::draw_deployments_list;
pub use logs_list::draw_logs_list;
pub use logs_viewer::draw_logs_viewer;
pub use main_menu::draw_main_menu;
pub use namespace_details::draw_namespace_details;
pub use namespace_list::draw_namespace_list;
pub use pods_list::draw_pods_list;

pub fn draw(f: &mut Frame, app: &mut App) {
    match app.current_screen {
        Screen::MainMenu => draw_main_menu(f, app),
        Screen::NamespaceList => draw_namespace_list(f, app),
        Screen::NamespaceDetails => draw_namespace_details(f, app),
        Screen::ClusterAnalysis => draw_cluster_analysis(f, app),
        Screen::ComponentDetails => draw_component_details(f, app),
        Screen::LogsList => draw_logs_list(f, app),
        Screen::LogsViewer => draw_logs_viewer(f, app),
        Screen::Capybara => draw_capybara(f, app),
        Screen::PodsList => draw_pods_list(f, app),
        Screen::DeploymentsList => draw_deployments_list(f, app),
    }
}
