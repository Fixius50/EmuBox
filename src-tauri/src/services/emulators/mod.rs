use crate::errors::EmuBoxError;
use crate::models::HardwareInfo;
use std::fs;
use std::path::{Path, PathBuf};

mod azahar;
mod cemu;
mod dolphin;
mod duckstation;
mod flycast;
mod libretro;
mod melonds;
mod mgba;
mod pcsx2;
mod ppsspp;
mod retroarch;
mod rpcs3;
mod ryujinx;
mod shadps4;
mod wine;

/// Un emulador = un archivo = un mantenedor. Cada implementación posee sus propios
/// binarios candidatos, plataformas soportadas y (si está verificada) su lógica de
/// configuración nativa según el hardware, de forma que se pueda auditar, corregir o
/// ampliar un emulador sin tocar el resto.
pub trait EmulatorProfile: Sync + Send {
    fn id(&self) -> &'static str;
    fn official_name(&self) -> &'static str;
    fn binary_candidates(&self) -> &'static [&'static str];
    fn supported_platforms(&self) -> &'static [&'static str];
    fn core_type(&self) -> &'static str;
    fn default_arguments(&self) -> &'static [&'static str];
    fn version_flag(&self) -> &'static str;
    fn version_arguments(&self) -> Vec<&'static str> {
        let flag = self.version_flag();
        if flag.trim().is_empty() {
            Vec::new()
        } else {
            vec![flag]
        }
    }

    /// Escribe/actualiza la configuración nativa según el backend operativo ya
    /// seleccionado por EmuBox. Por defecto no hace nada: solo se sobreescribe cuando
    /// la clave está verificada contra la configuración real del emulador.
    fn apply_hardware_config(
        &self,
        _hardware: &HardwareInfo,
        _renderer: RendererPreference,
    ) -> Result<(), EmuBoxError> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RendererPreference {
    OpenGl,
    Vulkan,
    Conservative,
}

impl RendererPreference {
    pub(crate) fn metadata_name(self, core_type: &str) -> &'static str {
        match (self, core_type) {
            (Self::OpenGl, "libretro") => "gl",
            (Self::OpenGl, _) => "opengl",
            (Self::Vulkan, _) => "vulkan",
            (Self::Conservative, _) => "auto",
        }
    }
}

/// Comparte la única decisión gráfica de EmuBox con los perfiles. Los backends
/// software o indeterminados no fuerzan una clave nativa: cada emulador conserva
/// su valor seguro hasta que exista una evidencia operativa acelerada.
pub(crate) fn renderer_preference(operational_backend: &str) -> RendererPreference {
    match operational_backend {
        "opengl" => RendererPreference::OpenGl,
        "vulkan" => RendererPreference::Vulkan,
        _ => RendererPreference::Conservative,
    }
}

pub fn registry() -> Vec<Box<dyn EmulatorProfile>> {
    let mut profiles: Vec<Box<dyn EmulatorProfile>> = vec![
        Box::new(pcsx2::Pcsx2),
        Box::new(duckstation::DuckStation),
        Box::new(dolphin::Dolphin),
        Box::new(retroarch::RetroArch),
        Box::new(ppsspp::Ppsspp),
        Box::new(mgba::Mgba),
        Box::new(melonds::MelonDs),
        Box::new(flycast::Flycast),
        Box::new(rpcs3::Rpcs3),
        Box::new(cemu::Cemu),
        Box::new(ryujinx::Ryujinx),
        Box::new(wine::Wine),
        Box::new(azahar::Azahar),
        Box::new(shadps4::ShadPs4),
    ];
    profiles.extend(libretro::profiles());
    profiles
}

pub(crate) fn config_home() -> PathBuf {
    PathBuf::from(crate::services::paths::emulators_dir())
}

/// Archivo de configuración gestionado y su destino relativo dentro del HOME
/// privado del sandbox. La tabla es cerrada: nunca acepta rutas de la UI o de
/// metadata SQLite para ampliar los montajes del juego.
pub(crate) fn managed_config(emulator_id: &str) -> Option<(PathBuf, PathBuf)> {
    let config = PathBuf::from(crate::services::paths::emulator_config_dir(emulator_id));
    match emulator_id {
        "retroarch" => Some((
            config.join("retroarch.cfg"),
            PathBuf::from(".config/retroarch/retroarch.cfg"),
        )),
        "pcsx2" => Some((
            config.join("PCSX2.ini"),
            PathBuf::from(".config/PCSX2/inis/PCSX2.ini"),
        )),
        "duckstation" => Some((
            config.join("settings.ini"),
            PathBuf::from(".config/duckstation/settings.ini"),
        )),
        "dolphin" => Some((
            config.join("Dolphin.ini"),
            PathBuf::from(".local/share/dolphin-emu/Config/Dolphin.ini"),
        )),
        "ppsspp" => Some((
            config.join("PSP/SYSTEM/ppsspp.ini"),
            PathBuf::from(".config/ppsspp/PSP/SYSTEM/ppsspp.ini"),
        )),
        _ => None,
    }
}

/// Inserta o reemplaza `key = "value"` en un archivo de configuración plano sin
/// secciones (formato de `retroarch.cfg`), preservando el resto del contenido.
pub(crate) fn upsert_flat_key(path: &Path, key: &str, value: &str) -> Result<(), EmuBoxError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            EmuBoxError::StorageUnavailable(format!("No se pudo crear {}: {}", parent.display(), e))
        })?;
    }

    let existing = fs::read_to_string(path).unwrap_or_default();
    let mut found = false;
    let mut lines: Vec<String> = existing
        .lines()
        .map(|l| {
            if l.trim_start().starts_with(&format!("{key} "))
                || l.trim_start().starts_with(&format!("{key}="))
            {
                found = true;
                format!("{key} = \"{value}\"")
            } else {
                l.to_string()
            }
        })
        .collect();

    if !found {
        lines.push(format!("{key} = \"{value}\""));
    }

    fs::write(path, lines.join("\n") + "\n").map_err(|e| {
        EmuBoxError::StorageUnavailable(format!("No se pudo escribir {}: {}", path.display(), e))
    })
}

/// Inserta o reemplaza `key = value` dentro de la sección `[section]` de un INI,
/// preservando el resto de secciones y claves ya presentes.
pub(crate) fn upsert_ini_key(
    path: &Path,
    section: &str,
    key: &str,
    value: &str,
) -> Result<(), EmuBoxError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            EmuBoxError::StorageUnavailable(format!("No se pudo crear {}: {}", parent.display(), e))
        })?;
    }

    let existing = fs::read_to_string(path).unwrap_or_default();
    let mut lines: Vec<String> = existing.lines().map(|l| l.to_string()).collect();
    let section_header = format!("[{section}]");

    let section_idx = lines.iter().position(|l| l.trim() == section_header);

    match section_idx {
        None => {
            if !lines.is_empty() && !lines.last().unwrap().is_empty() {
                lines.push(String::new());
            }
            lines.push(section_header);
            lines.push(format!("{key} = {value}"));
        }
        Some(start) => {
            let end = lines
                .iter()
                .skip(start + 1)
                .position(|l| l.trim_start().starts_with('['))
                .map(|i| start + 1 + i)
                .unwrap_or(lines.len());

            let key_idx = lines[start + 1..end]
                .iter()
                .position(|l| {
                    l.split('=')
                        .next()
                        .map(|k| k.trim() == key)
                        .unwrap_or(false)
                })
                .map(|i| start + 1 + i);

            match key_idx {
                Some(idx) => lines[idx] = format!("{key} = {value}"),
                None => lines.insert(end, format!("{key} = {value}")),
            }
        }
    }

    fs::write(path, lines.join("\n") + "\n").map_err(|e| {
        EmuBoxError::StorageUnavailable(format!("No se pudo escribir {}: {}", path.display(), e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_key_created_when_file_missing() {
        let dir = std::env::temp_dir().join(format!("emubox-test-{}", std::process::id()));
        let path = dir.join("retroarch.cfg");
        upsert_flat_key(&path, "video_driver", "vulkan").unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content.trim(), "video_driver = \"vulkan\"");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn flat_key_replaced_preserving_other_lines() {
        let dir = std::env::temp_dir().join(format!("emubox-test-{}", std::process::id() + 1));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("retroarch.cfg");
        fs::write(
            &path,
            "input_max_users = \"4\"\nvideo_driver = \"gl\"\nvideo_fullscreen = \"true\"\n",
        )
        .unwrap();
        upsert_flat_key(&path, "video_driver", "vulkan").unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("video_driver = \"vulkan\""));
        assert!(content.contains("input_max_users = \"4\""));
        assert!(content.contains("video_fullscreen = \"true\""));
        assert!(!content.contains("\"gl\""));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn ini_key_created_with_new_section() {
        let dir = std::env::temp_dir().join(format!("emubox-test-{}", std::process::id() + 2));
        let path = dir.join("Dolphin.ini");
        upsert_ini_key(&path, "Core", "GFXBackend", "Vulkan").unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content.trim(), "[Core]\nGFXBackend = Vulkan");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn ini_key_replaced_preserving_other_sections() {
        let dir = std::env::temp_dir().join(format!("emubox-test-{}", std::process::id() + 3));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("ppsspp.ini");
        fs::write(&path, "[General]\nFirstRun = False\n\n[Graphics]\nGraphicsBackend = 0\nVSync = True\n\n[CPU]\nCPUCore = 1\n").unwrap();
        upsert_ini_key(&path, "Graphics", "GraphicsBackend", "3").unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("GraphicsBackend = 3"));
        assert!(!content.contains("GraphicsBackend = 0"));
        assert!(content.contains("VSync = True"));
        assert!(content.contains("[CPU]"));
        assert!(content.contains("FirstRun = False"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn duckstation_and_pcsx2_hardware_config_test() {
        let dir = std::env::temp_dir().join(format!("emubox-test-{}", std::process::id() + 4));
        fs::create_dir_all(&dir).unwrap();
        let duck_path = dir.join("duckstation/settings.ini");
        let pcsx2_path = dir.join("PCSX2/ini/PCSX2.ini");

        upsert_ini_key(&duck_path, "GPU", "Renderer", "Vulkan").unwrap();
        upsert_ini_key(&pcsx2_path, "EmuCore/GS", "Renderer", "Vulkan").unwrap();

        let duck_content = fs::read_to_string(&duck_path).unwrap();
        let pcsx2_content = fs::read_to_string(&pcsx2_path).unwrap();

        assert!(duck_content.contains("[GPU]"));
        assert!(duck_content.contains("Renderer = Vulkan"));
        assert!(pcsx2_content.contains("[EmuCore/GS]"));
        assert!(pcsx2_content.contains("Renderer = Vulkan"));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn renderer_preference_follows_emubox_operational_backend() {
        assert_eq!(renderer_preference("opengl"), RendererPreference::OpenGl);
        assert_eq!(renderer_preference("vulkan"), RendererPreference::Vulkan);
        assert_eq!(
            renderer_preference("software"),
            RendererPreference::Conservative
        );
        assert_eq!(
            renderer_preference("auto"),
            RendererPreference::Conservative
        );
        assert_eq!(RendererPreference::OpenGl.metadata_name("libretro"), "gl");
        assert_eq!(
            RendererPreference::OpenGl.metadata_name("standalone"),
            "opengl"
        );
        assert_eq!(
            RendererPreference::Conservative.metadata_name("standalone"),
            "auto"
        );
    }

    #[test]
    fn managed_config_table_is_closed_and_confined_to_private_config() {
        for (id, expected) in [
            ("retroarch", ".config/retroarch/retroarch.cfg"),
            ("pcsx2", ".config/PCSX2/inis/PCSX2.ini"),
            ("duckstation", ".config/duckstation/settings.ini"),
            ("dolphin", ".local/share/dolphin-emu/Config/Dolphin.ini"),
            ("ppsspp", ".config/ppsspp/PSP/SYSTEM/ppsspp.ini"),
        ] {
            let (source, target) = managed_config(id).unwrap();
            assert!(source.starts_with(crate::services::paths::emulator_config_dir(id)));
            assert_eq!(target, PathBuf::from(expected));
        }
        assert!(managed_config("wine").is_none());
    }
}
