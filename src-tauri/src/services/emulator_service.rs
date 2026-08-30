use std::path::{Path, PathBuf};
use std::process::Command;
use rusqlite::params;
use crate::models::Emulator;
use crate::errors::EmuBoxError;
use crate::services::db_service::DatabaseService;

struct OfficialEmulatorDef {
    id: &'static str,
    official_name: &'static str,
    binary_candidates: &'static [&'static str],
    supported_platforms: &'static [&'static str],
    core_type: &'static str,
    default_arguments: &'static [&'static str],
    version_flag: &'static str,
}

const OFFICIAL_EMULATORS: &[OfficialEmulatorDef] = &[
    OfficialEmulatorDef {
        id: "pcsx2",
        official_name: "PCSX2",
        binary_candidates: &["pcsx2-qt", "pcsx2", "PCSX2.AppImage"],
        supported_platforms: &["ps2"],
        core_type: "standalone",
        default_arguments: &["-fullscreen", "-batch"],
        version_flag: "--version",
    },
    OfficialEmulatorDef {
        id: "duckstation",
        official_name: "DuckStation",
        binary_candidates: &["duckstation-qt", "duckstation-nogui", "DuckStation.AppImage"],
        supported_platforms: &["ps1"],
        core_type: "standalone",
        default_arguments: &["-fullscreen", "-batch"],
        version_flag: "--version",
    },
    OfficialEmulatorDef {
        id: "dolphin",
        official_name: "Dolphin Emulator",
        binary_candidates: &["dolphin-emu", "Dolphin.AppImage"],
        supported_platforms: &["gamecube", "wii"],
        core_type: "standalone",
        default_arguments: &["-b", "-e"],
        version_flag: "--version",
    },
    OfficialEmulatorDef {
        id: "retroarch",
        official_name: "RetroArch",
        binary_candidates: &["retroarch"],
        supported_platforms: &["snes", "genesis", "nes", "gba", "gb", "arcade", "n64", "ps1"],
        core_type: "libretro",
        default_arguments: &["-f"],
        version_flag: "--version",
    },
    OfficialEmulatorDef {
        id: "ppsspp",
        official_name: "PPSSPP",
        binary_candidates: &["ppsspp", "PPSSPPQt", "PPSSPPSDL"],
        supported_platforms: &["psp"],
        core_type: "standalone",
        default_arguments: &["--fullscreen"],
        version_flag: "--version",
    },
    OfficialEmulatorDef {
        id: "mgba",
        official_name: "mGBA",
        binary_candidates: &["mgba-qt", "mgba"],
        supported_platforms: &["gba", "gbc", "gb"],
        core_type: "standalone",
        default_arguments: &["-f"],
        version_flag: "-v",
    },
    OfficialEmulatorDef {
        id: "melonds",
        official_name: "melonDS",
        binary_candidates: &["melonds", "melonDS"],
        supported_platforms: &["nds"],
        core_type: "standalone",
        default_arguments: &["-f"],
        version_flag: "--version",
    },
    OfficialEmulatorDef {
        id: "flycast",
        official_name: "Flycast",
        binary_candidates: &["flycast", "Flycast.AppImage"],
        supported_platforms: &["dreamcast", "arcade"],
        core_type: "standalone",
        default_arguments: &["-config", "window:fullscreen=1"],
        version_flag: "--version",
    },
    OfficialEmulatorDef {
        id: "rpcs3",
        official_name: "RPCS3",
        binary_candidates: &["rpcs3", "RPCS3.AppImage"],
        supported_platforms: &["ps3"],
        core_type: "standalone",
        default_arguments: &["--no-gui"],
        version_flag: "--version",
    },
    OfficialEmulatorDef {
        id: "cemu",
        official_name: "Cemu",
        binary_candidates: &["cemu", "Cemu.AppImage"],
        supported_platforms: &["wiiu"],
        core_type: "standalone",
        default_arguments: &["-f", "-g"],
        version_flag: "--version",
    },
    OfficialEmulatorDef {
        id: "ryujinx",
        official_name: "Ryujinx",
        binary_candidates: &["ryujinx", "Ryujinx.AppImage"],
        supported_platforms: &["switch"],
        core_type: "standalone",
        default_arguments: &["--fullscreen"],
        version_flag: "--version",
    },
];

pub struct EmulatorService;

impl EmulatorService {
    fn find_binary_path(def: &OfficialEmulatorDef) -> Option<PathBuf> {
        let emu_dir = Path::new("/var/lib/emubox/emulators");
        for candidate in def.binary_candidates {
            let nested_path = emu_dir.join(def.id).join(candidate);
            if nested_path.is_file() {
                return Some(nested_path);
            }
            let direct_path = emu_dir.join(candidate);
            if direct_path.is_file() {
                return Some(direct_path);
            }
        }

        for candidate in def.binary_candidates {
            for prefix in &["/usr/local/bin", "/usr/bin", "/opt"] {
                let p = Path::new(prefix).join(candidate);
                if p.is_file() {
                    return Some(p);
                }
            }
        }

        for candidate in def.binary_candidates {
            if let Ok(output) = Command::new("which").arg(candidate).output() {
                if output.status.success() {
                    let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path_str.is_empty() {
                        let p = PathBuf::from(path_str);
                        if p.is_file() {
                            return Some(p);
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

        for def in OFFICIAL_EMULATORS {
            let (status, executable, version) = if let Some(binary_path) = Self::find_binary_path(def) {
                let raw_version = Self::probe_official_version(&binary_path, def.version_flag);
                ("active".to_string(), binary_path.to_string_lossy().to_string(), raw_version)
            } else {
                ("inactive".to_string(), "".to_string(), "No instalado".to_string())
            };

            let platforms_json = serde_json::to_string(&def.supported_platforms).unwrap_or_else(|_| "[]".to_string());
            let args_json = serde_json::to_string(&def.default_arguments).unwrap_or_else(|_| "[]".to_string());

            conn.execute(
                "INSERT INTO emulators (id, official_name, version, supported_platforms_json, core_type, status, executable_path, default_arguments_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(id) DO UPDATE SET
                   version = excluded.version,
                   status = excluded.status,
                   executable_path = excluded.executable_path,
                   default_arguments_json = excluded.default_arguments_json;",
                params![def.id, def.official_name, version, platforms_json, def.core_type, status, executable, args_json]
            ).map_err(|e| EmuBoxError::StorageUnavailable(format!("Error guardando emulador en SQLite: {}", e)))?;

            // Metadata específica por emulador
            let config_dir = format!("/var/lib/emubox/emulators/{}/config", def.id);
            let bios_dir = "/var/lib/emubox/bios".to_string();
            let saves_dir = "/var/lib/emubox/saves".to_string();
            let states_dir = "/var/lib/emubox/states".to_string();

            conn.execute(
                "INSERT INTO emulator_metadata (emulator_id, config_dir, bios_dir, saves_dir, states_dir, renderer)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'auto')
                 ON CONFLICT(emulator_id) DO NOTHING;",
                params![def.id, config_dir, bios_dir, saves_dir, states_dir]
            ).map_err(|e| EmuBoxError::StorageUnavailable(format!("Error guardando metadata de emulador: {}", e)))?;

            list.push(Emulator {
                id: def.id.to_string(),
                name: def.official_name.to_string(),
                version,
                supported_platforms: def.supported_platforms.iter().map(|s| s.to_string()).collect(),
                core_type: def.core_type.to_string(),
                status,
                executable,
                arguments: def.default_arguments.iter().map(|s| s.to_string()).collect(),
            });
        }

        Ok(list)
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
