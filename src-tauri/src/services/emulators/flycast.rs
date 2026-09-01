use super::EmulatorProfile;

/// AUR (`libretro-flycast` en repos oficiales para uso como core; binario standalone vía AUR).
/// No instalado en este entorno todavía.
pub struct Flycast;

impl EmulatorProfile for Flycast {
    fn id(&self) -> &'static str { "flycast" }
    fn official_name(&self) -> &'static str { "Flycast" }
    fn binary_candidates(&self) -> &'static [&'static str] { &["flycast", "Flycast.AppImage"] }
    fn supported_platforms(&self) -> &'static [&'static str] { &["dreamcast", "arcade"] }
    fn core_type(&self) -> &'static str { "standalone" }
    fn default_arguments(&self) -> &'static [&'static str] { &["-config", "window:fullscreen=1"] }
    fn version_flag(&self) -> &'static str { "--version" }

    // TODO: implementar apply_hardware_config cuando se instale y se verifique
    // su archivo de configuración real (emu.cfg / pvr.rend).
}
