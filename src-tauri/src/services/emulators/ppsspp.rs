use crate::errors::EmuBoxError;
use crate::models::HardwareInfo;
use super::{config_home, upsert_ini_key, vulkan_ok, EmulatorProfile};

pub struct Ppsspp;

impl EmulatorProfile for Ppsspp {
    fn id(&self) -> &'static str { "ppsspp" }
    fn official_name(&self) -> &'static str { "PPSSPP" }
    fn binary_candidates(&self) -> &'static [&'static str] { &["ppsspp", "PPSSPPQt", "PPSSPPSDL"] }
    fn supported_platforms(&self) -> &'static [&'static str] { &["psp"] }
    fn core_type(&self) -> &'static str { "standalone" }
    fn default_arguments(&self) -> &'static [&'static str] { &["--fullscreen"] }
    fn version_flag(&self) -> &'static str { "--version" }

    /// Verificado contra Core/ConfigValues.h (`enum class GPUBackend { OPENGL = 0,
    /// DIRECT3D11 = 2, VULKAN = 3 }`) y Core/Config.cpp
    /// (`ConfigSetting("GraphicsBackend", SETTING(g_Config, iGPUBackend), ...)` dentro de
    /// `graphicsSettings[]`, registrado bajo la sección "Graphics" en `g_sectionMeta`).
    /// Archivo: `ppsspp.ini`, sección `[Graphics]`. Se escribe el valor numérico plano
    /// (sin el sufijo "(NOMBRE)" que añade el traductor de depuración) para que
    /// `TryParse` lo lea directamente como entero.
    fn apply_hardware_config(&self, hardware: &HardwareInfo) -> Result<(), EmuBoxError> {
        let backend = if vulkan_ok(hardware) { "3" } else { "0" };
        let path = config_home().join("ppsspp/config/PSP/SYSTEM/ppsspp.ini");
        upsert_ini_key(&path, "Graphics", "GraphicsBackend", backend)
    }
}
