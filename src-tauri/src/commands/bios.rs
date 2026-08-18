use crate::models::BiosStatus;
use crate::services::BiosService;
use crate::errors::EmuBoxError;

#[tauri::command]
pub fn get_bios_requirements() -> Result<BiosStatus, EmuBoxError> {
    BiosService::get_bios_requirements()
}

#[tauri::command]
pub fn scan_bios() -> Result<BiosStatus, EmuBoxError> {
    BiosService::scan_bios()
}
