use std::collections::HashMap;
use crate::models::BiosStatus;
use crate::errors::EmuBoxError;

pub struct BiosService;

impl BiosService {
    pub fn get_bios_requirements() -> Result<BiosStatus, EmuBoxError> {
        Ok(BiosStatus {
            total_required: 0,
            total_found: 0,
            missing_required_count: 0,
            platforms: HashMap::new(),
        })
    }

    pub fn scan_bios() -> Result<BiosStatus, EmuBoxError> {
        Self::get_bios_requirements()
    }
}
