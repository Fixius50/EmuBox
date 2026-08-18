use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageDrive {
    pub id: String,
    pub name: String,
    pub mount_point: String,
    pub filesystem: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub is_removable: bool,
    pub is_system_drive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageLocation {
    pub id: String,
    pub label: String,
    pub path: String,
    pub total_files: usize,
    pub total_bytes: u64,
    pub accessible: bool,
    pub is_writable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageInfo {
    pub drives: Vec<StorageDrive>,
    pub locations: HashMap<String, StorageLocation>,
    pub total_games_storage_bytes: u64,
    pub total_saves_storage_bytes: u64,
}
