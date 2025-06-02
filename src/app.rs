use ratatui::widgets::ListState;

use crate::error::Result;
use crate::kubernetes::{
    load_deployments, load_namespaces, load_pods, ClusterAnalysis, DeploymentInfo, NamespaceInfo,
    PodInfo,
};
use crate::logs::ComponentLogs;

#[derive(Debug, PartialEq)]
pub enum Screen {
    MainMenu,
    NamespaceList,
    NamespaceDetails,
    ClusterAnalysis,
    ComponentDetails,
    LogsList,
    LogsViewer,
    Capybara,
    PodsList,        // New screen for pod selection
    DeploymentsList, // New screen for deployment selection
}

pub struct App {
    pub namespaces: Vec<NamespaceInfo>,
    pub current_screen: Screen,
    pub list_state: ListState,
    pub logs_scroll_state: ListState,
    pub details_scroll_state: ListState,
    pub selected_namespace: Option<String>,
    pub selected_component: Option<(String, String)>, // (component_name, component_type)
    pub pods: Vec<PodInfo>,
    pub deployments: Vec<DeploymentInfo>,
    pub current_logs: Option<ComponentLogs>,
    pub cluster_analysis: Option<ClusterAnalysis>,
    pub log_filter: Option<String>,
    pub show_capybara: bool,
}

impl App {
    pub fn new() -> Result<App> {
        let namespaces = load_namespaces()?;
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        let mut logs_scroll_state = ListState::default();
        logs_scroll_state.select(Some(0));

        let mut details_scroll_state = ListState::default();
        details_scroll_state.select(Some(0));

        Ok(App {
            namespaces,
            current_screen: Screen::MainMenu,
            list_state,
            logs_scroll_state,
            details_scroll_state,
            selected_namespace: None,
            selected_component: None,
            pods: Vec::new(),
            deployments: Vec::new(),
            current_logs: None,
            cluster_analysis: None,
            log_filter: None,
            show_capybara: false,
        })
    }

    pub fn next(&mut self) {
        let len = self.get_list_length();
        if len == 0 {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => (i + 1) % len,
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let len = self.get_list_length();
        if len == 0 {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => (i + len - 1) % len,
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn scroll_logs_down(&mut self) {
        if let Some(ref logs) = self.current_logs {
            let len = logs.entries.len();
            if len == 0 {
                return;
            }

            let i = match self.logs_scroll_state.selected() {
                Some(i) => (i + 1) % len,
                None => 0,
            };
            self.logs_scroll_state.select(Some(i));
        }
    }

    pub fn scroll_logs_up(&mut self) {
        if let Some(ref logs) = self.current_logs {
            let len = logs.entries.len();
            if len == 0 {
                return;
            }

            let i = match self.logs_scroll_state.selected() {
                Some(i) => (i + len - 1) % len,
                None => 0,
            };
            self.logs_scroll_state.select(Some(i));
        }
    }

    pub fn scroll_details_down(&mut self) {
        // For component details scrolling
        if let Some(analysis) = &self.cluster_analysis {
            let mut total_items = 0;
            for ns_analysis in &analysis.namespaces {
                total_items += ns_analysis.pods.len() + ns_analysis.deployments.len();
            }

            if total_items == 0 {
                return;
            }

            let i = match self.details_scroll_state.selected() {
                Some(i) => (i + 1) % total_items,
                None => 0,
            };
            self.details_scroll_state.select(Some(i));
        }
    }

    pub fn scroll_details_up(&mut self) {
        if let Some(analysis) = &self.cluster_analysis {
            let mut total_items = 0;
            for ns_analysis in &analysis.namespaces {
                total_items += ns_analysis.pods.len() + ns_analysis.deployments.len();
            }

            if total_items == 0 {
                return;
            }

            let i = match self.details_scroll_state.selected() {
                Some(i) => (i + total_items - 1) % total_items,
                None => 0,
            };
            self.details_scroll_state.select(Some(i));
        }
    }

    pub fn toggle_log_filter(&mut self, filter: &str) {
        match self.log_filter.as_deref() {
            Some(current) if current == filter => {
                self.log_filter = None; // Remove filter if same
            }
            _ => {
                self.log_filter = Some(filter.to_string());
            }
        }
        self.logs_scroll_state.select(Some(0)); // Reset scroll position
    }

    pub fn select(&mut self) -> Result<()> {
        match self.current_screen {
            Screen::MainMenu => self.handle_main_menu_selection(),
            Screen::NamespaceList => self.handle_namespace_selection(),
            Screen::NamespaceDetails => self.handle_namespace_details_selection(),
            Screen::ClusterAnalysis => self.handle_cluster_analysis_selection(),
            Screen::LogsList => self.handle_logs_list_selection(),
            Screen::PodsList => self.handle_pods_list_selection(),
            Screen::DeploymentsList => self.handle_deployments_list_selection(),
            _ => Ok(()),
        }
    }

    pub fn back(&mut self) {
        match self.current_screen {
            Screen::NamespaceList | Screen::ClusterAnalysis | Screen::Capybara => {
                self.current_screen = Screen::MainMenu;
                self.show_capybara = false;
            }
            Screen::NamespaceDetails => {
                self.current_screen = Screen::NamespaceList;
            }
            Screen::PodsList | Screen::DeploymentsList => {
                self.current_screen = Screen::NamespaceDetails;
            }
            Screen::ComponentDetails => {
                // Go back to the appropriate screen based on how we got here
                match self.selected_component {
                    Some((_, ref comp_type)) => match comp_type.as_str() {
                        "Pod" => self.current_screen = Screen::PodsList,
                        "Deployment" => self.current_screen = Screen::DeploymentsList,
                        _ => self.current_screen = Screen::ClusterAnalysis,
                    },
                    None => self.current_screen = Screen::ClusterAnalysis,
                }
                self.selected_component = None;
            }
            Screen::LogsList => {
                self.current_screen = Screen::NamespaceDetails;
            }
            Screen::LogsViewer => {
                self.current_screen = Screen::LogsList;
                self.current_logs = None;
                self.log_filter = None;
            }
            _ => {}
        }
        self.list_state.select(Some(0));
        self.logs_scroll_state.select(Some(0));
        self.details_scroll_state.select(Some(0));
    }

    fn get_list_length(&self) -> usize {
        match self.current_screen {
            Screen::MainMenu => 5, // Number of main menu items
            Screen::NamespaceList => self.namespaces.len(),
            Screen::NamespaceDetails => 3, // Pods, Deployments, Logs
            Screen::PodsList => self.pods.len(),
            Screen::DeploymentsList => self.deployments.len(),
            Screen::ClusterAnalysis => {
                // Count ONLY selectable components (pods and deployments)
                if let Some(ref analysis) = self.cluster_analysis {
                    let mut count = 0;
                    for ns_analysis in &analysis.namespaces {
                        count += ns_analysis.pods.len() + ns_analysis.deployments.len();
                    }
                    count
                } else {
                    0
                }
            }
            Screen::LogsList => {
                // Count actual log directories in the namespace
                if let Some(ref namespace) = self.selected_namespace {
                    let namespace_dir = format!("output/{}", namespace);
                    if let Ok(entries) = std::fs::read_dir(&namespace_dir) {
                        return entries
                            .flatten()
                            .filter(|entry| {
                                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                                    let component_path = entry.path();
                                    [
                                        component_path.join("logs.txt"),
                                        component_path.join("log.txt"),
                                        component_path.join("logs.json"),
                                    ]
                                    .iter()
                                    .any(|path| path.exists())
                                } else {
                                    false
                                }
                            })
                            .count();
                    }
                }
                0
            }
            _ => 0,
        }
    }

    fn handle_main_menu_selection(&mut self) -> Result<()> {
        if let Some(selected) = self.list_state.selected() {
            match selected {
                0 => {
                    // Load cluster analysis when entering
                    self.cluster_analysis = Some(crate::kubernetes::analyze_cluster()?);
                    self.current_screen = Screen::ClusterAnalysis;
                }
                1 => self.current_screen = Screen::NamespaceList,
                2 => {
                    self.current_screen = Screen::Capybara;
                    self.show_capybara = true;
                }
                3 => {}                         // Help - do nothing for now
                4 => return Err("exit".into()), // Exit
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_namespace_selection(&mut self) -> Result<()> {
        if let Some(selected) = self.list_state.selected() {
            if selected < self.namespaces.len() {
                let namespace = &self.namespaces[selected];
                self.selected_namespace = Some(namespace.name.clone());

                // Load namespace details
                self.pods = load_pods(&namespace.name)?;
                self.deployments = load_deployments(&namespace.name)?;

                self.current_screen = Screen::NamespaceDetails;
            }
        }
        Ok(())
    }

    fn handle_namespace_details_selection(&mut self) -> Result<()> {
        if let Some(selected) = self.list_state.selected() {
            match selected {
                0 => {
                    // View Pods
                    self.current_screen = Screen::PodsList;
                }
                1 => {
                    // View Deployments
                    self.current_screen = Screen::DeploymentsList;
                }
                2 => {
                    // View Logs
                    self.current_screen = Screen::LogsList;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_pods_list_selection(&mut self) -> Result<()> {
        if let Some(selected) = self.list_state.selected() {
            if selected < self.pods.len() {
                let pod = &self.pods[selected];
                self.selected_component = Some((pod.name.clone(), "Pod".to_string()));
                self.current_screen = Screen::ComponentDetails;
            }
        }
        Ok(())
    }

    fn handle_deployments_list_selection(&mut self) -> Result<()> {
        if let Some(selected) = self.list_state.selected() {
            if selected < self.deployments.len() {
                let deployment = &self.deployments[selected];
                self.selected_component = Some((deployment.name.clone(), "Deployment".to_string()));
                self.current_screen = Screen::ComponentDetails;
            }
        }
        Ok(())
    }

    fn handle_cluster_analysis_selection(&mut self) -> Result<()> {
        if let Some(selected) = self.list_state.selected() {
            if let Some(ref analysis) = self.cluster_analysis {
                let mut all_components = Vec::new();

                // Collect all components from all namespaces
                for ns_analysis in &analysis.namespaces {
                    // Add pods
                    for pod in &ns_analysis.pods {
                        all_components.push((pod.name.clone(), "Pod".to_string()));
                    }
                    // Add deployments
                    for deployment in &ns_analysis.deployments {
                        all_components.push((deployment.name.clone(), "Deployment".to_string()));
                    }
                }

                if selected < all_components.len() {
                    let (component_name, component_type) = &all_components[selected];
                    self.selected_component =
                        Some((component_name.clone(), component_type.clone()));
                    self.current_screen = Screen::ComponentDetails;
                }
            }
        }
        Ok(())
    }

    fn handle_logs_list_selection(&mut self) -> Result<()> {
        if let Some(selected) = self.list_state.selected() {
            if let Some(ref namespace) = self.selected_namespace {
                // Get list of actual log directories
                let namespace_dir = format!("output/{}", namespace);
                let mut log_components = Vec::new();

                if let Ok(entries) = std::fs::read_dir(&namespace_dir) {
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
                                log_components.push(component_name);
                            }
                        }
                    }
                }

                if selected < log_components.len() {
                    let component_name = &log_components[selected];
                    self.selected_component =
                        Some((component_name.clone(), "Component".to_string()));

                    // Load component logs using the updated function
                    match crate::logs::load_pod_logs(namespace, component_name) {
                        Ok(logs) => {
                            self.current_logs = Some(logs);
                            self.current_screen = Screen::LogsViewer;
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: Could not load logs for component {}: {}",
                                component_name, e
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn get_main_menu_items(&self) -> Vec<&'static str> {
        vec![
            "🔍 Cluster Analysis",
            "📁 Browse Namespaces",
            "🐹 Capybara Easter Egg",
            "❓ Help",
            "🚪 Exit",
        ]
    }

    pub fn get_namespace_details_items(&self) -> Vec<String> {
        let pod_count = self.pods.len();
        let deployment_count = self.deployments.len();

        vec![
            format!("📦 View Pods ({})", pod_count),
            format!("🚀 View Deployments ({})", deployment_count),
            format!("📋 View Logs"),
        ]
    }
}
