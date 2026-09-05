use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Emulator {
    pub id: String,
    pub name: String,
    pub version: String,
    pub supported_platforms: Vec<String>,
    pub core_type: String,
    pub status: String,
    pub executable: String,
    pub arguments: Vec<String>,
    #[serde(default)]
    pub architectures: Vec<super::Architecture>,
    #[serde(default)]
    pub requirements: EmulatorRequirements,
    #[serde(default)]
    pub compatibility: EmulatorCompatibility,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EmulatorRequirements {
    pub min_cpu_cores: usize,
    pub min_memory_mb: u64,
    pub vulkan: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EmulatorCompatibility {
    pub status: String,
    pub reason: String,
    pub host_architecture: String,
    pub binary_architecture: Option<String>,
}
