use super::EmulatorProfile;

pub struct Azahar;
impl EmulatorProfile for Azahar {
    fn id(&self) -> &'static str {
        "azahar"
    }
    fn official_name(&self) -> &'static str {
        "Azahar (3DS)"
    }
    fn binary_candidates(&self) -> &'static [&'static str] {
        &["azahar", "/opt/emubox/bin/azahar.AppImage"]
    }
    fn supported_platforms(&self) -> &'static [&'static str] {
        &["3ds"]
    }
    fn core_type(&self) -> &'static str {
        "standalone"
    }
    fn default_arguments(&self) -> &'static [&'static str] {
        &[]
    }
    fn version_flag(&self) -> &'static str {
        ""
    }
}
