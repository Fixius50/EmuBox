use crate::errors::EmuBoxError;
use crate::models::HardwareInfo;
use super::{config_home, upsert_ini_key, vulkan_ok, EmulatorProfile};

pub struct Pcsx2;

impl EmulatorProfile for Pcsx2 {
    fn id(&self) -> &'static str { "pcsx2" }
    fn official_name(&self) -> &'static str { "PCSX2" }
    fn binary_candidates(&self) -> &'static [&'static str] { &["pcsx2-qt", "pcsx2", "PCSX2.AppImage"] }
    fn supported_platforms(&self) -> &'static [&'static str] { &["ps2"] }
    fn core_type(&self) -> &'static str { "standalone" }
    fn default_arguments(&self) -> &'static [&'static str] { &["-fullscreen", "-batch"] }
    fn version_flag(&self) -> &'static str { "" }

    /// Verificado contra PCSX2 Qt (PCSX2.ini, sección [EmuCore/GS], clave Renderer).
    /// Asigna "Vulkan" cuando la GPU soporta Vulkan o "OpenGL" en caso contrario.
    fn apply_hardware_config(&self, hardware: &HardwareInfo) -> Result<(), EmuBoxError> {
        let renderer = if vulkan_ok(hardware) { "Vulkan" } else { "OpenGL" };
        let path = config_home().join("PCSX2/ini/PCSX2.ini");
        upsert_ini_key(&path, "EmuCore/GS", "Renderer", renderer)
    }
}
