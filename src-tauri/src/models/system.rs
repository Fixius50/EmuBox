use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareInfo {
    pub gpu_vendor: String,
    pub gpu_renderer: String,
    pub vulkan_driver_version: Option<String>,
    pub vulkan_supported: bool,
    pub drm_available: bool,
    pub gamescope_available: bool,
    pub recommended_compositor: String,
    pub device_model: String,
    pub cpu_model: String,
    pub cpu_cores: usize,
    pub cpu_architecture: String,
    pub total_memory_mb: u64,
    pub free_memory_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayInfo {
    pub resolution: String,
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32,
    pub device_pixel_ratio: f32,
    pub color_depth: u32,
    pub hdr_supported: bool,
    pub active_compositor: String,
    pub gamescope_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub r#type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioInfo {
    pub master_volume: u32,
    pub ui_sound_effects: bool,
    pub background_music: bool,
    pub latency_ms: u32,
    pub sample_rate: u32,
    pub devices: Vec<AudioDevice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub os_name: String,
    pub kernel_version: String,
    pub architecture: String,
    pub kernel_architecture: String,
    pub hostname: String,
    pub uptime_seconds: u64,
    pub hardware: HardwareInfo,
    pub display: DisplayInfo,
    pub audio: AudioInfo,
    pub battery_level_percent: Option<u32>,
    pub is_plugged_in: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirstRunDetectionResult {
    pub gpu_vendor: String,
    pub gpu_renderer: String,
    pub vulkan_supported: bool,
    pub gamepads_detected: Vec<String>,
    pub installed_emulators: Vec<String>,
    pub roms_directory_found: bool,
    pub config_generated: bool,
}
