use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Instant;
use reqwest::blocking::Client;
use sha2::{Digest, Sha256};
use rusqlite::params;
use crate::errors::EmuBoxError;
use crate::models::{CreateDownloadRequest, DownloadJob, DownloadSource, DownloadSourceType, DownloadStatus};
use crate::services::db_service::DatabaseService;

const DOWNLOAD_ROOT: &str = "/var/lib/emubox/games";
const DOWNLOAD_CACHE_ROOT: &str = "/var/cache/emubox/downloads";

struct RuntimeControl {
    paused: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
}

static ACTIVE_JOBS: OnceLock<Mutex<HashMap<String, RuntimeControl>>> = OnceLock::new();

fn active_jobs() -> &'static Mutex<HashMap<String, RuntimeControl>> {
    ACTIVE_JOBS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub struct DownloadService;

impl DownloadService {
    pub fn create_source(source: DownloadSource) -> Result<DownloadSource, EmuBoxError> {
        if source.id.trim().is_empty() || source.game_id.trim().is_empty() || source.name.trim().is_empty() {
            return Err(EmuBoxError::InvalidConfiguration("La fuente necesita id, gameId y nombre".to_string()));
        }
        if !matches!(&source.source_type, DownloadSourceType::Http) {
            return Err(EmuBoxError::InvalidConfiguration("Solo HTTP/HTTPS está habilitado; torrent requiere un adaptador explícito".to_string()));
        }
        let uri = reqwest::Url::parse(&source.uri)
            .map_err(|e| EmuBoxError::InvalidConfiguration(format!("URL inválida: {}", e)))?;
        if !matches!(uri.scheme(), "http" | "https") {
            return Err(EmuBoxError::InvalidConfiguration("La fuente debe usar HTTP o HTTPS".to_string()));
        }
        let conn = DatabaseService::get_connection()?;
        conn.execute(
            "INSERT INTO download_sources (id, game_id, name, source_type, uri, size_bytes, checksum, available)
             VALUES (?1, ?2, ?3, 'http', ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET name = excluded.name, uri = excluded.uri, size_bytes = excluded.size_bytes, checksum = excluded.checksum, available = excluded.available;",
            params![source.id, source.game_id, source.name, source.uri, source.size_bytes, source.checksum, if source.available { 1 } else { 0 }],
        ).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        Ok(source)
    }

    pub fn create_job(request: CreateDownloadRequest) -> Result<DownloadJob, EmuBoxError> {
        let source = request.source;
        Self::create_source(source.clone())?;
        let destination = Self::destination_path(&request.platform, &source.uri)?;
        let id = format!("download-{}", uuid_like());
        let conn = DatabaseService::get_connection()?;
        conn.execute(
            "INSERT INTO download_jobs (id, game_id, source_id, platform, destination_path, status, total_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5, 'queued', ?6);",
            params![id, request.game_id, source.id, request.platform, destination.to_string_lossy().to_string(), source.size_bytes],
        ).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        Self::get_job(&id)?.ok_or_else(|| EmuBoxError::Unknown("No se pudo crear el trabajo de descarga".to_string()))
    }

    pub fn list_jobs() -> Result<Vec<DownloadJob>, EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        let mut stmt = conn.prepare("SELECT id, game_id, source_id, platform, destination_path, status, progress, downloaded_bytes, total_bytes, speed_bytes_per_second, error FROM download_jobs ORDER BY rowid DESC")
            .map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        let rows = stmt.query_map([], Self::row_to_job).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        Ok(rows.flatten().collect())
    }

    pub fn get_job(id: &str) -> Result<Option<DownloadJob>, EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        let mut stmt = conn.prepare("SELECT id, game_id, source_id, platform, destination_path, status, progress, downloaded_bytes, total_bytes, speed_bytes_per_second, error FROM download_jobs WHERE id = ?1")
            .map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        let mut rows = stmt.query_map(params![id], Self::row_to_job).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        rows.next().transpose().map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))
    }

    pub fn start(id: String) -> Result<DownloadJob, EmuBoxError> {
        let job = Self::get_job(&id)?.ok_or_else(|| EmuBoxError::NotFound(format!("Descarga no encontrada: {}", id)))?;
        if !active_jobs().lock().unwrap().is_empty() {
            return Ok(job);
        }
        let (source_uri, checksum): (String, Option<String>) = DatabaseService::get_connection()?.query_row("SELECT uri, checksum FROM download_sources WHERE id = ?1", params![job.source_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        let paused = Arc::new(AtomicBool::new(false));
        let cancelled = Arc::new(AtomicBool::new(false));
        active_jobs().lock().unwrap().insert(id.clone(), RuntimeControl { paused: paused.clone(), cancelled: cancelled.clone() });
        Self::update_status(&id, DownloadStatus::Downloading, None)?;
        let worker_id = id.clone();
        thread::spawn(move || {
            let result = Self::run_http(worker_id.clone(), source_uri, checksum, job.destination_path, paused, cancelled);
            active_jobs().lock().unwrap().remove(&worker_id);
            if let Err(error) = result {
                let _ = Self::update_status(&worker_id, DownloadStatus::Failed, Some(error.to_string()));
            }
            Self::start_next_queued();
        });
        Self::get_job(&id)?.ok_or(EmuBoxError::NotFound(id))
    }

    pub fn pause(id: &str) -> Result<DownloadJob, EmuBoxError> {
        if let Some(control) = active_jobs().lock().unwrap().get(id) { control.paused.store(true, Ordering::Relaxed); }
        Self::update_status(id, DownloadStatus::Paused, None)?;
        Self::get_job(id)?.ok_or_else(|| EmuBoxError::NotFound(id.to_string()))
    }

    pub fn resume(id: String) -> Result<DownloadJob, EmuBoxError> {
        if let Some(control) = active_jobs().lock().unwrap().get(&id) { control.paused.store(false, Ordering::Relaxed); return Self::get_job(&id)?.ok_or(EmuBoxError::NotFound(id)); }
        Self::start(id)
    }

    pub fn cancel(id: &str) -> Result<DownloadJob, EmuBoxError> {
        if let Some(control) = active_jobs().lock().unwrap().get(id) { control.cancelled.store(true, Ordering::Relaxed); }
        Self::update_status(id, DownloadStatus::Cancelled, None)?;
        Self::get_job(id)?.ok_or_else(|| EmuBoxError::NotFound(id.to_string()))
    }

    fn run_http(id: String, uri: String, checksum: Option<String>, destination: String, paused: Arc<AtomicBool>, cancelled: Arc<AtomicBool>) -> Result<(), EmuBoxError> {
        let destination = PathBuf::from(destination);
        let parent = destination.parent().ok_or_else(|| EmuBoxError::InvalidConfiguration("Destino inválido".to_string()))?;
        fs::create_dir_all(parent).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        let partial_name = destination.file_name().and_then(|name| name.to_str()).unwrap_or("download.bin");
        let partial = Path::new(DOWNLOAD_CACHE_ROOT).join(format!("{}.part", partial_name));
        fs::create_dir_all(DOWNLOAD_CACHE_ROOT).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        let existing = partial.metadata().map(|m| m.len()).unwrap_or(0);
        let client = Client::new();
        let mut request = client.get(&uri);
        if existing > 0 { request = request.header(reqwest::header::RANGE, format!("bytes={}-", existing)); }
        let mut response = request.send().map_err(|e| EmuBoxError::Unknown(e.to_string()))?;
        let mut downloaded = if existing > 0 && response.status() == reqwest::StatusCode::PARTIAL_CONTENT { existing } else { 0 };
        if downloaded == 0 { response = client.get(&uri).send().map_err(|e| EmuBoxError::Unknown(e.to_string()))?; }
        if !response.status().is_success() { return Err(EmuBoxError::Unknown(format!("HTTP {}", response.status()))); }
        let total = response.content_length().map(|size| size + downloaded);
        let mut file = if downloaded > 0 { OpenOptions::new().append(true).open(&partial) } else { File::create(&partial) }
            .map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        let started = Instant::now();
        let mut buffer = [0u8; 64 * 1024];
        loop {
            if cancelled.load(Ordering::Relaxed) { return Ok(()); }
            if paused.load(Ordering::Relaxed) { thread::sleep(std::time::Duration::from_millis(100)); continue; }
            let read = response.read(&mut buffer).map_err(|e| EmuBoxError::Unknown(e.to_string()))?;
            if read == 0 { break; }
            file.write_all(&buffer[..read]).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
            downloaded += read as u64;
            let speed = (downloaded as f64 / started.elapsed().as_secs_f64().max(0.001)) as u64;
            Self::update_progress(&id, downloaded, total, speed)?;
        }
        fs::rename(&partial, &destination).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        if let Some(expected) = checksum {
            let mut file = File::open(&destination).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
            let mut hasher = Sha256::new();
            let mut hash_buffer = [0u8; 64 * 1024];
            loop {
                let read = file.read(&mut hash_buffer).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
                if read == 0 { break; }
                hasher.update(&hash_buffer[..read]);
            }
            let actual = format!("{:x}", hasher.finalize());
            if actual != expected.trim().to_ascii_lowercase() {
                let _ = fs::remove_file(&destination);
                return Err(EmuBoxError::Unknown("El checksum SHA-256 de la descarga no coincide".to_string()));
            }
        }
        Self::update_status(&id, DownloadStatus::Completed, None)?;
        Ok(())
    }

    fn destination_path(platform: &str, uri: &str) -> Result<PathBuf, EmuBoxError> {
        if !platform.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') || platform.is_empty() { return Err(EmuBoxError::InvalidConfiguration("Plataforma inválida".to_string())); }
        let url = reqwest::Url::parse(uri).map_err(|e| EmuBoxError::InvalidConfiguration(e.to_string()))?;
        let filename = url.path_segments().and_then(|mut segments| segments.rfind(|s| !s.is_empty())).unwrap_or("download.bin");
        let filename = filename.replace(['/', '\\'], "_");
        let filename = if filename.is_empty() { "download.bin" } else { filename.as_str() };
        Ok(Path::new(DOWNLOAD_ROOT).join(platform).join(filename))
    }

    fn update_status(id: &str, status: DownloadStatus, error: Option<String>) -> Result<(), EmuBoxError> {
        let value = match status { DownloadStatus::Queued => "queued", DownloadStatus::Downloading => "downloading", DownloadStatus::Paused => "paused", DownloadStatus::Completed => "completed", DownloadStatus::Failed => "failed", DownloadStatus::Cancelled => "cancelled" };
        DatabaseService::get_connection()?.execute("UPDATE download_jobs SET status = ?1, error = ?2 WHERE id = ?3", params![value, error, id]).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        Ok(())
    }

    fn update_progress(id: &str, downloaded: u64, total: Option<u64>, speed: u64) -> Result<(), EmuBoxError> {
        let progress = total.map(|total| (downloaded as f32 / total.max(1) as f32).min(1.0)).unwrap_or(0.0);
        DatabaseService::get_connection()?.execute("UPDATE download_jobs SET progress = ?1, downloaded_bytes = ?2, total_bytes = COALESCE(?3, total_bytes), speed_bytes_per_second = ?4 WHERE id = ?5", params![progress, downloaded, total, speed, id]).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        Ok(())
    }

    fn start_next_queued() {
        if active_jobs().lock().unwrap().is_empty() {
            if let Ok(jobs) = Self::list_jobs() {
                if let Some(next) = jobs.into_iter().find(|job| matches!(job.status, DownloadStatus::Queued)) {
                    let _ = Self::start(next.id);
                }
            }
        }
    }

    fn row_to_job(row: &rusqlite::Row<'_>) -> rusqlite::Result<DownloadJob> {
        let status: String = row.get(5)?;
        Ok(DownloadJob { id: row.get(0)?, game_id: row.get(1)?, source_id: row.get(2)?, platform: row.get(3)?, destination_path: row.get(4)?, status: match status.as_str() { "downloading" => DownloadStatus::Downloading, "paused" => DownloadStatus::Paused, "completed" => DownloadStatus::Completed, "failed" => DownloadStatus::Failed, "cancelled" => DownloadStatus::Cancelled, _ => DownloadStatus::Queued }, progress: row.get(6)?, downloaded_bytes: row.get(7)?, total_bytes: row.get(8)?, speed_bytes_per_second: row.get(9)?, error: row.get(10)? })
    }
}

fn uuid_like() -> String { format!("{}-{}", std::process::id(), chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()) }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destination_is_confined_to_platform_directory() {
        let destination = DownloadService::destination_path("ps3", "https://example.test/files/game.pkg").unwrap();
        assert_eq!(destination, Path::new(DOWNLOAD_ROOT).join("ps3/game.pkg"));
        assert!(DownloadService::destination_path("../ps3", "https://example.test/game.pkg").is_err());
    }

    #[test]
    fn torrent_sources_are_rejected_until_an_adapter_exists() {
        let source = DownloadSource {
            id: "torrent-test".to_string(),
            game_id: "future-game".to_string(),
            name: "Authorized torrent".to_string(),
            source_type: DownloadSourceType::Torrent,
            uri: "magnet:?xt=urn:btih:test".to_string(),
            size_bytes: None,
            checksum: None,
            available: true,
        };
        assert!(DownloadService::create_source(source).is_err());
    }
}