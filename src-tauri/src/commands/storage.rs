use std::collections::HashMap;
use crate::models::{StorageInfo, StorageLocation};
use crate::services::StorageService;
use crate::errors::EmuBoxError;

#[tauri::command]
pub fn get_storage_info() -> Result<StorageInfo, EmuBoxError> {
    StorageService::get_storage_info()
}

#[tauri::command]
pub fn get_storage_locations() -> Result<HashMap<String, StorageLocation>, EmuBoxError> {
    StorageService::get_storage_locations()
}
