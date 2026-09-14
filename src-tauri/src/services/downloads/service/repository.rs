use super::DownloadService;
use crate::errors::EmuBoxError;
use crate::{
    models::{
        CreateDownloadRequest, DownloadJob, DownloadSource, DownloadSourceType, DownloadStatus,
    },
    services::{db_service::DatabaseService, paths},
};
use rusqlite::{params, OptionalExtension};
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};
static JOB_CREATION: Mutex<()> = Mutex::new(());

impl DownloadService {
    pub fn create_source(source: DownloadSource) -> Result<DownloadSource, EmuBoxError> {
        let connection = DatabaseService::get_connection()?;
        Self::create_source_on(&connection, source)
    }

    pub(super) fn create_source_on(
        conn: &rusqlite::Connection,
        source: DownloadSource,
    ) -> Result<DownloadSource, EmuBoxError> {
        if source.id.trim().is_empty()
            || source.game_id.trim().is_empty()
            || source.name.trim().is_empty()
        {
            return Err(EmuBoxError::InvalidConfiguration(
                "La fuente necesita id, gameId y nombre".to_string(),
            ));
        }
        let owner: Option<String> = conn
            .query_row(
                "SELECT game_id FROM download_sources WHERE id=?1",
                params![source.id],
                |row| row.get(0),
            )
            .optional()
            .map_err(crate::services::download_providers::io_error)?;
        if owner.is_some_and(|owner| owner != source.game_id) {
            return Err(EmuBoxError::InvalidConfiguration(
                "No se puede reasignar una fuente a otro juego".into(),
            ));
        }
        let source_type_str = match &source.source_type {
            DownloadSourceType::Http => "http",
            DownloadSourceType::Torrent => "torrent",
            DownloadSourceType::Magnet => "magnet",
            DownloadSourceType::Other => "other",
        };
        conn.execute(
            "INSERT INTO download_sources (id, game_id, name, source_type, uri, size_bytes, checksum, available)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET game_id = excluded.game_id, name = excluded.name, source_type = excluded.source_type, uri = excluded.uri, size_bytes = excluded.size_bytes, checksum = excluded.checksum, available = excluded.available;",
            params![source.id, source.game_id, source.name, source_type_str, source.uri, source.size_bytes, source.checksum, if source.available { 1 } else { 0 }],
        ).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        Ok(source)
    }

    pub fn create_job(request: CreateDownloadRequest) -> Result<DownloadJob, EmuBoxError> {
        let _guard = JOB_CREATION
            .lock()
            .map_err(crate::services::download_providers::io_error)?;
        let source = request.source;
        let option = crate::services::manifest_service::source_option(source.clone());
        if !option.downloadable {
            return Err(EmuBoxError::InvalidConfiguration(
                option
                    .reason
                    .unwrap_or_else(|| "Fuente no disponible".into()),
            ));
        }
        if source.game_id != request.game_id {
            return Err(EmuBoxError::InvalidConfiguration(
                "La fuente no pertenece al juego solicitado".into(),
            ));
        }
        let platform: String = DatabaseService::get_connection()?
            .query_row(
                "SELECT platform_id FROM games WHERE id=?1",
                params![request.game_id],
                |row| row.get(0),
            )
            .map_err(|_| EmuBoxError::NotFound("Juego de catalogo inexistente".into()))?;
        if platform != request.platform {
            return Err(EmuBoxError::InvalidConfiguration(
                "La plataforma no coincide con el juego".into(),
            ));
        }
        let provider = crate::services::download_resolver::resolve(&source)?;
        Self::create_source(source.clone())?;
        Self::destination_path(&request.platform, &source.uri)?;
        let existing = DatabaseService::get_connection()?.query_row(
            "SELECT id FROM download_jobs WHERE source_id = ?1 AND status IN ('queued', 'downloading', 'paused', 'completed', 'downloaded') ORDER BY rowid DESC LIMIT 1",
            params![source.id],
            |row| row.get::<_, String>(0),
        ).ok();
        if let Some(existing_id) = existing {
            if let Some(job) = Self::get_job(&existing_id)? {
                if !matches!(
                    job.status,
                    DownloadStatus::Completed | DownloadStatus::Downloaded
                ) || Path::new(&job.destination_path).exists()
                {
                    return Ok(job);
                }
            }
        }
        let id = format!("download-{}", uuid_like());
        let destination = crate::services::download_manager::content_root()
            .join(&request.platform)
            .join(&id);
        let mut conn = DatabaseService::get_connection()?;
        let transaction = conn
            .transaction()
            .map_err(crate::services::download_providers::io_error)?;
        transaction.execute(
            "INSERT INTO download_jobs (id, game_id, source_id, platform, destination_path, status, total_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5, 'queued', ?6);",
            params![id, request.game_id, source.id, request.platform, destination.to_string_lossy().to_string(), source.size_bytes],
        ).map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        transaction
            .execute(
                "INSERT INTO download_execution(job_id,source_json,provider) VALUES (?1,?2,?3)",
                params![
                    id,
                    serde_json::to_string(&source)
                        .map_err(crate::services::download_providers::io_error)?,
                    provider.as_str()
                ],
            )
            .map_err(crate::services::download_providers::io_error)?;
        transaction
            .commit()
            .map_err(crate::services::download_providers::io_error)?;
        Self::get_job(&id)?.ok_or_else(|| {
            EmuBoxError::Unknown("No se pudo crear el trabajo de descarga".to_string())
        })
    }

    pub fn list_jobs() -> Result<Vec<DownloadJob>, EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        let mut stmt = conn.prepare("SELECT id, game_id, source_id, platform, destination_path, status, progress, downloaded_bytes, total_bytes, speed_bytes_per_second, error, (SELECT provider FROM download_execution WHERE job_id=id), (SELECT phase FROM download_execution WHERE job_id=id) FROM download_jobs ORDER BY rowid DESC")
            .map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        let rows = stmt
            .query_map([], Self::row_to_job)
            .map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(crate::services::download_providers::io_error)
    }

    pub fn list_sources(
        game_id: &str,
    ) -> Result<Vec<crate::services::manifest_service::SourceOption>, EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        let mut statement = conn.prepare("SELECT id, game_id, name, uri, size_bytes, checksum, available FROM download_sources WHERE game_id = ?1 ORDER BY rowid")
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        let rows = statement
            .query_map(params![game_id], |row| {
                let uri: String = row.get(3)?;
                let source_type = match crate::services::manifest_service::source_access(&uri) {
                    Some("magnet") => DownloadSourceType::Magnet,
                    Some("torrent") => DownloadSourceType::Torrent,
                    Some("unsupported") | None => DownloadSourceType::Other,
                    _ => DownloadSourceType::Http,
                };
                Ok(crate::services::manifest_service::source_option(
                    DownloadSource {
                        id: row.get(0)?,
                        game_id: row.get(1)?,
                        name: row.get(2)?,
                        uri,
                        source_type,
                        size_bytes: row.get(4)?,
                        checksum: row.get(5)?,
                        available: row.get::<_, i64>(6)? != 0,
                    },
                ))
            })
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))
    }

    pub fn get_job(id: &str) -> Result<Option<DownloadJob>, EmuBoxError> {
        let conn = DatabaseService::get_connection()?;
        let mut stmt = conn.prepare("SELECT id, game_id, source_id, platform, destination_path, status, progress, downloaded_bytes, total_bytes, speed_bytes_per_second, error, (SELECT provider FROM download_execution WHERE job_id=id), (SELECT phase FROM download_execution WHERE job_id=id) FROM download_jobs WHERE id = ?1")
            .map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        let mut rows = stmt
            .query_map(params![id], Self::row_to_job)
            .map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))?;
        rows.next()
            .transpose()
            .map_err(|e| EmuBoxError::StorageUnavailable(e.to_string()))
    }

    pub(super) fn destination_path(platform: &str, uri: &str) -> Result<PathBuf, EmuBoxError> {
        if !platform
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            || platform.is_empty()
        {
            return Err(EmuBoxError::InvalidConfiguration(
                "Plataforma inválida".to_string(),
            ));
        }
        let raw_filename = if uri.starts_with("magnet:") {
            decode_magnet_dn(uri)
        } else {
            let url = reqwest::Url::parse(uri)
                .map_err(|e| EmuBoxError::InvalidConfiguration(e.to_string()))?;
            url.path_segments()
                .and_then(|mut segments| segments.rfind(|s| !s.is_empty()))
                .unwrap_or("download.bin")
                .to_string()
        };
        let filename = raw_filename.replace(['/', '\\'], "_");
        let filename = if filename.is_empty() {
            "download.bin".to_string()
        } else {
            filename
        };
        Ok(Path::new(&paths::games_dir()).join(platform).join(filename))
    }

    fn row_to_job(row: &rusqlite::Row<'_>) -> rusqlite::Result<DownloadJob> {
        let status: String = row.get(5)?;
        Ok(DownloadJob {
            id: row.get(0)?,
            game_id: row.get(1)?,
            source_id: row.get(2)?,
            platform: row.get(3)?,
            destination_path: row.get(4)?,
            status: match status.as_str() {
                "downloading" => DownloadStatus::Downloading,
                "paused" => DownloadStatus::Paused,
                "completed" => DownloadStatus::Completed,
                "downloaded" => DownloadStatus::Downloaded,
                "failed" => DownloadStatus::Failed,
                "cancelled" => DownloadStatus::Cancelled,
                _ => DownloadStatus::Queued,
            },
            progress: row.get(6)?,
            downloaded_bytes: row.get(7)?,
            total_bytes: row.get(8)?,
            speed_bytes_per_second: row.get(9)?,
            error: row.get(10)?,
            provider: row.get(11).unwrap_or(None),
            phase: row.get(12).unwrap_or(None),
        })
    }
}

fn uuid_like() -> String {
    format!(
        "{}-{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    )
}

fn decode_magnet_dn(uri: &str) -> String {
    let dn_part = uri.split('&').find_map(|p| {
        if let Some(val) = p.strip_prefix("dn=") {
            Some(val)
        } else if let Some(idx) = p.find("?dn=") {
            Some(&p[idx + 4..])
        } else {
            None
        }
    });
    if let Some(raw) = dn_part {
        let mut decoded = String::new();
        let mut chars = raw.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '%' {
                let h1 = chars.next();
                let h2 = chars.next();
                if let (Some(h1), Some(h2)) = (h1, h2) {
                    let hex: String = [h1, h2].iter().collect();
                    if let Ok(b) = u8::from_str_radix(&hex, 16) {
                        decoded.push(b as char);
                        continue;
                    }
                    decoded.push('%');
                    decoded.push(h1);
                    decoded.push(h2);
                } else {
                    decoded.push('%');
                    if let Some(h1) = h1 {
                        decoded.push(h1);
                    }
                }
            } else if c == '+' {
                decoded.push(' ');
            } else {
                decoded.push(c);
            }
        }
        if !decoded.trim().is_empty() {
            return decoded;
        }
    }
    "download.bin".to_string()
}
