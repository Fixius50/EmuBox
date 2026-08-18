use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BiosFile {
    pub filename: String,
    pub expected_md5: Option<String>,
    pub expected_sha1: Option<String>,
    pub description: String,
    pub found_path: Option<String>,
    pub state: String,
    pub file_size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BiosRequirement {
    pub platform_id: String,
    pub platform_name: String,
    pub emulator_id: String,
    pub bios_files: Vec<BiosFile>,
    pub all_required_present: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BiosStatus {
    pub total_required: usize,
    pub total_found: usize,
    pub missing_required_count: usize,
    pub platforms: HashMap<String, BiosRequirement>,
}
