use crate::{errors::EmuBoxError, models::{EmuBoxConfig, SystemSettings}};
use std::{fs, io::{ErrorKind, Write}, path::Path};
use super::paths;

const DEFAULT_CONFIG: &str = include_str!("../../../data/config/config.json");
const DEFAULT_SETTINGS: &str = include_str!("../../../data/settings.json");

fn read_or_default(path: &Path, defaults: &str) -> Result<String, EmuBoxError> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(defaults.into()),
        Err(error) => Err(EmuBoxError::StorageUnavailable(format!("{}: {error}", path.display()))),
    }
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), EmuBoxError> {
    let parent = path.parent().ok_or_else(|| EmuBoxError::InvalidConfiguration("Ruta de configuracion sin directorio".into()))?;
    fs::create_dir_all(parent).map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
    let content = serde_json::to_vec_pretty(value).map_err(|error| EmuBoxError::InvalidConfiguration(error.to_string()))?;
    let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
    let temporary = parent.join(format!(".emubox-config-{}-{timestamp}", std::process::id()));
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&temporary)
        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
    let result = file.write_all(&content).and_then(|_| file.sync_all()).and_then(|_| fs::rename(&temporary, path));
    if result.is_err() { let _ = fs::remove_file(&temporary); }
    result.map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))
}

pub fn get_config() -> Result<EmuBoxConfig, EmuBoxError> {
    serde_json::from_str(&read_or_default(Path::new(&paths::config_file()), DEFAULT_CONFIG)?)
        .map_err(|error| EmuBoxError::InvalidConfiguration(error.to_string()))
}

pub fn save_config(config: EmuBoxConfig) -> Result<(), EmuBoxError> {
    write_json(Path::new(&paths::config_file()), &config)
}

pub fn get_settings() -> Result<SystemSettings, EmuBoxError> {
    let mut value: serde_json::Value = serde_json::from_str(&read_or_default(Path::new(&paths::settings_file()), DEFAULT_SETTINGS)?)
        .map_err(|error| EmuBoxError::InvalidConfiguration(error.to_string()))?;
    if value.get("library").is_none() {
        let defaults: serde_json::Value = serde_json::from_str(DEFAULT_SETTINGS).map_err(|error| EmuBoxError::InvalidConfiguration(error.to_string()))?;
        value["library"] = defaults["library"].clone();
    }
    serde_json::from_value(value).map_err(|error| EmuBoxError::InvalidConfiguration(error.to_string()))
}

pub fn save_settings(settings: SystemSettings) -> Result<bool, EmuBoxError> {
    write_json(Path::new(&paths::settings_file()), &settings)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn configuration_io_roundtrip_without_system_paths() {
        let directory = std::env::temp_dir().join(format!("emubox-config-tests-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("config.json");
        assert_eq!(read_or_default(&path, DEFAULT_CONFIG).unwrap(), DEFAULT_CONFIG);
        let config: EmuBoxConfig = serde_json::from_str(DEFAULT_CONFIG).unwrap();
        write_json(&path, &config).unwrap();
        let saved: EmuBoxConfig = serde_json::from_str(&read_or_default(&path, "invalid").unwrap()).unwrap();
        assert_eq!(saved.version, config.version);
        assert!(read_or_default(&directory, DEFAULT_CONFIG).is_err());
        fs::remove_dir_all(directory).unwrap();
    }
}