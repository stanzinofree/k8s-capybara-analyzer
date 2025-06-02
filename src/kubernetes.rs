use serde_json::Value;
use std::fs;
use std::path::Path;

use crate::error::Result;

#[derive(Debug, Clone)]
pub struct NamespaceInfo {
    pub name: String,
    pub pod_count: usize,
    pub deployment_count: usize,
}

#[derive(Debug, Clone)]
pub struct PodInfo {
    pub name: String,
    pub status: String,
    pub ready: bool,
    pub cpu_usage: Option<String>,
    pub memory_usage: Option<String>,
    pub restart_count: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeploymentInfo {
    pub name: String,
    pub ready_replicas: u32,
    pub desired_replicas: u32,
    pub strategy: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ClusterIssue {
    pub severity: IssueSeverity,
    pub component: String,
    pub component_type: String,
    pub namespace: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum IssueSeverity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct ClusterAnalysis {
    pub namespaces: Vec<NamespaceAnalysis>,
    pub total_pods: usize,
    pub total_deployments: usize,
    pub total_issues: usize,
}

#[derive(Debug, Clone)]
pub struct NamespaceAnalysis {
    pub name: String,
    pub pods: Vec<PodInfo>,
    pub deployments: Vec<DeploymentInfo>,
    pub issues: Vec<ClusterIssue>,
}

pub fn load_namespaces() -> Result<Vec<NamespaceInfo>> {
    let mut namespaces = Vec::new();

    let output_dir = Path::new("output");
    if !output_dir.exists() {
        return Err("Output directory not found".into());
    }

    for entry in fs::read_dir(output_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let namespace_name = entry.file_name().to_string_lossy().to_string();

            // Count pods and deployments
            let pods = load_pods(&namespace_name).unwrap_or_default();
            let deployments = load_deployments(&namespace_name).unwrap_or_default();

            namespaces.push(NamespaceInfo {
                name: namespace_name,
                pod_count: pods.len(),
                deployment_count: deployments.len(),
            });
        }
    }

    namespaces.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(namespaces)
}

pub fn load_pods(namespace: &str) -> Result<Vec<PodInfo>> {
    let mut pods = Vec::new();
    let pods_file = format!("output/{}/pods.json", namespace);

    if Path::new(&pods_file).exists() {
        let content = fs::read_to_string(&pods_file)?;
        let pods_data: Value = serde_json::from_str(&content)?;

        if let Some(items) = pods_data["items"].as_array() {
            for pod in items.iter() {
                if let (Some(name), Some(status)) = (
                    pod["metadata"]["name"].as_str(),
                    pod["status"]["phase"].as_str(),
                ) {
                    let ready = pod["status"]["conditions"]
                        .as_array()
                        .and_then(|conditions| conditions.iter().find(|c| c["type"] == "Ready"))
                        .and_then(|ready_condition| ready_condition["status"].as_str())
                        .map(|status| status == "True")
                        .unwrap_or(false);

                    // Extract additional fields
                    let cpu_usage = pod["usage"]["cpu"].as_str().map(|s| s.to_string());
                    let memory_usage = pod["usage"]["memory"].as_str().map(|s| s.to_string());
                    let restart_count = pod["status"]["containerStatuses"]
                        .as_array()
                        .and_then(|containers| containers.get(0))
                        .and_then(|container| container["restartCount"].as_u64())
                        .map(|count| count.to_string());

                    let image = pod["spec"]["containers"]
                        .as_array()
                        .and_then(|containers| containers.get(0))
                        .and_then(|container| container["image"].as_str())
                        .map(|s| s.to_string());

                    pods.push(PodInfo {
                        name: name.to_string(),
                        status: status.to_string(),
                        ready,
                        cpu_usage,
                        memory_usage,
                        restart_count,
                        image,
                    });
                }
            }
        }
    }

    Ok(pods)
}

pub fn load_deployments(namespace: &str) -> Result<Vec<DeploymentInfo>> {
    let mut deployments = Vec::new();
    let deployments_file = format!("output/{}/deployments.json", namespace);

    if Path::new(&deployments_file).exists() {
        let content = fs::read_to_string(&deployments_file)?;
        let deployments_data: Value = serde_json::from_str(&content)?;

        if let Some(items) = deployments_data["items"].as_array() {
            for deployment in items.iter() {
                if let Some(name) = deployment["metadata"]["name"].as_str() {
                    let ready_replicas =
                        deployment["status"]["readyReplicas"].as_u64().unwrap_or(0) as u32;
                    let desired_replicas =
                        deployment["spec"]["replicas"].as_u64().unwrap_or(0) as u32;

                    // Extract additional fields
                    let strategy = deployment["spec"]["strategy"]["type"]
                        .as_str()
                        .map(|s| s.to_string());
                    let image = deployment["spec"]["template"]["spec"]["containers"]
                        .as_array()
                        .and_then(|containers| containers.get(0))
                        .and_then(|container| container["image"].as_str())
                        .map(|s| s.to_string());

                    deployments.push(DeploymentInfo {
                        name: name.to_string(),
                        ready_replicas,
                        desired_replicas,
                        strategy,
                        image,
                    });
                }
            }
        }
    }

    Ok(deployments)
}

pub fn analyze_cluster() -> Result<ClusterAnalysis> {
    let namespaces = load_namespaces()?;
    let mut namespace_analyses = Vec::new();
    let mut total_pods = 0;
    let mut total_deployments = 0;
    let mut total_issues = 0;

    for namespace in &namespaces {
        let pods = load_pods(&namespace.name).unwrap_or_default();
        let deployments = load_deployments(&namespace.name).unwrap_or_default();

        // Analyze issues in this namespace
        let mut issues = Vec::new();

        // Check for pod issues
        for pod in &pods {
            if !pod.ready || pod.status != "Running" {
                issues.push(ClusterIssue {
                    severity: IssueSeverity::Warning,
                    component: pod.name.clone(),
                    component_type: "Pod".to_string(),
                    namespace: namespace.name.clone(),
                    description: format!(
                        "Pod {} is not ready or not running (status: {})",
                        pod.name, pod.status
                    ),
                });
            }
        }

        // Check for deployment issues
        for deployment in &deployments {
            if deployment.ready_replicas != deployment.desired_replicas {
                let severity = if deployment.ready_replicas == 0 {
                    IssueSeverity::Critical
                } else {
                    IssueSeverity::Warning
                };

                issues.push(ClusterIssue {
                    severity,
                    component: deployment.name.clone(),
                    component_type: "Deployment".to_string(),
                    namespace: namespace.name.clone(),
                    description: format!(
                        "Deployment {} has {}/{} replicas ready",
                        deployment.name, deployment.ready_replicas, deployment.desired_replicas
                    ),
                });
            }
        }

        total_pods += pods.len();
        total_deployments += deployments.len();
        total_issues += issues.len();

        namespace_analyses.push(NamespaceAnalysis {
            name: namespace.name.clone(),
            pods,
            deployments,
            issues,
        });
    }

    Ok(ClusterAnalysis {
        namespaces: namespace_analyses,
        total_pods,
        total_deployments,
        total_issues,
    })
}
