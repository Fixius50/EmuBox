use crate::errors::EmuBoxError;
use crate::models::HardwareInfo;
use super::{config_home, upsert_flat_key, vulkan_ok, EmulatorProfile};

pub struct RetroArch;

impl EmulatorProfile for RetroArch {
    fn id(&self) -> &'static str { "retroarch" }
    fn official_name(&self) -> &'static str { "RetroArch" }
    fn binary_candidates(&self) -> &'static [&'static str] { &["retroarch"] }
    fn supported_platforms(&self) -> &'static [&'static str] {
        &["snes", "genesis", "nes", "gba", "gb", "arcade", "n64", "ps1"]
    }
    fn core_type(&self) -> &'static str { "libretro" }
    fn default_arguments(&self) -> &'static [&'static str] { &["-f"] }
    fn version_flag(&self) -> &'static str { "--version" }

    /// Verificado: convención estable `video_driver` de libretro/RetroArch, en
    /// `retroarch.cfg` (formato plano `clave = "valor"`, sin secciones). Gobierna también
    /// los cores libretro instalados (flycast, melonds, ppsspp, dolphin, mgba), ya que
    /// todos renderizan a través del video driver del frontend.
    fn apply_hardware_config(&self, hardware: &HardwareInfo) -> Result<(), EmuBoxError> {
        let driver = if vulkan_ok(hardware) { "vulkan" } else { "glcore" };
        let path = config_home().join("retroarch/retroarch.cfg");
        upsert_flat_key(&path, "video_driver", driver)
    }
}
