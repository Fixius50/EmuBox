use std::fs;
use std::path::{Path, PathBuf};
use crate::errors::EmuBoxError;
use crate::models::HardwareInfo;

mod cemu;
mod dolphin;
mod duckstation;
mod flycast;
mod melonds;
mod mgba;
mod pcsx2;
mod ppsspp;
mod retroarch;
mod rpcs3;
mod ryujinx;

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

    /// Escribe/actualiza la configuración nativa real del emulador según el hardware
    /// detectado (renderer, etc.). Por defecto no hace nada: solo se sobreescribe cuando
    /// la clave de configuración está verificada contra el código fuente oficial o un
    /// archivo de configuración real generado por el binario instalado.
    fn apply_hardware_config(&self, _hardware: &HardwareInfo) -> Result<(), EmuBoxError> {
        Ok(())
    }
}

pub fn registry() -> Vec<Box<dyn EmulatorProfile>> {
    vec![
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
    ]
}

pub(crate) fn config_home() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg);
        }
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/emubox".to_string());
    PathBuf::from(home).join(".config")
}

/// Inserta o reemplaza `key = "value"` en un archivo de configuración plano sin
/// secciones (formato de `retroarch.cfg`), preservando el resto del contenido.
pub(crate) fn upsert_flat_key(path: &Path, key: &str, value: &str) -> Result<(), EmuBoxError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| EmuBoxError::StorageUnavailable(format!("No se pudo crear {}: {}", parent.display(), e)))?;
    }

    let existing = fs::read_to_string(path).unwrap_or_default();
    let mut found = false;
    let mut lines: Vec<String> = existing.lines().map(|l| {
        if l.trim_start().starts_with(&format!("{key} ")) || l.trim_start().starts_with(&format!("{key}=")) {
            found = true;
            format!("{key} = \"{value}\"")
        } else {
            l.to_string()
        }
    }).collect();

    if !found {
        lines.push(format!("{key} = \"{value}\""));
    }

    fs::write(path, lines.join("\n") + "\n")
        .map_err(|e| EmuBoxError::StorageUnavailable(format!("No se pudo escribir {}: {}", path.display(), e)))
}

/// Inserta o reemplaza `key = value` dentro de la sección `[section]` de un INI,
/// preservando el resto de secciones y claves ya presentes.
pub(crate) fn upsert_ini_key(path: &Path, section: &str, key: &str, value: &str) -> Result<(), EmuBoxError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| EmuBoxError::StorageUnavailable(format!("No se pudo crear {}: {}", parent.display(), e)))?;
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
            let end = lines.iter().skip(start + 1)
                .position(|l| l.trim_start().starts_with('['))
                .map(|i| start + 1 + i)
                .unwrap_or(lines.len());

            let key_idx = lines[start + 1..end].iter().position(|l| {
                l.split('=').next().map(|k| k.trim() == key).unwrap_or(false)
            }).map(|i| start + 1 + i);

            match key_idx {
                Some(idx) => lines[idx] = format!("{key} = {value}"),
                None => lines.insert(end, format!("{key} = {value}")),
            }
        }
    }

    fs::write(path, lines.join("\n") + "\n")
        .map_err(|e| EmuBoxError::StorageUnavailable(format!("No se pudo escribir {}: {}", path.display(), e)))
}

/// `true` si hay un driver Vulkan real detectado sobre un vendor de GPU conocido
/// (no una VM sin passthrough ni un renderer software).
pub(crate) fn vulkan_ok(hardware: &HardwareInfo) -> bool {
    hardware.vulkan_driver_version.is_some() && hardware.gpu_vendor != "generic"
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
        fs::write(&path, "input_max_users = \"4\"\nvideo_driver = \"gl\"\nvideo_fullscreen = \"true\"\n").unwrap();
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
}
