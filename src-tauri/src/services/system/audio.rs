use crate::{
    errors::EmuBoxError,
    models::{AudioDevice, AudioInfo},
};

pub fn detect() -> Result<AudioInfo, EmuBoxError> {
    let server = crate::services::host_command::json("pactl", &["--format=json", "info"])?;
    let sinks = crate::services::host_command::json("pactl", &["--format=json", "list", "sinks"])?;
    let sinks = sinks.as_array().ok_or_else(|| {
        EmuBoxError::HardwareUnavailable("Lista de salidas de audio invalida".into())
    })?;
    let default_name = server["default_sink_name"].as_str();
    let active = sinks.iter().find(|sink| {
        sink["name"]
            .as_str()
            .is_some_and(|name| Some(name) == default_name)
    });
    let volume = active
        .and_then(|sink| sink["volume"].as_object())
        .and_then(|channels| {
            let values: Vec<f64> = channels
                .values()
                .filter_map(|channel| {
                    channel["value_percent"]
                        .as_str()?
                        .trim_end_matches('%')
                        .parse()
                        .ok()
                })
                .collect();
            (!values.is_empty())
                .then(|| (values.iter().sum::<f64>() / values.len() as f64).round() as u32)
        });
    let settings = crate::services::SystemService::get_settings()?;
    Ok(AudioInfo {
        master_volume: volume,
        ui_sound_effects: settings.audio["uiSoundEffects"].as_bool().ok_or_else(|| {
            EmuBoxError::InvalidConfiguration("audio.uiSoundEffects no es booleano".into())
        })?,
        background_music: settings.audio["backgroundMusic"].as_bool().ok_or_else(|| {
            EmuBoxError::InvalidConfiguration("audio.backgroundMusic no es booleano".into())
        })?,
        latency_ms: None,
        sample_rate: active
            .and_then(|sink| sink["sample_specification"].as_str())
            .and_then(|specification| {
                specification
                    .split_whitespace()
                    .find_map(|part| part.strip_suffix("Hz")?.parse().ok())
            }),
        devices: sinks
            .iter()
            .filter_map(|sink| {
                let name = sink["name"].as_str()?;
                Some(AudioDevice {
                    id: name.into(),
                    name: sink["description"].as_str().unwrap_or(name).into(),
                    is_default: Some(name) == default_name,
                    r#type: "sink".into(),
                })
            })
            .collect(),
    })
}
