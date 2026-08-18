use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigPaths {
    pub roms: String,
    pub saves: String,
    pub states: String,
    pub screenshots: String,
    pub covers: String,
    pub logs: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigDisplay {
    pub resolution: String,
    pub refresh_rate: u32,
    pub fullscreen: bool,
    pub vsync: bool,
    pub gamescope_enabled: bool,
    pub gamescope_scaling: String,
    pub crt_shader: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigAudio {
    pub volume: u32,
    pub ui_sound_effects: bool,
    pub background_music: bool,
    pub latency_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigInput {
    pub deadzone: f32,
    pub vibration_enabled: bool,
    pub swap_south_east_buttons: bool,
    pub poll_rate_hz: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigEmulators {
    pub default_mapping: HashMap<String, String>,
    pub custom_binaries_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigInterface {
    pub locale: String,
    pub theme: String,
    pub animations: bool,
    pub show_fps_overlay: bool,
    pub performance_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmuBoxConfig {
    pub version: u32,
    pub paths: ConfigPaths,
    pub display: ConfigDisplay,
    pub audio: ConfigAudio,
    pub input: ConfigInput,
    pub emulators: ConfigEmulators,
    pub interface: ConfigInterface,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemSettings {
    pub display: serde_json::Value,
    pub audio: serde_json::Value,
    pub gamepad: serde_json::Value,
    pub library: serde_json::Value,
    pub system: Option<serde_json::Value>,
}
