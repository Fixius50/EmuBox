use super::{managed_config_source, upsert_ini_key, EmulatorProfile, RendererPreference};
use crate::errors::EmuBoxError;
use crate::models::HardwareInfo;
use crate::services::runtime::sandbox_paths;

pub struct Dolphin;

impl EmulatorProfile for Dolphin {
    fn id(&self) -> &'static str {
        "dolphin"
    }
    fn official_name(&self) -> &'static str {
        "Dolphin Emulator"
    }
    fn binary_candidates(&self) -> &'static [&'static str] {
        &["dolphin-emu", "Dolphin.AppImage"]
    }
    fn supported_platforms(&self) -> &'static [&'static str] {
        &["gamecube", "wii"]
    }
    fn core_type(&self) -> &'static str {
        "standalone"
    }
    fn default_arguments(&self) -> &'static [&'static str] {
        &["-u", sandbox_paths::DOLPHIN_USER_DIRECTORY, "-b", "-e"]
    }
    fn version_flag(&self) -> &'static str {
        "--version"
    }

    /// Verificado contra Source/Core/Core/Config/MainSettings.cpp:
    /// `MAIN_GFX_BACKEND{{System::Main, "Core", "GFXBackend"}, ...}` y
    /// `VideoBackendBase::GetConfigName()` ("OGL" | "Vulkan" | "Software Renderer").
    /// Archivo: `Dolphin.ini`, sección `[Core]`.
    fn apply_hardware_config(
        &self,
        _hardware: &HardwareInfo,
        renderer: RendererPreference,
    ) -> Result<(), EmuBoxError> {
        let backend = match renderer {
            RendererPreference::Vulkan => "Vulkan",
            RendererPreference::OpenGl => "OGL",
            RendererPreference::Conservative => return Ok(()),
        };
        let path = managed_config_source(self.id())?;
        upsert_ini_key(&path, "Core", "GFXBackend", backend)
    }
}
