use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionState {
    Accelerated,
    Software,
    #[default]
    Indeterminate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphicsDevice {
    pub id: String,
    pub card_nodes: Vec<String>,
    pub render_nodes: Vec<String>,
    pub render_identifiers: Vec<String>,
    pub pci_address: Option<String>,
    pub driver: String,
    pub vendor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphicsObservation {
    pub api: String,
    pub state: DetectionState,
    pub renderer: String,
    pub device_id: Option<String>,
    pub correlation: String,
    pub reported_device: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphicsProbe {
    pub api: String,
    pub state: DetectionState,
    pub reason: String,
    pub exit_code: Option<i32>,
    pub observations: Vec<GraphicsObservation>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphicsCapabilities {
    pub detection_state: DetectionState,
    pub inventory_complete: bool,
    pub inventory_reason: Option<String>,
    pub devices: Vec<GraphicsDevice>,
    pub probes: Vec<GraphicsProbe>,
    pub selected_device_id: Option<String>,
    pub active_device_id: Option<String>,
    pub operational_backend: String,
    pub fallback_reason: Option<String>,
    pub vendor: String,
    pub renderer: String,
    pub driver_version: Option<String>,
    pub vulkan: bool,
    pub opengl: bool,
    pub opengl_renderer: Option<String>,
    pub drm: bool,
    pub gamescope: bool,
    pub cage: bool,
    pub accelerated: bool,
    pub backend: String,
    pub gpu_kind: String,
    pub virtual_machine: bool,
    pub gamescope_ready: bool,
    pub compositor: String,
    pub device: String,
}
