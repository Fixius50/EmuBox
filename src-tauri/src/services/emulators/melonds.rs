use super::EmulatorProfile;

/// AUR (`libretro-melonds` en repos oficiales para uso como core; binario standalone vía AUR).
/// No instalado en este entorno todavía.
pub struct MelonDs;

impl EmulatorProfile for MelonDs {
    fn id(&self) -> &'static str { "melonds" }
    fn official_name(&self) -> &'static str { "melonDS" }
    fn binary_candidates(&self) -> &'static [&'static str] { &["melonds", "melonDS"] }
    fn supported_platforms(&self) -> &'static [&'static str] { &["nds"] }
    fn core_type(&self) -> &'static str { "standalone" }
    fn default_arguments(&self) -> &'static [&'static str] { &["-f"] }
    fn version_flag(&self) -> &'static str { "--version" }

    // TODO: implementar apply_hardware_config cuando se instale y se verifique
    // su archivo de configuración real (melonDS.ini / Renderer3D). No adivinar la
    // clave sin poder confirmarla contra el binario instalado.
}
