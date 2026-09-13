use crate::{errors::EmuBoxError, models::{GamepadDevice, GamepadStatus}};
use std::{fs, path::Path};

fn read(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|value| value.trim().to_string())
}

fn bit_count(bitmap: &str) -> Option<usize> {
    bitmap.split_whitespace().try_fold(0, |total, chunk| {
        u64::from_str_radix(chunk, 16).ok().map(|value| total + value.count_ones() as usize)
    })
}

pub fn devices() -> Result<Vec<GamepadDevice>, EmuBoxError> {
    let entries = fs::read_dir("/sys/class/input").map_err(|error| EmuBoxError::HardwareUnavailable(error.to_string()))?;
    let mut devices = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| EmuBoxError::HardwareUnavailable(error.to_string()))?;
        let name = entry.file_name().to_string_lossy().to_string();
        let Some(index) = name.strip_prefix("js").and_then(|value| value.parse::<usize>().ok()) else { continue; };
        let device = entry.path().join("device");
        devices.push(GamepadDevice {
            index, id: format!("/dev/input/{name}"),
            name: read(&device.join("name")).unwrap_or(name), connected: true,
            vendor_id: read(&device.join("id/vendor")), product_id: read(&device.join("id/product")),
            buttons_count: read(&device.join("capabilities/key")).and_then(|value| bit_count(&value)),
            axes_count: read(&device.join("capabilities/abs")).and_then(|value| bit_count(&value)),
            has_vibration: read(&device.join("capabilities/ff")).and_then(|value| bit_count(&value)).map(|count| count > 0),
            battery_percent: None, is_primary: false,
        });
    }
    devices.sort_by_key(|device| device.index);
    if let Some(primary) = devices.first_mut() { primary.is_primary = true; }
    Ok(devices)
}

pub fn status() -> Result<GamepadStatus, EmuBoxError> {
    let devices = devices()?;
    Ok(GamepadStatus { connected_count: devices.len(), primary_device_index: devices.first().map(|device| device.index), devices })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_kernel_capability_bitmaps() {
        assert_eq!(bit_count("f 3"), Some(6));
        assert_eq!(bit_count("0"), Some(0));
        assert_eq!(bit_count("invalid"), None);
    }
}