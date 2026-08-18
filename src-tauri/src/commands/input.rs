use crate::models::{GamepadDevice, GamepadStatus};
use crate::errors::EmuBoxError;

#[tauri::command]
pub fn get_gamepads() -> Result<Vec<GamepadDevice>, EmuBoxError> {
    Ok(vec![
        GamepadDevice {
            index: 0,
            id: "xinput-pad-0".to_string(),
            name: "Xbox Wireless Controller".to_string(),
            connected: true,
            vendor_id: None,
            product_id: None,
            buttons_count: 16,
            axes_count: 4,
            has_vibration: true,
            battery_percent: None,
            is_primary: true,
        }
    ])
}

#[tauri::command]
pub fn get_gamepad_status() -> Result<GamepadStatus, EmuBoxError> {
    let devices = get_gamepads()?;
    Ok(GamepadStatus {
        connected_count: devices.len(),
        primary_device_index: 0,
        devices,
    })
}
