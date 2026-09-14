use super::EmulatorProfile;

pub struct Wine;

impl EmulatorProfile for Wine {
    fn id(&self) -> &'static str {
        "wine"
    }
    fn official_name(&self) -> &'static str {
        "Wine"
    }
    fn binary_candidates(&self) -> &'static [&'static str] {
        &["wine"]
    }
    fn supported_platforms(&self) -> &'static [&'static str] {
        &["pc"]
    }
    fn core_type(&self) -> &'static str {
        "standalone"
    }
    fn default_arguments(&self) -> &'static [&'static str] {
        &[]
    }
    fn version_flag(&self) -> &'static str {
        "--version"
    }
}
