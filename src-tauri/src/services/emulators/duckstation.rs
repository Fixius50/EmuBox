use super::EmulatorProfile;

/// No disponible en repos oficiales de Arch; requiere AUR. No instalado en este
/// entorno todavía.
pub struct DuckStation;

impl EmulatorProfile for DuckStation {
    fn id(&self) -> &'static str { "duckstation" }
    fn official_name(&self) -> &'static str { "DuckStation" }
    fn binary_candidates(&self) -> &'static [&'static str] { &["duckstation-qt", "duckstation-nogui", "DuckStation.AppImage"] }
    fn supported_platforms(&self) -> &'static [&'static str] { &["ps1"] }
    fn core_type(&self) -> &'static str { "standalone" }
    fn default_arguments(&self) -> &'static [&'static str] { &["-fullscreen", "-batch"] }
    fn version_flag(&self) -> &'static str { "--version" }

    // TODO: implementar apply_hardware_config cuando se instale. Candidato conocido:
    // sección `[GPU]` clave `Renderer` (Automatic/Vulkan/OpenGL/Software), pero la ruta
    // exacta del archivo (XDG_CONFIG_HOME vs XDG_DATA_HOME) debe confirmarse contra el
    // binario instalado antes de escribirla automáticamente.
}
