use std::path::{Path, PathBuf};
use std::process::Command;
use rusqlite::params;
use crate::models::Emulator;
use crate::errors::EmuBoxError;
use crate::services::db_service::DatabaseService;
use crate::services::emulators::{self, EmulatorProfile};

pub struct EmulatorService;

impl EmulatorService {
    fn provision_dedicated_environment(profile: &dyn EmulatorProfile, source_binary: &Path) -> PathBuf {
        let emu_dir = Path::new("/var/lib/emubox/emulators").join(profile.id());
        let bin_dir = emu_dir.join("bin");
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
        let emu_dir = Path::new("/var/lib/emubox/emulators");
        for candidate in profile.binary_candidates() {
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
            for prefix in &["/usr/local/bin", "/usr/bin", "/opt"] {
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

    fn probe_official_version(binary_path: &Path, flag: &str) -> String {
        if let Ok(output) = Command::new(binary_path).arg(flag).output() {
            let text = if output.status.success() {
                String::from_utf8_lossy(&output.stdout).to_string()
            } else {
                String::from_utf8_lossy(&output.stderr).to_string()
            };

            let first_line = text.lines().next().unwrap_or("").trim();
            if !first_line.is_empty() {
                return first_line.to_string();
            }
        }
        "Instalado (Oficial)".to_string()
    }

    pub fn scan_emulators() -> Result<Vec<Emulator>, EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        let mut list = Vec::new();

        for profile in emulators::registry() {
            let (status, executable, version) = if let Some(binary_path) = Self::find_binary_path(profile.as_ref()) {
                let raw_version = Self::probe_official_version(&binary_path, profile.version_flag());
                ("active".to_string(), binary_path.to_string_lossy().to_string(), raw_version)
            } else {
                ("inactive".to_string(), "".to_string(), "No instalado".to_string())
            };

            let platforms_json = serde_json::to_string(&profile.supported_platforms()).unwrap_or_else(|_| "[]".to_string());
            let args_json = serde_json::to_string(&profile.default_arguments()).unwrap_or_else(|_| "[]".to_string());

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
            let config_dir = format!("/var/lib/emubox/emulators/{}/config", profile.id());
            let bios_dir = "/var/lib/emubox/bios".to_string();
            let saves_dir = "/var/lib/emubox/saves".to_string();
            let states_dir = "/var/lib/emubox/states".to_string();

            conn.execute(
                "INSERT INTO emulator_metadata (emulator_id, config_dir, bios_dir, saves_dir, states_dir, renderer)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'auto')
                 ON CONFLICT(emulator_id) DO NOTHING;",
                params![profile.id(), config_dir, bios_dir, saves_dir, states_dir]
            ).map_err(|e| EmuBoxError::StorageUnavailable(format!("Error guardando metadata de emulador: {}", e)))?;

            list.push(Emulator {
                id: profile.id().to_string(),
                name: profile.official_name().to_string(),
                version,
                supported_platforms: profile.supported_platforms().iter().map(|s| s.to_string()).collect(),
                core_type: profile.core_type().to_string(),
                status,
                executable,
                arguments: profile.default_arguments().iter().map(|s| s.to_string()).collect(),
            });
        }

        Ok(list)
    }

    /// Aplica el perfil de hardware detectado: cada emulador registrado en
    /// `services::emulators` resuelve y persiste su propio renderer óptimo (metadata
    /// SQLite + configuración nativa si la tiene implementada), sin intervención del
    /// usuario. Debe ejecutarse tras `scan_emulators` y cada vez que cambie el hardware
    /// (hotplug de GPU/monitor).
    pub fn apply_hardware_profile(hardware: &crate::models::HardwareInfo) -> Result<(), EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        let vulkan_ok = hardware.vulkan_driver_version.is_some() && hardware.gpu_vendor != "generic";

        for profile in emulators::registry() {
            let renderer = match profile.core_type() {
                "libretro" => if vulkan_ok { "vulkan" } else { "gl" },
                _ => if vulkan_ok { "vulkan" } else { "opengl" },
            };
            conn.execute(
                "UPDATE emulator_metadata SET renderer = ?1 WHERE emulator_id = ?2;",
                params![renderer, profile.id()]
            ).map_err(|e| EmuBoxError::StorageUnavailable(format!("Error aplicando perfil de hardware a {}: {}", profile.id(), e)))?;

            profile.apply_hardware_config(hardware)?;
        }

        Ok(())
    }

    pub fn get_emulators() -> Result<Vec<Emulator>, EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, official_name, version, supported_platforms_json, core_type, status, executable_path, default_arguments_json
             FROM emulators ORDER BY official_name ASC;"
        ).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;

        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let version: String = row.get(2)?;
            let platforms_str: String = row.get(3)?;
            let core_type: String = row.get(4)?;
            let status: String = row.get(5)?;
            let executable: String = row.get(6)?;
            let args_str: String = row.get(7)?;

            let supported_platforms: Vec<String> = serde_json::from_str(&platforms_str).unwrap_or_default();
            let arguments: Vec<String> = serde_json::from_str(&args_str).unwrap_or_default();

            Ok(Emulator {
                id,
                name,
                version,
                supported_platforms,
                core_type,
                status,
                executable,
                arguments,
            })
        }).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            if let Ok(emu) = r {
                list.push(emu);
            }
        }

        if list.is_empty() {
            return Self::scan_emulators();
        }

        Ok(list)
    }

    pub fn get_emulator_by_id(id: String) -> Result<Option<Emulator>, EmuBoxError> {
        let emulators = Self::get_emulators()?;
        Ok(emulators.into_iter().find(|e| e.id == id))
    }

    pub fn get_emulator_status(id: String) -> Result<String, EmuBoxError> {
        let emu = Self::get_emulator_by_id(id)?;
        Ok(emu.map(|e| e.status).unwrap_or_else(|| "not_found".to_string()))
    }

    pub fn save_emulator(emulator: Emulator) -> Result<(), EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        let platforms_json = serde_json::to_string(&emulator.supported_platforms).unwrap_or_else(|_| "[]".to_string());
        let args_json = serde_json::to_string(&emulator.arguments).unwrap_or_else(|_| "[]".to_string());

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
            .map_err(|e| EmuBoxError::StorageUnavailable(format!("Error al eliminar emulador de SQLite: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockTestProfile;
    impl EmulatorProfile for MockTestProfile {
        fn id(&self) -> &'static str { "test_emu" }
        fn official_name(&self) -> &'static str { "Test Emulator" }
        fn binary_candidates(&self) -> &'static [&'static str] { &["test_emu"] }
        fn supported_platforms(&self) -> &'static [&'static str] { &["snes"] }
        fn core_type(&self) -> &'static str { "standalone" }
        fn default_arguments(&self) -> &'static [&'static str] { &[] }
        fn version_flag(&self) -> &'static str { "--version" }
    }

    #[test]
    fn test_provision_dedicated_environment() {
        let temp_bin = std::env::temp_dir().join(format!("test-bin-{}", std::process::id()));
        let _ = std::fs::write(&temp_bin, "#!/bin/sh\necho test");

        let profile = MockTestProfile;
        let provisioned = EmulatorService::provision_dedicated_environment(&profile, &temp_bin);

        assert!(provisioned.to_string_lossy().contains("/var/lib/emubox/emulators/test_emu/bin/"));
        assert!(provisioned.is_file());

        let _ = std::fs::remove_file(&temp_bin);
    }
}
