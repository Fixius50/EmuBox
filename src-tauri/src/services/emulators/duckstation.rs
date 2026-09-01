use crate::errors::EmuBoxError;
use crate::models::HardwareInfo;
use super::{config_home, upsert_ini_key, vulkan_ok, EmulatorProfile};

/// Binario instalado en el sistema (`/usr/bin/duckstation-qt`).
pub struct DuckStation;

impl EmulatorProfile for DuckStation {
    fn id(&self) -> &'static str { "duckstation" }
    fn official_name(&self) -> &'static str { "DuckStation" }
    fn binary_candidates(&self) -> &'static [&'static str] { &["duckstation-qt", "duckstation-nogui", "DuckStation.AppImage"] }
    fn supported_platforms(&self) -> &'static [&'static str] { &["ps1"] }
    fn core_type(&self) -> &'static str { "standalone" }
    fn default_arguments(&self) -> &'static [&'static str] { &["-fullscreen", "-batch"] }
    fn version_flag(&self) -> &'static str { "--version" }

    /// Verificado contra DuckStation Qt (`settings.ini`, sección `[GPU]`, clave `Renderer`).
    /// Asigna "Vulkan" cuando la GPU soporta Vulkan o "OpenGL" en caso contrario.
    fn apply_hardware_config(&self, hardware: &HardwareInfo) -> Result<(), EmuBoxError> {
        let renderer = if vulkan_ok(hardware) { "Vulkan" } else { "OpenGL" };
        let path = config_home().join("duckstation/settings.ini");
        upsert_ini_key(&path, "GPU", "Renderer", renderer)
    }
}
