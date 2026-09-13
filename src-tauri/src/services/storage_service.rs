use super::paths;
use crate::errors::EmuBoxError;
use crate::models::{StorageDrive, StorageInfo, StorageLocation};
use std::collections::HashMap;
use std::{path::Path, process::Command};

pub struct StorageService;

impl StorageService {
    pub fn get_storage_info() -> Result<StorageInfo, EmuBoxError> {
        let disks = sysinfo::Disks::new_with_refreshed_list();
        let locations = Self::get_storage_locations()?;
        Ok(StorageInfo {
            drives: disks
                .iter()
                .map(|disk| StorageDrive {
                    id: disk.mount_point().to_string_lossy().into(),
                    name: disk.name().to_string_lossy().into(),
                    mount_point: disk.mount_point().to_string_lossy().into(),
                    filesystem: disk.file_system().to_string_lossy().into(),
                    total_bytes: disk.total_space(),
                    available_bytes: disk.available_space(),
                    used_bytes: disk.total_space().saturating_sub(disk.available_space()),
                    is_removable: disk.is_removable(),
                    is_system_drive: disk.mount_point() == Path::new("/"),
                })
                .collect(),
            total_games_storage_bytes: locations
                .get("roms")
                .map_or(0, |location| location.total_bytes),
            total_saves_storage_bytes: locations
                .get("saves")
                .map_or(0, |location| location.total_bytes),
            locations,
        })
    }

    pub fn get_storage_locations() -> Result<HashMap<String, StorageLocation>, EmuBoxError> {
        [
            ("roms", "Juegos", paths::games_dir()),
            ("saves", "Partidas", paths::saves_dir()),
            ("states", "Estados", paths::states_dir()),
            ("screenshots", "Capturas", paths::screenshots_dir()),
            ("bios", "BIOS", paths::bios_dir()),
            ("logs", "Registros", paths::LOG_DIR.into()),
            ("cache", "Cache", paths::CACHE_DIR.into()),
            ("covers", "Portadas", format!("{}/covers", paths::DATA_DIR)),
        ]
        .into_iter()
        .map(|(id, label, path)| {
            Self::inspect_location(id, label, &path).map(|location| (id.into(), location))
        })
        .collect()
    }

    fn inspect_location(id: &str, label: &str, path: &str) -> Result<StorageLocation, EmuBoxError> {
        let mut location = StorageLocation {
            id: id.into(),
            label: label.into(),
            path: path.into(),
            total_files: 0,
            total_bytes: 0,
            accessible: Path::new(path).is_dir(),
            is_writable: false,
        };
        if !location.accessible {
            return Ok(location);
        }
        location.is_writable = Command::new("test")
            .args(["-w", path])
            .status()
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?
            .success();
        for entry in walkdir::WalkDir::new(path).follow_links(false) {
            let entry =
                entry.map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
            if entry.file_type().is_file() {
                location.total_files += 1;
                location.total_bytes += entry
                    .metadata()
                    .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?
                    .len();
            }
        }
        Ok(location)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_directory_is_not_reported_accessible() {
        let path =
            std::env::temp_dir().join(format!("emubox-absent-storage-{}", std::process::id()));
        let location =
            StorageService::inspect_location("roms", "Juegos", path.to_str().unwrap()).unwrap();
        assert!(!location.accessible);
        assert!(!location.is_writable);
    }
}
