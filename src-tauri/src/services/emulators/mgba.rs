use super::EmulatorProfile;

pub struct Mgba;

impl EmulatorProfile for Mgba {
    fn id(&self) -> &'static str {
        "mgba"
    }
    fn official_name(&self) -> &'static str {
        "mGBA"
    }
    fn binary_candidates(&self) -> &'static [&'static str] {
        &["mgba-qt", "mgba"]
    }
    fn supported_platforms(&self) -> &'static [&'static str] {
        &["gba", "gbc", "gb"]
    }
    fn core_type(&self) -> &'static str {
        "standalone"
    }
    fn default_arguments(&self) -> &'static [&'static str] {
        &["-f"]
    }
    fn version_flag(&self) -> &'static str {
        "--version"
    }

    // Sin `apply_hardware_config`: mgba-qt no expone una elección de backend
    // Vulkan/OpenGL relevante (GBA/GBC/GB no requiere aceleración 3D), se usa el
    // no-op por defecto del trait.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qt_frontend_uses_supported_version_option() {
        assert_eq!(Mgba.version_arguments(), ["--version"]);
    }
}
