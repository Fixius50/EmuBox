use crate::models::{GamepadDevice, GamepadStatus};
use crate::errors::EmuBoxError;

#[tauri::command]
pub fn get_gamepads() -> Result<Vec<GamepadDevice>, EmuBoxError> {
    crate::services::input_service::devices()
}

#[tauri::command]
pub fn get_gamepad_status() -> Result<GamepadStatus, EmuBoxError> {
    crate::services::input_service::status()
}
