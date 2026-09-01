use super::EmulatorProfile;

/// No disponible en repos oficiales de Arch; requiere AUR (build muy largo). No
/// instalado en este entorno todavía.
pub struct Rpcs3;

impl EmulatorProfile for Rpcs3 {
    fn id(&self) -> &'static str { "rpcs3" }
    fn official_name(&self) -> &'static str { "RPCS3" }
    fn binary_candidates(&self) -> &'static [&'static str] { &["rpcs3", "RPCS3.AppImage"] }
    fn supported_platforms(&self) -> &'static [&'static str] { &["ps3"] }
    fn core_type(&self) -> &'static str { "standalone" }
    fn default_arguments(&self) -> &'static [&'static str] { &["--no-gui"] }
    fn version_flag(&self) -> &'static str { "--version" }

    // TODO: implementar apply_hardware_config cuando se instale y se verifique su
    // archivo real (`config.yml`, clave `Video > Renderer`, "Vulkan" | "OpenGL").
}
