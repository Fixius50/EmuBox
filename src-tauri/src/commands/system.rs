use crate::models::{SystemInfo, HardwareInfo, DisplayInfo, AudioInfo, FirstRunDetectionResult, EmuBoxConfig, SystemSettings};
use crate::services::SystemService;
use crate::errors::EmuBoxError;

#[tauri::command]
pub fn get_system_info() -> Result<SystemInfo, EmuBoxError> {
    SystemService::get_system_info()
}

#[tauri::command]
pub fn get_hardware_info() -> Result<HardwareInfo, EmuBoxError> {
    SystemService::get_hardware_info()
}

#[tauri::command]
pub fn get_display_info() -> Result<DisplayInfo, EmuBoxError> {
    SystemService::get_display_info()
}

#[tauri::command]
pub fn get_audio_info() -> Result<AudioInfo, EmuBoxError> {
    SystemService::get_audio_info()
}

#[tauri::command]
pub fn first_run_detection() -> Result<FirstRunDetectionResult, EmuBoxError> {
    SystemService::first_run_detection()
}

#[tauri::command]
pub fn get_config() -> Result<EmuBoxConfig, EmuBoxError> {
    SystemService::get_config()
}

#[tauri::command]
pub fn save_config(config: EmuBoxConfig) -> Result<(), EmuBoxError> {
    SystemService::save_config(config)
}

#[tauri::command]
pub fn get_settings() -> Result<SystemSettings, EmuBoxError> {
    SystemService::get_settings()
}

#[tauri::command]
pub fn save_settings(settings: SystemSettings) -> Result<bool, EmuBoxError> {
    SystemService::save_settings(settings)
}

#[tauri::command]
pub fn system_shutdown() -> Result<(), EmuBoxError> {
    log::info!("System shutdown requested");
    Ok(())
}

#[tauri::command]
pub fn system_restart() -> Result<(), EmuBoxError> {
    log::info!("System restart requested");
    Ok(())
}

#[tauri::command]
pub fn system_sleep() -> Result<(), EmuBoxError> {
    log::info!("System sleep requested");
    Ok(())
}

#[tauri::command]
pub fn system_logout() -> Result<(), EmuBoxError> {
    log::info!("System logout requested");
    Ok(())
}

#[tauri::command]
pub fn exit_to_linux_shell() -> Result<(), EmuBoxError> {
    log::info!("Exit to Linux Shell requested by user");
    SystemService::exit_to_linux_shell()
}
