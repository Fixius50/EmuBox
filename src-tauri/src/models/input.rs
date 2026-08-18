use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GamepadDevice {
    pub index: usize,
    pub id: String,
    pub name: String,
    pub connected: bool,
    pub vendor_id: Option<String>,
    pub product_id: Option<String>,
    pub buttons_count: usize,
    pub axes_count: usize,
    pub has_vibration: bool,
    pub battery_percent: Option<u32>,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GamepadStatus {
    pub connected_count: usize,
    pub primary_device_index: usize,
    pub devices: Vec<GamepadDevice>,
}
