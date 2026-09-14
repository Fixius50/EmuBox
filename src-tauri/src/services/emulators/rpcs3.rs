use super::EmulatorProfile;

/// Binario instalado en el sistema (`/usr/bin/rpcs3`).
pub struct Rpcs3;

impl EmulatorProfile for Rpcs3 {
    fn id(&self) -> &'static str { "rpcs3" }
    fn official_name(&self) -> &'static str { "RPCS3" }
    fn binary_candidates(&self) -> &'static [&'static str] { &["rpcs3", "RPCS3.AppImage", "/opt/emubox/bin/RPCS3.AppImage"] }
    fn supported_platforms(&self) -> &'static [&'static str] { &["ps3"] }
    fn core_type(&self) -> &'static str { "standalone" }
    fn default_arguments(&self) -> &'static [&'static str] { &["--no-gui"] }
    fn version_flag(&self) -> &'static str { "--version" }
    fn version_arguments(&self) -> Vec<&'static str> { vec!["--headless", "--version"] }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovery_uses_installed_appimage_and_headless_version() {
        assert!(Rpcs3.binary_candidates().contains(&"/opt/emubox/bin/RPCS3.AppImage"));
        assert_eq!(Rpcs3.version_arguments(), ["--headless", "--version"]);
        assert_eq!(Rpcs3.default_arguments(), ["--no-gui"]);
    }
}
