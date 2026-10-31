use crate::errors::EmuBoxError;
use crate::models::Emulator;
use crate::services::EmulatorService;

#[tauri::command]
pub async fn get_emulators() -> Result<Vec<Emulator>, EmuBoxError> {
    super::blocking(EmulatorService::get_emulators).await
}

#[tauri::command]
pub async fn get_emulator_by_id(id: String) -> Result<Option<Emulator>, EmuBoxError> {
    super::blocking(move || EmulatorService::get_emulator_by_id(id)).await
}

#[tauri::command]
pub async fn scan_emulators() -> Result<Vec<Emulator>, EmuBoxError> {
    super::blocking(EmulatorService::scan_emulators).await
}

/// Reaplica el perfil de renderer automático (metadata SQLite + configs nativas)
/// según el hardware real detectado. Pensado para dispararse tras un hotplug de
/// GPU/monitor (p. ej. desde `emubox-drm-sync`), sin intervención del usuario.
#[tauri::command]
pub async fn apply_hardware_profile() -> Result<(), EmuBoxError> {
    super::blocking(|| {
        let hardware = crate::services::SystemService::get_hardware_info()?; EmulatorService::apply_hardware_profile(&hardware)
    
    }).await

}

#[tauri::command]
pub async fn get_emulator_status(id: String) -> Result<String, EmuBoxError> {
    super::blocking(move || EmulatorService::get_emulator_status(id)).await
}

#[tauri::command]
pub async fn save_emulator(emulator: Emulator) -> Result<(), EmuBoxError> {
    super::blocking(move || EmulatorService::save_emulator(emulator)).await
}

#[tauri::command]
pub async fn delete_emulator(id: String) -> Result<(), EmuBoxError> {
    super::blocking(move || EmulatorService::delete_emulator(id)).await
}
