use super::{config_home, upsert_ini_key, EmulatorProfile, RendererPreference};
use crate::errors::EmuBoxError;
use crate::models::HardwareInfo;

/// Binario instalado en el sistema (`/usr/bin/duckstation-qt`).
pub struct DuckStation;

impl EmulatorProfile for DuckStation {
    fn id(&self) -> &'static str {
        "duckstation"
    }
    fn official_name(&self) -> &'static str {
        "DuckStation"
    }
    fn binary_candidates(&self) -> &'static [&'static str] {
        &[
            "duckstation-qt",
            "duckstation-nogui",
            "DuckStation.AppImage",
        ]
    }
    fn supported_platforms(&self) -> &'static [&'static str] {
        &["ps1"]
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

    /// Verificado contra DuckStation Qt (`settings.ini`, sección `[GPU]`, clave `Renderer`).
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
        let path = config_home().join("duckstation/config/settings.ini");
        upsert_ini_key(&path, "GPU", "Renderer", renderer)
    }
}
