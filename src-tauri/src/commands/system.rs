use crate::errors::EmuBoxError;
use crate::models::{
    AudioInfo, DisplayInfo, EmuBoxConfig, FirstRunDetectionResult, HardwareInfo, SystemInfo,
    SystemSettings,
};
use crate::services::SystemService;

#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, EmuBoxError> {
    super::blocking(SystemService::get_system_info).await
}

#[tauri::command]
pub async fn get_hardware_info() -> Result<HardwareInfo, EmuBoxError> {
    super::blocking(SystemService::get_hardware_info).await
}

#[tauri::command]
pub async fn get_display_info() -> Result<DisplayInfo, EmuBoxError> {
    super::blocking(SystemService::get_display_info).await
}

#[tauri::command]
pub async fn get_audio_info() -> Result<AudioInfo, EmuBoxError> {
    super::blocking(SystemService::get_audio_info).await
}

#[tauri::command]
pub async fn first_run_detection() -> Result<FirstRunDetectionResult, EmuBoxError> {
    super::blocking(SystemService::first_run_detection).await
}

#[tauri::command]
pub async fn get_config() -> Result<EmuBoxConfig, EmuBoxError> {
    super::blocking(SystemService::get_config).await
}

#[tauri::command]
pub async fn save_config(config: EmuBoxConfig) -> Result<(), EmuBoxError> {
    super::blocking(move || SystemService::save_config(config)).await
}

#[tauri::command]
pub async fn get_settings() -> Result<SystemSettings, EmuBoxError> {
    super::blocking(SystemService::get_settings).await
}

#[tauri::command]
pub async fn save_settings(settings: SystemSettings) -> Result<bool, EmuBoxError> {
    super::blocking(move || SystemService::save_settings(settings)).await
}

#[tauri::command]
pub fn system_shutdown() -> Result<(), EmuBoxError> {
    crate::services::power_service::execute(crate::models::system::PowerAction::Shutdown)
}

#[tauri::command]
pub fn system_restart() -> Result<(), EmuBoxError> {
    crate::services::power_service::execute(crate::models::system::PowerAction::Restart)
}

#[tauri::command]
pub fn system_sleep() -> Result<(), EmuBoxError> {
    crate::services::power_service::execute(crate::models::system::PowerAction::Sleep)
}

#[tauri::command]
pub fn system_logout() -> Result<(), EmuBoxError> {
    crate::services::power_service::execute(crate::models::system::PowerAction::Logout)
}

#[tauri::command]
pub fn restart_app_session(app: tauri::AppHandle) {
    app.restart();
}

#[tauri::command]
pub fn exit_to_linux_shell() -> Result<(), EmuBoxError> {
    log::info!("Exit to Linux Shell requested by user");
    SystemService::exit_to_linux_shell()
}
