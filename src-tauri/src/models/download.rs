use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DownloadSourceType {
    Http,
    Torrent,
    Magnet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadSource {
    pub id: String,
    pub game_id: String,
    pub name: String,
    pub source_type: DownloadSourceType,
    pub uri: String,
    pub size_bytes: Option<u64>,
    pub checksum: Option<String>,
    pub available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DownloadStatus {
    Queued,
    Downloading,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadJob {
    pub id: String,
    pub game_id: String,
    pub source_id: String,
    pub platform: String,
    pub destination_path: String,
    pub status: DownloadStatus,
    pub progress: f32,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub speed_bytes_per_second: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDownloadRequest {
    pub game_id: String,
    pub platform: String,
    pub source: DownloadSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HydraDownloadItem {
    pub title: String,
    pub uris: Vec<String>,
    #[serde(default)]
    pub file_size: Option<serde_json::Value>,
    #[serde(default)]
    pub upload_date: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HydraDownloadManifest {
    #[serde(default)]
    pub name: Option<String>,
    pub downloads: Vec<HydraDownloadItem>,
    #[serde(default)]
    pub platform: Option<String>,
}

impl HydraDownloadItem {
    pub fn parse_size_bytes(&self) -> Option<u64> {
        self.file_size.as_ref().and_then(parse_file_size_value)
    }
}

pub fn parse_file_size_value(value: &serde_json::Value) -> Option<u64> {
    match value {
        serde_json::Value::Number(n) => n.as_u64(),
        serde_json::Value::String(s) => parse_file_size_str(s),
        _ => None,
    }
}

pub fn parse_file_size_str(input: &str) -> Option<u64> {
    let s = input.trim();
    if s.is_empty() {
        return None;
    }
    if let Ok(bytes) = s.parse::<u64>() {
        return Some(bytes);
    }
    let mut num_str = String::new();
    let mut unit_str = String::new();
    let mut found_unit = false;

    for c in s.chars() {
        if !found_unit && (c.is_ascii_digit() || c == '.' || c == ',') {
            if c == ',' {
                num_str.push('.');
            } else {
                num_str.push(c);
            }
        } else if c.is_alphabetic() {
            found_unit = true;
            unit_str.push(c.to_ascii_uppercase());
        }
    }

    let num: f64 = num_str.trim().parse().ok()?;
    let multiplier: f64 = match unit_str.trim() {
        "TB" | "TIB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        "GB" | "GIB" => 1024.0 * 1024.0 * 1024.0,
        "MB" | "MIB" => 1024.0 * 1024.0,
        "KB" | "KIB" => 1024.0,
        "B" | "BYTES" | "BYTE" => 1.0,
        _ => 1.0,
    };

    Some((num * multiplier) as u64)
}