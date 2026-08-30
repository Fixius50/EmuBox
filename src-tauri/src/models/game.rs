use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: String,
    pub title: String,
    pub platform: String,
    pub platform_name: String,
    pub emulator_id: String,
    pub release_year: u32,
    pub genre: String,
    pub developer: String,
    pub publisher: String,
    pub rating: f32,
    pub favorite: bool,
    pub cover_image: String,
    pub backdrop_image: String,
    pub description: String,
    pub play_time_minutes: u32,
    pub last_played: Option<String>,
    pub rom_path: Option<String>,
    pub file_size_mb: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Platform {
    pub id: String,
    pub name: String,
    pub short_name: String,
    pub manufacturer: String,
    pub generation: u32,
    pub release_year: u32,
    pub color: String,
    pub icon: String,
    pub default_emulator_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameFilter {
    pub platform: Option<String>,
    pub search: Option<String>,
    pub favorite: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanGamesRequest {
    pub platforms: Option<Vec<String>>,
    pub roms_directory: Option<String>,
    pub deep_scan: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanGamesResult {
    pub scanned_count: usize,
    pub added_count: usize,
    pub updated_count: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameEmulatorAssociation {
    pub game_id: String,
    pub emulator_id: String,
    pub is_default: bool,
    pub priority: i32,
    pub custom_arguments: Vec<String>,
    pub custom_config_path: Option<String>,
    pub enabled: bool,
}
