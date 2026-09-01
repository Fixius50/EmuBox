use super::EmulatorProfile;

/// No disponible en repos oficiales de Arch; requiere AUR (build largo, sin binario -bin
/// confirmado). No instalado en este entorno todavía.
pub struct Pcsx2;

impl EmulatorProfile for Pcsx2 {
    fn id(&self) -> &'static str { "pcsx2" }
    fn official_name(&self) -> &'static str { "PCSX2" }
    fn binary_candidates(&self) -> &'static [&'static str] { &["pcsx2-qt", "pcsx2", "PCSX2.AppImage"] }
    fn supported_platforms(&self) -> &'static [&'static str] { &["ps2"] }
    fn core_type(&self) -> &'static str { "standalone" }
    fn default_arguments(&self) -> &'static [&'static str] { &["-fullscreen", "-batch"] }
    fn version_flag(&self) -> &'static str { "--version" }

    // TODO: implementar apply_hardware_config cuando se instale y se verifique su
    // archivo real (`PCSX2.ini`, sección `[EmuCore/GS]`, clave `Renderer`). La
    // codificación exacta de valores (enum GSRendererType) debe confirmarse contra
    // el binario instalado antes de escribirla automáticamente.
}
