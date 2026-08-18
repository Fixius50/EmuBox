use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub timestamp: u64,
    pub level: String,
    pub source: String,
    pub category: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticReport {
    pub generated_at: u64,
    pub os_info: String,
    pub kernel_version: String,
    pub architecture: String,
    pub gpu_adapter: String,
    pub vulkan_ready: bool,
    pub gamescope_ready: bool,
    pub pipewire_ready: bool,
    pub storage_mounted: bool,
    pub emulators_installed_count: usize,
    pub emulators_missing_count: usize,
    pub connected_gamepads_count: usize,
    pub recent_errors: Vec<LogEntry>,
    pub raw_summary_text: String,
}
