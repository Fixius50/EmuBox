use crate::errors::EmuBoxError;
use crate::models::Emulator;
use crate::services::db_service::DatabaseService;
use crate::services::emulators::{self, EmulatorProfile};
use crate::services::paths;
use crate::services::runtime::sandbox_paths;
use rusqlite::params;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct EmulatorService;

fn row_to_emulator(row: &rusqlite::Row<'_>) -> rusqlite::Result<Emulator> {
    let parse_list = |column| -> rusqlite::Result<Vec<String>> {
        let json: String = row.get(column)?;
        serde_json::from_str(&json).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                column,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })
    };
    Ok(Emulator {
        id: row.get(0)?,
        name: row.get(1)?,
        version: row.get(2)?,
        supported_platforms: parse_list(3)?,
        core_type: row.get(4)?,
        status: row.get(5)?,
        executable: row.get(6)?,
        arguments: parse_list(7)?,
        architectures: Vec::new(),
        requirements: Default::default(),
        compatibility: Default::default(),
    })
}

impl EmulatorService {
    fn provision_dedicated_environment(
        profile: &dyn EmulatorProfile,
        source_binary: &Path,
    ) -> PathBuf {
        let emu_dir = PathBuf::from(paths::emulator_dir(profile.id()));
        let bin_dir = emu_dir
            .join("bin")
            .join(crate::models::Architecture::current().as_str());
        let config_dir = emu_dir.join("config");
        let logs_dir = emu_dir.join("logs");

        let _ = std::fs::create_dir_all(&bin_dir);
        let _ = std::fs::create_dir_all(&config_dir);
        let _ = std::fs::create_dir_all(&logs_dir);

        if source_binary.starts_with(&emu_dir) {
            return source_binary.to_path_buf();
        }

        let binary_name = source_binary
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new(profile.id()));
        let target_symlink = bin_dir.join(binary_name);

        if target_symlink.exists() || std::fs::symlink_metadata(&target_symlink).is_ok() {
            let _ = std::fs::remove_file(&target_symlink);
        }

        #[cfg(unix)]
        {
            let _ = std::os::unix::fs::symlink(source_binary, &target_symlink);
        }

        if target_symlink.is_file() {
            target_symlink
        } else {
            source_binary.to_path_buf()
        }
    }

    fn find_binary_path(profile: &dyn EmulatorProfile) -> Option<PathBuf> {
        let emu_dir = PathBuf::from(paths::emulators_dir());
        for candidate in profile.binary_candidates() {
            let native = emu_dir
                .join(profile.id())
                .join("bin")
                .join(crate::models::Architecture::current().as_str())
                .join(candidate);
            if native.is_file() {
                return Some(native);
            }
            let nested_bin = emu_dir.join(profile.id()).join("bin").join(candidate);
            if nested_bin.is_file() {
                return Some(nested_bin);
            }
            let direct_nested = emu_dir.join(profile.id()).join(candidate);
            if direct_nested.is_file() {
                return Some(direct_nested);
            }
            let direct_path = emu_dir.join(candidate);
            if direct_path.is_file() {
                return Some(direct_path);
            }
        }

        for candidate in profile.binary_candidates() {
            for prefix in &[
                sandbox_paths::SYSTEM_LOCAL_BIN,
                sandbox_paths::SYSTEM_BIN,
                sandbox_paths::OPT_ROOT,
            ] {
                let p = Path::new(prefix).join(candidate);
                if p.is_file() {
                    return Some(Self::provision_dedicated_environment(profile, &p));
                }
            }
        }

        for candidate in profile.binary_candidates() {
            if let Ok(output) = Command::new("which").arg(candidate).output() {
                if output.status.success() {
                    let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path_str.is_empty() {
                        let p = PathBuf::from(path_str);
                        if p.is_file() {
                            return Some(Self::provision_dedicated_environment(profile, &p));
                        }
                    }
                }
            }
        }

        None
    }

    fn probe_official_version(binary_path: &Path, arguments: &[&str]) -> String {
        if arguments.is_empty() {
            return "Instalado (Oficial)".to_string();
        }
        if let Ok(text) = crate::services::host_command::output_with_timeout(
            &binary_path.to_string_lossy(),
            arguments,
            "3s",
        ) {
            let first_line = text.lines().next().unwrap_or("").trim();
            if !first_line.is_empty() {
                return first_line.to_string();
            }
        }
        "Instalado (version no disponible)".to_string()
    }

    pub fn scan_emulators() -> Result<Vec<Emulator>, EmuBoxError> {
        let hardware = crate::services::SystemService::get_hardware_info()?;
        Self::scan_with_hardware(&hardware)
    }

    pub fn scan_with_hardware(
        hardware: &crate::models::HardwareInfo,
    ) -> Result<Vec<Emulator>, EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        let mut list = Vec::new();
        let host = crate::models::Architecture::current();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(25);
        let mut versions = std::collections::HashMap::new();

        for profile in emulators::registry() {
            if std::time::Instant::now() >= deadline {
                return Err(EmuBoxError::ProcessFailed(
                    "Tiempo limite del inventario de emuladores agotado".into(),
                ));
            }
            let (status, executable, version) =
                if !crate::services::emulator_capabilities::supports(profile.id(), host) {
                    (
                        "inactive".to_string(),
                        "".to_string(),
                        "Arquitectura no compatible".to_string(),
                    )
                } else if let Some(binary_path) = Self::find_binary_path(profile.as_ref()) {
                    if crate::services::binary_service::validate_binary(&binary_path, host, true)
                        .is_ok()
                    {
                        let arguments = profile.version_arguments();
                        let key = (
                            std::fs::canonicalize(&binary_path)
                                .unwrap_or_else(|_| binary_path.clone()),
                            arguments
                                .iter()
                                .map(|argument| argument.to_string())
                                .collect::<Vec<_>>(),
                        );
                        let raw_version = versions
                            .entry(key)
                            .or_insert_with(|| {
                                Self::probe_official_version(&binary_path, &arguments)
                            })
                            .clone();
                        (
                            "active".to_string(),
                            binary_path.to_string_lossy().to_string(),
                            raw_version,
                        )
                    } else {
                        (
                            "inactive".to_string(),
                            binary_path.to_string_lossy().to_string(),
                            "Binario incompatible".to_string(),
                        )
                    }
                } else {
                    (
                        "inactive".to_string(),
                        "".to_string(),
                        "No instalado".to_string(),
                    )
                };

            let platforms_json = serde_json::to_string(&profile.supported_platforms())
                .unwrap_or_else(|_| "[]".to_string());
            let args_json = serde_json::to_string(&profile.default_arguments())
                .unwrap_or_else(|_| "[]".to_string());

            conn.execute(
                "INSERT INTO emulators (id, official_name, version, supported_platforms_json, core_type, status, executable_path, default_arguments_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(id) DO UPDATE SET
                   version = excluded.version,
                   status = excluded.status,
                   executable_path = excluded.executable_path,
                   default_arguments_json = excluded.default_arguments_json;",
                params![profile.id(), profile.official_name(), version, platforms_json, profile.core_type(), status, executable, args_json]
            ).map_err(|e| EmuBoxError::StorageUnavailable(format!("Error guardando emulador en SQLite: {}", e)))?;

            // Metadata específica por emulador
            let config_dir = paths::emulator_config_dir(profile.id());
            let bios_dir = paths::bios_dir();
            let saves_dir = paths::saves_dir();
            let states_dir = paths::states_dir();

            conn.execute(
                "INSERT INTO emulator_metadata (emulator_id, config_dir, bios_dir, saves_dir, states_dir, renderer)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'auto')
                 ON CONFLICT(emulator_id) DO NOTHING;",
                params![profile.id(), config_dir, bios_dir, saves_dir, states_dir]
            ).map_err(|e| EmuBoxError::StorageUnavailable(format!("Error guardando metadata de emulador: {}", e)))?;

            let mut emulator = Emulator {
                id: profile.id().to_string(),
                name: profile.official_name().to_string(),
                version,
                supported_platforms: profile
                    .supported_platforms()
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                core_type: profile.core_type().to_string(),
                status,
                executable,
                arguments: profile
                    .default_arguments()
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                architectures: Vec::new(),
                requirements: Default::default(),
                compatibility: Default::default(),
            };
            crate::services::emulator_capabilities::refresh(&mut emulator, host, hardware);
            list.push(emulator);
        }

        Ok(list)
    }

    /// Aplica el perfil de hardware detectado: cada emulador registrado en
    /// `services::emulators` resuelve y persiste su propio renderer óptimo (metadata
    /// SQLite + configuración nativa si la tiene implementada), sin intervención del
    /// usuario. Debe ejecutarse tras `scan_emulators` y cada vez que cambie el hardware
    /// (hotplug de GPU/monitor).
    pub fn apply_hardware_profile(
        hardware: &crate::models::HardwareInfo,
    ) -> Result<(), EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        let renderer = emulators::renderer_preference(&hardware.graphics.operational_backend);

        for profile in emulators::registry() {
            if !crate::services::emulator_capabilities::supports(
                profile.id(),
                crate::models::Architecture::current(),
            ) {
                continue;
            }
            conn.execute(
                "UPDATE emulator_metadata SET renderer = ?1 WHERE emulator_id = ?2;",
                params![renderer.metadata_name(profile.core_type()), profile.id()],
            )
            .map_err(|e| {
                EmuBoxError::StorageUnavailable(format!(
                    "Error aplicando perfil de hardware a {}: {}",
                    profile.id(),
                    e
                ))
            })?;

            profile.apply_hardware_config(hardware, renderer)?;
        }

        Ok(())
    }

    pub fn get_emulators() -> Result<Vec<Emulator>, EmuBoxError> {
        let hardware = crate::services::SystemService::get_hardware_info()?;
        let list = Self::cached_with_hardware(&hardware)?;
        if list.is_empty() {
            return Self::scan_with_hardware(&hardware);
        }
        Ok(list)
    }

    pub fn cached_with_hardware(
        hardware: &crate::models::HardwareInfo,
    ) -> Result<Vec<Emulator>, EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, official_name, version, supported_platforms_json, core_type, status, executable_path, default_arguments_json
             FROM emulators ORDER BY official_name ASC;"
        ).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;

        let rows = stmt
            .query_map([], row_to_emulator)
            .map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;

        let mut list = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        for emulator in &mut list {
            crate::services::emulator_capabilities::refresh(
                emulator,
                crate::models::Architecture::current(),
                hardware,
            );
        }

        Ok(list)
    }

    pub fn get_emulator_by_id(id: String) -> Result<Option<Emulator>, EmuBoxError> {
        let emulators = Self::get_emulators()?;
        Ok(emulators.into_iter().find(|e| e.id == id))
    }

    pub fn get_emulator_status(id: String) -> Result<String, EmuBoxError> {
        let emu = Self::get_emulator_by_id(id)?;
        Ok(emu
            .map(|e| e.status)
            .unwrap_or_else(|| "not_found".to_string()))
    }

    pub fn save_emulator(emulator: Emulator) -> Result<(), EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        let platforms_json = serde_json::to_string(&emulator.supported_platforms)
            .map_err(|error| EmuBoxError::InvalidConfiguration(error.to_string()))?;
        let args_json = serde_json::to_string(&emulator.arguments)
            .map_err(|error| EmuBoxError::InvalidConfiguration(error.to_string()))?;

        conn.execute(
            "INSERT INTO emulators (id, official_name, version, supported_platforms_json, core_type, status, executable_path, default_arguments_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
               official_name = excluded.official_name,
               version = excluded.version,
               supported_platforms_json = excluded.supported_platforms_json,
               core_type = excluded.core_type,
               status = excluded.status,
               executable_path = excluded.executable_path,
               default_arguments_json = excluded.default_arguments_json;",
            params![
                emulator.id,
                emulator.name,
                emulator.version,
                platforms_json,
                emulator.core_type,
                emulator.status,
                emulator.executable,
                args_json
            ]
        ).map_err(|e| EmuBoxError::StorageUnavailable(format!("Error al persistir emulador en SQLite: {}", e)))?;

        Ok(())
    }

    pub fn delete_emulator(id: String) -> Result<(), EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        conn.execute("DELETE FROM emulators WHERE id = ?1;", params![id])
            .map_err(|e| {
                EmuBoxError::StorageUnavailable(format!(
                    "Error al eliminar emulador de SQLite: {}",
                    e
                ))
            })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emulator_mapping_rejects_invalid_json_and_rows() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        for (platforms, arguments) in [("not json", "[]"), ("[]", "[1]"), ("{}", "[]")] {
            assert!(connection
                .query_row(
                    "SELECT 'id','name','1',?1,'standalone','installed','/bin/true',?2",
                    params![platforms, arguments],
                    row_to_emulator,
                )
                .is_err());
        }
        let emulator = connection.query_row(
            "SELECT 'id','name','1','[\"ps3\"]','standalone','installed','/bin/true','[\"--fullscreen\"]'",
            [], row_to_emulator,
        ).unwrap();
        assert_eq!(emulator.supported_platforms, vec!["ps3"]);
        assert_eq!(emulator.arguments, vec!["--fullscreen"]);
        assert!(connection
            .query_row(
                "SELECT NULL,'name','1','[]','standalone','installed','/bin/true','[]'",
                [],
                row_to_emulator
            )
            .is_err());
    }

    #[test]
    fn test_provision_dedicated_environment() {
        let binary = crate::services::binary_service::resolve_executable("true")
            .expect("coreutils true must exist");
        let profile = emulators::registry()
            .into_iter()
            .find(|profile| profile.id() == "ppsspp")
            .unwrap();
        let provisioned =
            EmulatorService::provision_dedicated_environment(profile.as_ref(), &binary);

        assert!(provisioned.starts_with(
            PathBuf::from(paths::emulator_dir("ppsspp"))
                .join("bin")
                .join(crate::models::Architecture::current().as_str())
        ));
        assert!(provisioned.is_file());

        let _ = std::fs::remove_dir_all(paths::emulator_dir("ppsspp"));
    }
}
