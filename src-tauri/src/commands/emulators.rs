use crate::models::Emulator;
use crate::services::EmulatorService;
use crate::errors::EmuBoxError;

#[tauri::command]
pub fn get_emulators() -> Result<Vec<Emulator>, EmuBoxError> {
    EmulatorService::get_emulators()
}

#[tauri::command]
pub fn get_emulator_by_id(id: String) -> Result<Option<Emulator>, EmuBoxError> {
    EmulatorService::get_emulator_by_id(id)
}

#[tauri::command]
pub fn scan_emulators() -> Result<Vec<Emulator>, EmuBoxError> {
    EmulatorService::scan_emulators()
}

/// Reaplica el perfil de renderer automático (metadata SQLite + configs nativas)
/// según el hardware real detectado. Pensado para dispararse tras un hotplug de
/// GPU/monitor (p. ej. desde `emubox-drm-sync`), sin intervención del usuario.
#[tauri::command]
pub fn apply_hardware_profile() -> Result<(), EmuBoxError> {
    let hardware = crate::services::SystemService::get_hardware_info()?;
    EmulatorService::apply_hardware_profile(&hardware)
}

#[tauri::command]
pub fn get_emulator_status(id: String) -> Result<String, EmuBoxError> {
    EmulatorService::get_emulator_status(id)
}

#[tauri::command]
pub fn save_emulator(emulator: Emulator) -> Result<(), EmuBoxError> {
    EmulatorService::save_emulator(emulator)
}

#[tauri::command]
pub fn delete_emulator(id: String) -> Result<(), EmuBoxError> {
    EmulatorService::delete_emulator(id)
}
