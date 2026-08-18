use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunningGameInfo {
    pub pid: u32,
    pub game_id: String,
    pub game_title: String,
    pub platform_id: String,
    pub emulator_id: String,
    pub emulator_name: String,
    pub executable: String,
    pub arguments: Vec<String>,
    pub start_time: u64,
    pub cpu_percent: f32,
    pub memory_mb: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessStatus {
    pub has_active_game: bool,
    pub running_game: Option<RunningGameInfo>,
    pub active_child_pids: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchGameRequest {
    pub game_id: String,
    pub emulator_id: String,
    pub rom_path: Option<String>,
    pub save_state_slot: Option<u32>,
    pub custom_args: Option<Vec<String>>,
    pub fullscreen: Option<bool>,
    pub use_gamescope: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchResult {
    pub success: bool,
    pub message: String,
    pub pid: Option<u32>,
    pub executable: Option<String>,
    pub start_time: Option<u64>,
}
