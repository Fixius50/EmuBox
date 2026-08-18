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
}
