use super::EmulatorProfile;

pub struct ShadPs4;
impl EmulatorProfile for ShadPs4 {
    fn id(&self) -> &'static str {
        "shadps4"
    }
    fn official_name(&self) -> &'static str {
        "shadPS4"
    }
    fn binary_candidates(&self) -> &'static [&'static str] {
        &["shadps4", "/opt/emubox/bin/Shadps4-sdl.AppImage"]
    }
    fn supported_platforms(&self) -> &'static [&'static str] {
        &["ps4"]
    }
    fn core_type(&self) -> &'static str {
        "standalone"
    }
    fn default_arguments(&self) -> &'static [&'static str] {
        &["--fullscreen", "true", "--game"]
    }
    fn version_flag(&self) -> &'static str {
        ""
    }
}
