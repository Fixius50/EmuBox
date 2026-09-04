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