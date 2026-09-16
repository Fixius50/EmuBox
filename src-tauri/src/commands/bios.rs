use crate::errors::EmuBoxError;
use crate::models::BiosStatus;
use crate::services::BiosService;

#[tauri::command]
pub fn get_bios_requirements() -> Result<BiosStatus, EmuBoxError> {
    BiosService::get_bios_requirements()
}

#[tauri::command]
pub fn scan_bios() -> Result<BiosStatus, EmuBoxError> {
    BiosService::scan_bios()
}
