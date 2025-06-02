use serde_json::Value;
use std::fs;
use std::path::Path;

use crate::error::Result;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
    Debug,
}

#[derive(Debug, Clone)]
pub struct ComponentLogs {
    pub component_name: String,
    pub component_type: String,
    pub namespace: String,
    pub entries: Vec<LogEntry>,
    pub total_entries: usize,
}

impl LogLevel {
    pub fn from_str(s: &str) -> Self {
        let s_lower = s.to_lowercase();
        match s_lower.as_str() {
            "error" | "err" | "fatal" | "failed" => LogLevel::Error,
            "warning" | "warn" | "w" => LogLevel::Warning,
            "info" | "information" | "i" => LogLevel::Info,
            "debug" | "dbg" | "trace" | "d" => LogLevel::Debug,
            _ => {
                // Check if the string contains error keywords
                if s_lower.contains("error")
                    || s_lower.contains("failed")
                    || s_lower.contains("fatal")
                {
                    LogLevel::Error
                } else if s_lower.contains("warn") {
                    LogLevel::Warning
                } else if s_lower.contains("info") {
                    LogLevel::Info
                } else if s_lower.contains("debug") || s_lower.contains("trace") {
                    LogLevel::Debug
                } else {
                    LogLevel::Info
                }
            }
        }
    }

    pub fn color_code(&self) -> &'static str {
        match self {
            LogLevel::Error => "🔴",
            LogLevel::Warning => "🟡",
            LogLevel::Info => "🔵",
            LogLevel::Debug => "⚪",
        }
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warning => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
        }
    }
}

pub fn load_pod_logs(namespace: &str, pod_name: &str) -> Result<ComponentLogs> {
    // Per la struttura reale che hai mostrato: output/cert-manager/cert-manager-cainjector-dc95f9d66-t6rg9/logs.txt
    let logs_path = format!("output/{}/{}/logs.txt", namespace, pod_name);
    load_component_logs(&logs_path, pod_name, "Pod", namespace)
}

pub fn load_deployment_logs(namespace: &str, deployment_name: &str) -> Result<ComponentLogs> {
    // Prima prova deployment diretto, poi cerca tra i pod del deployment
    let logs_path = format!("output/{}/{}/logs.txt", namespace, deployment_name);

    // Se non trova deployment diretto, cerca pod correlati
    if !Path::new(&logs_path).exists() {
        // Cerca pod che iniziano con il nome del deployment
        let namespace_dir = format!("output/{}", namespace);
        if let Ok(entries) = fs::read_dir(&namespace_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let dir_name = entry.file_name().to_string_lossy().to_string();
                    if dir_name.starts_with(deployment_name) {
                        // Trovato un pod del deployment, usa quello
                        return load_component_logs(
                            &format!("output/{}/{}/logs.txt", namespace, dir_name),
                            &dir_name,
                            "Pod",
                            namespace,
                        );
                    }
                }
            }
        }
    }

    load_component_logs(&logs_path, deployment_name, "Deployment", namespace)
}

pub fn load_service_logs(namespace: &str, service_name: &str) -> Result<ComponentLogs> {
    let logs_path = format!("output/{}/services/{}/logs.json", namespace, service_name);
    load_component_logs(&logs_path, service_name, "Service", namespace)
}

fn load_component_logs(
    logs_path: &str,
    component_name: &str,
    component_type: &str,
    namespace: &str,
) -> Result<ComponentLogs> {
    let mut entries = Vec::new();
    let mut found_logs = false;

    // Lista di possibili percorsi per i log
    let possible_paths = vec![
        // Struttura originale
        logs_path.to_string(),
        // Struttura reale osservata
        format!("output/{}/{}/logs.txt", namespace, component_name),
        format!("output/{}/{}/log.txt", namespace, component_name),
        // Alternative comuni
        format!("output/{}/logs/{}.log", namespace, component_name),
        format!("output/{}/logs/{}.txt", namespace, component_name),
        format!("output/{}/{}-logs.txt", namespace, component_name),
        // Prova anche con il tipo di componente
        format!(
            "output/{}/{}-{}/logs.txt",
            namespace,
            component_type.to_lowercase(),
            component_name
        ),
    ];

    for log_path in possible_paths {
        if Path::new(&log_path).exists() {
            let content = fs::read_to_string(&log_path)?;

            // Try to parse as JSON logs first
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                entries = parse_json_logs(&json)?;
            } else {
                // Fall back to plain text logs
                entries = parse_text_logs(&content);
            }
            found_logs = true;
            break;
        }
    }

    // Se non troviamo log, cerca tutti i file .txt o .log nella directory del componente
    if !found_logs {
        let component_dir = format!("output/{}/{}", namespace, component_name);
        if let Ok(dir_entries) = fs::read_dir(&component_dir) {
            for entry in dir_entries.flatten() {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "txt" || extension == "log" {
                        if let Ok(content) = fs::read_to_string(&path) {
                            entries = parse_text_logs(&content);
                            found_logs = true;
                            break;
                        }
                    }
                }
            }
        }
    }

    let total_entries = entries.len();

    // Sort by timestamp (most recent first) se abbiamo timestamp validi
    entries.sort_by(|a, b| {
        // Prova a confrontare timestamp, fallback su ordine alfabetico
        match (parse_timestamp(&a.timestamp), parse_timestamp(&b.timestamp)) {
            (Some(time_a), Some(time_b)) => time_b.cmp(&time_a), // Più recenti prima
            _ => b.timestamp.cmp(&a.timestamp),                  // Fallback alfabetico inverso
        }
    });

    Ok(ComponentLogs {
        component_name: component_name.to_string(),
        component_type: component_type.to_string(),
        namespace: namespace.to_string(),
        entries,
        total_entries,
    })
}

// Funzione helper per parsing timestamp
fn parse_timestamp(timestamp: &str) -> Option<std::time::SystemTime> {
    // Prova vari formati comuni
    if timestamp.contains('T') && timestamp.contains('Z') {
        // ISO format: 2024-01-01T10:00:00Z
        return None; // Per ora, implementazione semplice
    }
    if timestamp.starts_with('[') && timestamp.contains(']') {
        // Bracket format: [2024-01-01 10:00:00]
        return None;
    }
    None
}

fn parse_json_logs(json: &Value) -> Result<Vec<LogEntry>> {
    let mut entries = Vec::new();

    match json {
        Value::Array(logs) => {
            for log in logs {
                if let Some(entry) = parse_single_json_log(log) {
                    entries.push(entry);
                }
            }
        }
        Value::Object(_) => {
            // Single log entry
            if let Some(entry) = parse_single_json_log(json) {
                entries.push(entry);
            }
        }
        _ => {
            return Err("Invalid JSON log format".into());
        }
    }

    Ok(entries)
}

fn parse_single_json_log(log: &Value) -> Option<LogEntry> {
    let timestamp = log["timestamp"]
        .as_str()
        .or_else(|| log["time"].as_str())
        .or_else(|| log["@timestamp"].as_str())
        .unwrap_or("unknown")
        .to_string();

    let level_str = log["level"]
        .as_str()
        .or_else(|| log["severity"].as_str())
        .or_else(|| log["loglevel"].as_str())
        .unwrap_or("info");

    let message = log["message"]
        .as_str()
        .or_else(|| log["msg"].as_str())
        .or_else(|| log["text"].as_str())
        .unwrap_or("No message")
        .to_string();

    let source = log["source"]
        .as_str()
        .or_else(|| log["logger"].as_str())
        .or_else(|| log["component"].as_str())
        .unwrap_or("unknown")
        .to_string();

    Some(LogEntry {
        timestamp,
        level: LogLevel::from_str(level_str),
        message,
        source,
    })
}

fn parse_text_logs(content: &str) -> Vec<LogEntry> {
    let mut entries = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        let entry = parse_single_text_log(line, line_num);
        entries.push(entry);
    }

    entries
}

fn parse_single_text_log(line: &str, line_num: usize) -> LogEntry {
    // Common log patterns to extract timestamp, level, and message

    // Pattern 1: Kubernetes logs with [LEVEL] in the message
    // [INFO] plugin/reload: Running configuration...
    // [ERROR] plugin/kubernetes: Failed to watch...
    if let Some(captures) = parse_kubernetes_bracket_log(line) {
        return LogEntry {
            timestamp: format!("line-{}", line_num + 1),
            level: LogLevel::from_str(&captures.0),
            message: captures.1,
            source: "kubernetes".to_string(),
        };
    }

    // Pattern 2: ISO timestamp + level
    // 2024-01-01T10:00:00Z INFO: Message here
    if let Some(captures) = regex_extract_iso_log(line) {
        return LogEntry {
            timestamp: captures.0,
            level: LogLevel::from_str(&captures.1),
            message: captures.2,
            source: "app".to_string(),
        };
    }

    // Pattern 3: Simple timestamp + level
    // [2024-01-01 10:00:00] ERROR: Message here
    if let Some(captures) = regex_extract_bracket_log(line) {
        return LogEntry {
            timestamp: captures.0,
            level: LogLevel::from_str(&captures.1),
            message: captures.2,
            source: "app".to_string(),
        };
    }

    // Pattern 4: Kubernetes pod logs
    // 2024-01-01T10:00:00.000000000Z stderr F Message here
    if let Some(captures) = regex_extract_k8s_log(line) {
        return LogEntry {
            timestamp: captures.0,
            level: LogLevel::from_str(&captures.1),
            message: captures.2,
            source: "container".to_string(),
        };
    }

    // Pattern 5: Check for log level anywhere in the line
    if let Some(level) = extract_log_level_from_line(line) {
        return LogEntry {
            timestamp: format!("line-{}", line_num + 1),
            level,
            message: line.to_string(),
            source: "raw".to_string(),
        };
    }

    // Fallback: treat entire line as message
    LogEntry {
        timestamp: format!("line-{}", line_num + 1),
        level: LogLevel::Info,
        message: line.to_string(),
        source: "raw".to_string(),
    }
}

// Parser specifico per log Kubernetes con [LEVEL]
fn parse_kubernetes_bracket_log(line: &str) -> Option<(String, String)> {
    // Cerca pattern come [INFO], [ERROR], [WARN], [DEBUG]
    let levels = ["ERROR", "WARN", "WARNING", "INFO", "DEBUG", "TRACE"];

    for level in &levels {
        let pattern = format!("[{}]", level);
        if let Some(pos) = line.find(&pattern) {
            let level_str = level.to_string();
            let message = if pos + pattern.len() < line.len() {
                line[pos + pattern.len()..].trim().to_string()
            } else {
                line.to_string()
            };
            return Some((level_str, message));
        }
    }
    None
}

// Extract log level from anywhere in the line
fn extract_log_level_from_line(line: &str) -> Option<LogLevel> {
    let line_upper = line.to_uppercase();

    // Priority order: ERROR > WARN > INFO > DEBUG
    if line_upper.contains("ERROR") || line_upper.contains("FAILED") || line_upper.contains("FATAL")
    {
        Some(LogLevel::Error)
    } else if line_upper.contains("WARN") || line_upper.contains("WARNING") {
        Some(LogLevel::Warning)
    } else if line_upper.contains("INFO") {
        Some(LogLevel::Info)
    } else if line_upper.contains("DEBUG") || line_upper.contains("TRACE") {
        Some(LogLevel::Debug)
    } else {
        None
    }
}

// Simple regex-like extraction (without regex crate to keep dependencies minimal)
fn regex_extract_iso_log(line: &str) -> Option<(String, String, String)> {
    // Look for ISO timestamp pattern
    if let Some(t_pos) = line.find('T') {
        if let Some(z_pos) = line[t_pos..].find('Z') {
            let timestamp = line[..t_pos + z_pos + 1].to_string();
            let rest = &line[t_pos + z_pos + 1..].trim();

            // Look for level
            let parts: Vec<&str> = rest.splitn(2, ':').collect();
            if parts.len() == 2 {
                let level = parts[0].trim().to_string();
                let message = parts[1].trim().to_string();
                return Some((timestamp, level, message));
            }
        }
    }
    None
}

fn regex_extract_bracket_log(line: &str) -> Option<(String, String, String)> {
    if line.starts_with('[') {
        if let Some(end_bracket) = line.find(']') {
            let timestamp = line[1..end_bracket].to_string();
            let rest = &line[end_bracket + 1..].trim();

            let parts: Vec<&str> = rest.splitn(2, ':').collect();
            if parts.len() == 2 {
                let level = parts[0].trim().to_string();
                let message = parts[1].trim().to_string();
                return Some((timestamp, level, message));
            }
        }
    }
    None
}

fn regex_extract_k8s_log(line: &str) -> Option<(String, String, String)> {
    let parts: Vec<&str> = line.splitn(4, ' ').collect();
    if parts.len() >= 4 {
        let timestamp = parts[0].to_string();
        let stream = parts[1]; // stdout/stderr
        let _partial = parts[2]; // F/P
        let message = parts[3..].join(" ");

        let level = if stream == "stderr" { "ERROR" } else { "INFO" };
        return Some((timestamp, level.to_string(), message));
    }
    None
}

pub fn filter_logs_by_level<'a>(
    logs: &'a ComponentLogs,
    min_level: &LogLevel,
) -> Vec<&'a LogEntry> {
    let min_priority = level_priority(min_level);

    logs.entries
        .iter()
        .filter(|entry| level_priority(&entry.level) >= min_priority)
        .collect()
}

pub fn search_logs<'a>(logs: &'a ComponentLogs, query: &str) -> Vec<&'a LogEntry> {
    let query_lower = query.to_lowercase();

    logs.entries
        .iter()
        .filter(|entry| {
            entry.message.to_lowercase().contains(&query_lower)
                || entry.source.to_lowercase().contains(&query_lower)
        })
        .collect()
}

fn level_priority(level: &LogLevel) -> u8 {
    match level {
        LogLevel::Debug => 0,
        LogLevel::Info => 1,
        LogLevel::Warning => 2,
        LogLevel::Error => 3,
    }
}

impl ComponentLogs {
    pub fn get_error_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| matches!(e.level, LogLevel::Error))
            .count()
    }

    pub fn get_warning_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| matches!(e.level, LogLevel::Warning))
            .count()
    }

    pub fn get_recent_logs(&self, count: usize) -> &[LogEntry] {
        let end = std::cmp::min(count, self.entries.len());
        &self.entries[..end]
    }
}
