use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderId {
    Http,
    BitTorrent,
}

impl ProviderId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::BitTorrent => "bittorrent",
        }
    }
}

#[derive(Default, Clone)]
pub struct TransferControl {
    pub paused: Arc<AtomicBool>,
    pub cancelled: Arc<AtomicBool>,
}

impl TransferControl {
    pub fn interrupted(&self) -> bool {
        self.paused.load(Ordering::Relaxed) || self.cancelled.load(Ordering::Relaxed)
    }
}

pub struct TransferRequest<'a> {
    pub uri: &'a str,
    pub directory: &'a Path,
    pub filename: &'a str,
    pub control: &'a TransferControl,
    pub max_bytes: Option<u64>,
}

pub struct TransferProgress {
    pub downloaded: u64,
    pub total: Option<u64>,
    pub speed: u64,
}

pub enum TransferOutcome {
    Complete(Vec<PathBuf>),
    Interrupted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallationKind { Inno, Ps3 }

#[derive(Clone, Serialize, Deserialize)]
pub struct PreparedInstallation {
    pub kind: InstallationKind,
    pub root: PathBuf,
}

#[derive(Serialize, Deserialize)]
pub struct PublishedDownload {
    pub job_id: String,
    pub source_digest: String,
    pub files: Vec<PathBuf>,
    pub launch: Option<PathBuf>,
    pub preparation_reason: Option<String>,
    #[serde(default)]
    pub installation: Option<PreparedInstallation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DownloadSourceType {
    Http,
    Torrent,
    Magnet,
    Other,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadSourceOption {
    #[serde(flatten)]
    pub source: DownloadSource,
    pub access: String,
    pub downloadable: bool,
    pub reason: Option<String>,
    pub provider: Option<String>,
    pub connector: Option<String>,
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
    Downloaded,
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
    pub provider: Option<String>,
    pub phase: Option<String>,
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
