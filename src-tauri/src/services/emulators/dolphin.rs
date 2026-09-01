use crate::errors::EmuBoxError;
use crate::models::HardwareInfo;
use super::{config_home, upsert_ini_key, vulkan_ok, EmulatorProfile};

pub struct Dolphin;

impl EmulatorProfile for Dolphin {
    fn id(&self) -> &'static str { "dolphin" }
    fn official_name(&self) -> &'static str { "Dolphin Emulator" }
    fn binary_candidates(&self) -> &'static [&'static str] { &["dolphin-emu", "Dolphin.AppImage"] }
    fn supported_platforms(&self) -> &'static [&'static str] { &["gamecube", "wii"] }
    fn core_type(&self) -> &'static str { "standalone" }
    fn default_arguments(&self) -> &'static [&'static str] { &["-b", "-e"] }
    fn version_flag(&self) -> &'static str { "--version" }

    /// Verificado contra Source/Core/Core/Config/MainSettings.cpp:
    /// `MAIN_GFX_BACKEND{{System::Main, "Core", "GFXBackend"}, ...}` y
    /// `VideoBackendBase::GetConfigName()` ("OGL" | "Vulkan" | "Software Renderer").
    /// Archivo: `Dolphin.ini`, sección `[Core]`.
    fn apply_hardware_config(&self, hardware: &HardwareInfo) -> Result<(), EmuBoxError> {
        let backend = if vulkan_ok(hardware) { "Vulkan" } else { "OGL" };
        let path = config_home().join("dolphin-emu/Dolphin.ini");
        upsert_ini_key(&path, "Core", "GFXBackend", backend)
    }
}
