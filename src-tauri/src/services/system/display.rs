use crate::{errors::EmuBoxError, models::DisplayInfo};

pub fn detect() -> Result<DisplayInfo, EmuBoxError> {
    let outputs = crate::services::host_command::json("wlr-randr", &["--json"])?;
    parse(&outputs)
}

fn parse(outputs: &serde_json::Value) -> Result<DisplayInfo, EmuBoxError> {
    let output = outputs
        .as_array()
        .and_then(|outputs| {
            outputs
                .iter()
                .find(|output| output["enabled"].as_bool() == Some(true))
        })
        .ok_or_else(|| {
            EmuBoxError::HardwareUnavailable("No se detecta una salida Wayland activa".into())
        })?;
    let mode = output["modes"]
        .as_array()
        .and_then(|modes| {
            modes
                .iter()
                .find(|mode| mode["current"].as_bool() == Some(true))
        })
        .ok_or_else(|| {
            EmuBoxError::HardwareUnavailable("Modo de pantalla actual no disponible".into())
        })?;
    let width = mode["width"]
        .as_u64()
        .and_then(|value| u32::try_from(value).ok());
    let height = mode["height"]
        .as_u64()
        .and_then(|value| u32::try_from(value).ok());
    let (width, height) = width
        .zip(height)
        .ok_or_else(|| EmuBoxError::HardwareUnavailable("Dimensiones no disponibles".into()))?;
    let gamescope_active = std::env::var_os("GAMESCOPE_WAYLAND_DISPLAY").is_some();
    Ok(DisplayInfo {
        resolution: format!("{width}x{height}"),
        width,
        height,
        refresh_rate: mode["refresh"]
            .as_f64()
            .map(|value| (value / 1000.0).round() as u32),
        device_pixel_ratio: output["scale"].as_f64().map(|value| value as f32),
        color_depth: None,
        hdr_supported: None,
        active_compositor: if gamescope_active {
            "gamescope"
        } else {
            "wayland"
        }
        .into(),
        gamescope_active,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_output_is_not_a_default_monitor() {
        assert!(parse(&serde_json::json!([])).is_err());
        assert!(parse(&serde_json::json!([{"enabled": false}])).is_err());
    }
}
