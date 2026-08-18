use std::collections::HashMap;
use crate::models::{StorageInfo, StorageLocation};
use crate::errors::EmuBoxError;

pub struct StorageService;

impl StorageService {
    pub fn get_storage_info() -> Result<StorageInfo, EmuBoxError> {
        Ok(StorageInfo {
            drives: vec![],
            locations: HashMap::new(),
            total_games_storage_bytes: 0,
            total_saves_storage_bytes: 0,
        })
    }

    pub fn get_storage_locations() -> Result<HashMap<String, StorageLocation>, EmuBoxError> {
        Ok(HashMap::new())
    }
}
