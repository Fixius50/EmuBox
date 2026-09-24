use crate::services::paths;
use crate::{
    errors::EmuBoxError,
    models::{EmuBoxConfig, SystemSettings},
};
use std::{
    fs,
    io::{ErrorKind, Write},
    path::Path,
};

const DEFAULT_CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../data/config/config.json"
));
const DEFAULT_SETTINGS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../data/settings.json"
));

fn read_or_default(path: &Path, defaults: &str) -> Result<String, EmuBoxError> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(defaults.into()),
        Err(error) => Err(EmuBoxError::StorageUnavailable(format!(
            "{}: {error}",
            path.display()
        ))),
    }
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), EmuBoxError> {
    let parent = path.parent().ok_or_else(|| {
        EmuBoxError::InvalidConfiguration("Ruta de configuracion sin directorio".into())
    })?;
    fs::create_dir_all(parent)
        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
    let content = serde_json::to_vec_pretty(value)
        .map_err(|error| EmuBoxError::InvalidConfiguration(error.to_string()))?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temporary = parent.join(format!(".emubox-config-{}-{timestamp}", std::process::id()));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
    let permissions = match fs::metadata(path) {
        Ok(metadata) => Some(metadata.permissions()),
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => {
            let _ = fs::remove_file(&temporary);
            return Err(EmuBoxError::StorageUnavailable(error.to_string()));
        }
    };
    let result = permissions
        .map_or(Ok(()), |permissions| file.set_permissions(permissions))
        .and_then(|_| file.write_all(&content))
        .and_then(|_| file.sync_all())
        .and_then(|_| fs::rename(&temporary, path));
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result.map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))
}

fn strip_adaptive_settings(value: &mut serde_json::Value) -> bool {
    let mut changed = false;
    if let Some(display) = value
        .get_mut("display")
        .and_then(serde_json::Value::as_object_mut)
    {
        for key in ["resolution", "refreshRate", "fullscreen"] {
            changed |= display.remove(key).is_some();
        }
    }
    if let Some(system) = value
        .get_mut("system")
        .and_then(serde_json::Value::as_object_mut)
    {
        for key in ["performanceMode", "vramLimit"] {
            changed |= system.remove(key).is_some();
        }
    }
    changed
}

fn strip_adaptive_config(value: &mut serde_json::Value) -> bool {
    let mut changed = false;
    if let Some(display) = value
        .get_mut("display")
        .and_then(serde_json::Value::as_object_mut)
    {
        for key in [
            "resolution",
            "refreshRate",
            "fullscreen",
            "gamescopeEnabled",
            "gamescopeScaling",
        ] {
            changed |= display.remove(key).is_some();
        }
    }
    if let Some(interface) = value
        .get_mut("interface")
        .and_then(serde_json::Value::as_object_mut)
    {
        changed |= interface.remove("performanceMode").is_some();
    }
    changed
}

pub fn get_config() -> Result<EmuBoxConfig, EmuBoxError> {
    let config_file = paths::config_file();
    let path = Path::new(&config_file);
    let mut value: serde_json::Value =
        serde_json::from_str(&read_or_default(path, DEFAULT_CONFIG)?)
            .map_err(|error| EmuBoxError::InvalidConfiguration(error.to_string()))?;
    if strip_adaptive_config(&mut value) {
        write_json(path, &value)?;
    }
    serde_json::from_value(value)
        .map_err(|error| EmuBoxError::InvalidConfiguration(error.to_string()))
}

pub fn save_config(config: EmuBoxConfig) -> Result<(), EmuBoxError> {
    write_json(Path::new(&paths::config_file()), &config)
}

pub fn get_settings() -> Result<SystemSettings, EmuBoxError> {
    let mut value: serde_json::Value = serde_json::from_str(&read_or_default(
        Path::new(&paths::settings_file()),
        DEFAULT_SETTINGS,
    )?)
    .map_err(|error| EmuBoxError::InvalidConfiguration(error.to_string()))?;
    let mut changed = false;
    if value.get("library").is_none() {
        let defaults: serde_json::Value = serde_json::from_str(DEFAULT_SETTINGS)
            .map_err(|error| EmuBoxError::InvalidConfiguration(error.to_string()))?;
        value["library"] = defaults["library"].clone();
        changed = true;
    }
    changed |= strip_adaptive_settings(&mut value);
    if changed {
        write_json(Path::new(&paths::settings_file()), &value)?;
    }
    serde_json::from_value(value)
        .map_err(|error| EmuBoxError::InvalidConfiguration(error.to_string()))
}

pub fn save_settings(settings: SystemSettings) -> Result<bool, EmuBoxError> {
    let mut value = serde_json::to_value(settings)
        .map_err(|error| EmuBoxError::InvalidConfiguration(error.to_string()))?;
    strip_adaptive_settings(&mut value);
    let settings: SystemSettings = serde_json::from_value(value)
        .map_err(|error| EmuBoxError::InvalidConfiguration(error.to_string()))?;
    write_json(Path::new(&paths::settings_file()), &settings)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn configuration_io_roundtrip_without_system_paths() {
        let directory =
            std::env::temp_dir().join(format!("emubox-config-tests-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("config.json");
        assert_eq!(
            read_or_default(&path, DEFAULT_CONFIG).unwrap(),
            DEFAULT_CONFIG
        );
        let config: EmuBoxConfig = serde_json::from_str(DEFAULT_CONFIG).unwrap();
        write_json(&path, &config).unwrap();
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        write_json(&path, &config).unwrap();
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        let saved: EmuBoxConfig =
            serde_json::from_str(&read_or_default(&path, "invalid").unwrap()).unwrap();
        assert_eq!(saved.version, config.version);
        assert!(read_or_default(&directory, DEFAULT_CONFIG).is_err());
        fs::remove_dir_all(directory).unwrap();
    }
}
