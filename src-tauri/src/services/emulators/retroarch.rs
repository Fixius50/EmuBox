use super::{config_home, upsert_flat_key, EmulatorProfile, RendererPreference};
use crate::errors::EmuBoxError;
use crate::models::HardwareInfo;
use std::path::{Path, PathBuf};

pub struct RetroArch;

fn configure_gamemode(path: &Path, virtual_machine: bool) -> Result<(), EmuBoxError> {
    if virtual_machine {
        upsert_flat_key(path, "gamemode_enable", "false")?;
    }
    Ok(())
}

impl EmulatorProfile for RetroArch {
    fn id(&self) -> &'static str {
        "retroarch"
    }
    fn official_name(&self) -> &'static str {
        "RetroArch"
    }
    fn binary_candidates(&self) -> &'static [&'static str] {
        &["retroarch"]
    }
    fn supported_platforms(&self) -> &'static [&'static str] {
        &[
            "snes", "genesis", "nes", "gba", "gb", "arcade", "n64", "ps1",
        ]
    }
    fn core_type(&self) -> &'static str {
        "libretro"
    }
    fn default_arguments(&self) -> &'static [&'static str] {
        &["-f"]
    }
    fn version_flag(&self) -> &'static str {
        "--version"
    }

    /// Verificado: convención estable `video_driver` de libretro/RetroArch, en
    /// `retroarch.cfg` (formato plano `clave = "valor"`, sin secciones). Gobierna también
    /// los cores libretro instalados (flycast, melonds, ppsspp, dolphin, mgba), ya que
    /// todos renderizan a través del video driver del frontend.
    fn apply_hardware_config(
        &self,
        hardware: &HardwareInfo,
        renderer: RendererPreference,
    ) -> Result<(), EmuBoxError> {
        let path = config_home().join("retroarch/config/retroarch.cfg");
        match renderer {
            RendererPreference::Vulkan => upsert_flat_key(&path, "video_driver", "vulkan")?,
            RendererPreference::OpenGl => upsert_flat_key(&path, "video_driver", "glcore")?,
            RendererPreference::Conservative => (),
        }
        configure_gamemode(&path, hardware.is_virtual_machine)?;
        if hardware.is_virtual_machine {
            let user_config = std::env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
                .or_else(|| {
                    std::env::var_os("HOME")
                        .map(PathBuf::from)
                        .filter(|path| path.is_absolute())
                        .map(|home| home.join(".config"))
                });
            if let Some(user_config) = user_config {
                configure_gamemode(&user_config.join("retroarch/retroarch.cfg"), true)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn vm_disables_gamemode_without_changing_native_preferences_on_physical_hosts() {
        let directory =
            std::env::temp_dir().join(format!("emubox-retroarch-gamemode-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let config = directory.join("retroarch.cfg");
        configure_gamemode(&config, false).unwrap();
        assert!(!config.exists());
        let original =
            "gamemode_enable = \"true\"\nvideo_driver = \"glcore\"\ninput_max_users = \"4\"\n";
        fs::write(&config, original).unwrap();
        configure_gamemode(&config, false).unwrap();
        assert_eq!(fs::read_to_string(&config).unwrap(), original);
        configure_gamemode(&config, true).unwrap();
        let updated = fs::read_to_string(&config).unwrap();
        assert_eq!(
            updated,
            original.replace("gamemode_enable = \"true\"", "gamemode_enable = \"false\"")
        );
        configure_gamemode(&config, true).unwrap();
        assert_eq!(fs::read_to_string(&config).unwrap(), updated);
        fs::remove_dir_all(directory).unwrap();
    }
}
