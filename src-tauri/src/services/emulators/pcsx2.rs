use super::{managed_config_source, upsert_ini_key, EmulatorProfile, RendererPreference};
use crate::errors::EmuBoxError;
use crate::models::HardwareInfo;

pub struct Pcsx2;

impl EmulatorProfile for Pcsx2 {
    fn id(&self) -> &'static str {
        "pcsx2"
    }
    fn official_name(&self) -> &'static str {
        "PCSX2"
    }
    fn binary_candidates(&self) -> &'static [&'static str] {
        &["pcsx2-qt", "pcsx2", "PCSX2.AppImage"]
    }
    fn supported_platforms(&self) -> &'static [&'static str] {
        &["ps2"]
    }
    fn core_type(&self) -> &'static str {
        "standalone"
    }
    fn default_arguments(&self) -> &'static [&'static str] {
        &["-fullscreen", "-batch"]
    }
    fn version_flag(&self) -> &'static str {
        ""
    }

    /// Verificado contra PCSX2 Qt (PCSX2.ini, sección [EmuCore/GS], clave Renderer).
    /// Sigue el backend operativo de EmuBox; sin evidencia acelerada no fuerza renderer.
    fn apply_hardware_config(
        &self,
        _hardware: &HardwareInfo,
        renderer: RendererPreference,
    ) -> Result<(), EmuBoxError> {
        let renderer = match renderer {
            RendererPreference::Vulkan => "Vulkan",
            RendererPreference::OpenGl => "OpenGL",
            RendererPreference::Conservative => return Ok(()),
        };
        let path = managed_config_source(self.id())?;
        upsert_ini_key(&path, "EmuCore/GS", "Renderer", renderer)
    }
}
