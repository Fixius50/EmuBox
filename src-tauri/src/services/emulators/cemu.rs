use super::EmulatorProfile;

/// Binario instalado en el sistema (`/usr/bin/cemu`).
pub struct Cemu;

impl EmulatorProfile for Cemu {
    fn id(&self) -> &'static str { "cemu" }
    fn official_name(&self) -> &'static str { "Cemu" }
    fn binary_candidates(&self) -> &'static [&'static str] { &["cemu", "Cemu.AppImage"] }
    fn supported_platforms(&self) -> &'static [&'static str] { &["wiiu"] }
    fn core_type(&self) -> &'static str { "standalone" }
    fn default_arguments(&self) -> &'static [&'static str] { &["-f", "-g"] }
    fn version_flag(&self) -> &'static str { "--version" }
}
