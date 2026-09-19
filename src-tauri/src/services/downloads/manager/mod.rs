mod publication;
#[cfg(test)]
mod tests;
mod transfer;
pub use publication::{candidates, select_candidate};
use transfer::run;

use crate::services::{
    db_service::DatabaseService, download_providers::io_error, download_service::DownloadService,
};
use crate::{
    errors::EmuBoxError,
    models::{DownloadJob, DownloadSource, DownloadStatus, TransferControl},
};
use rusqlite::{params, OptionalExtension};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{atomic::Ordering, Mutex, OnceLock},
    thread,
};

static ACTIVE: OnceLock<Mutex<HashMap<String, TransferControl>>> = OnceLock::new();
fn active() -> &'static Mutex<HashMap<String, TransferControl>> {
    ACTIVE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn content_root() -> PathBuf {
    #[cfg(test)]
    return std::env::temp_dir().join(format!("emubox-downloads-tests-{}", std::process::id()));
    #[cfg(not(test))]
    PathBuf::from(crate::services::paths::games_dir())
}

fn clean_staging(job: &DownloadJob) -> Result<(), EmuBoxError> {
    if job.id.is_empty()
        || !job
            .id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        return Err(EmuBoxError::InvalidConfiguration(
            "Identificador de trabajo invalido".into(),
        ));
    }
    let destination = Path::new(&job.destination_path);
    let root = destination
        .parent()
        .ok_or_else(|| EmuBoxError::InvalidConfiguration("Destino invalido".into()))?
        .join(".emubox-staging")
        .join(&job.id);
    if root.exists() {
        fs::remove_dir_all(root).map_err(io_error)?;
    }
    Ok(())
}

pub fn recover() -> Result<(), EmuBoxError> {
    let _guard = active().lock().map_err(io_error)?;
    DatabaseService::get_connection()?.execute("UPDATE download_jobs SET status='paused', speed_bytes_per_second=0, error='Trabajo interrumpido; reanudar explicitamente' WHERE status IN ('downloading','queued')", []).map_err(io_error)?;
    Ok(())
}

fn status(id: &str, state: &str, phase: &str, error: Option<&str>) -> Result<(), EmuBoxError> {
    let mut connection = DatabaseService::get_connection()?;
    let transaction = connection.transaction().map_err(io_error)?;
    transaction
        .execute(
            "UPDATE download_jobs SET status=?1,error=?2,speed_bytes_per_second=0 WHERE id=?3",
            params![state, error, id],
        )
        .map_err(io_error)?;
    transaction
        .execute(
            "UPDATE download_execution SET phase=?1 WHERE job_id=?2",
            params![phase, id],
        )
        .map_err(io_error)?;
    transaction.commit().map_err(io_error)
}

pub fn start(id: String) -> Result<DownloadJob, EmuBoxError> {
    start_requested(id, false)
}

pub fn resume(id: String) -> Result<DownloadJob, EmuBoxError> {
    start_requested(id, true)
}

fn start_requested(id: String, retry_preparation: bool) -> Result<DownloadJob, EmuBoxError> {
    let mut guard = active().lock().map_err(io_error)?;
    let mut job =
        DownloadService::get_job(&id)?.ok_or_else(|| EmuBoxError::NotFound(id.clone()))?;
    if matches!(
        job.status,
        DownloadStatus::Completed | DownloadStatus::Downloaded
    ) {
        if !Path::new(&job.destination_path).exists() {
            return Err(EmuBoxError::NotFound(
                "El contenido descargado ya no existe; crea un trabajo nuevo".into(),
            ));
        }
        if matches!(job.status, DownloadStatus::Completed) || !retry_preparation {
            return Ok(job);
        }
    }
    if matches!(job.status, DownloadStatus::Cancelled) {
        return Err(EmuBoxError::InvalidConfiguration(
            "Trabajo cancelado; selecciona la fuente para crear otro".into(),
        ));
    }
    if let Some(control) = guard.get(&id) {
        if control.paused.load(Ordering::Relaxed) {
            return Err(EmuBoxError::ProcessFailed(
                "Pausa en curso; espera a que el proveedor cierre la transferencia".into(),
            ));
        }
        return Ok(job);
    }
    let execution: Option<(String, Option<String>)> = DatabaseService::get_connection()?
        .query_row(
            "SELECT source_json,artifacts_json FROM download_execution WHERE job_id=?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(io_error)?;
    if execution
        .as_ref()
        .is_some_and(|(_, artifacts)| artifacts.is_some())
        && !Path::new(&job.destination_path).exists()
    {
        return Err(EmuBoxError::StorageUnavailable(
            "El contenido publicado ya no existe; no se vuelve a descargar al reanudar".into(),
        ));
    }
    let source: DownloadSource = match execution {
        Some((json, _)) => serde_json::from_str(&json).map_err(io_error)?,
        None => {
            let source = DownloadService::list_sources(&job.game_id)?
                .into_iter()
                .find(|option| option.source.id == job.source_id)
                .ok_or_else(|| {
                    EmuBoxError::NotFound("Fuente del trabajo antiguo no disponible".into())
                })?
                .source;
            let provider = crate::services::download_resolver::resolve(&source)?;
            DatabaseService::get_connection()?
                .execute(
                    "INSERT INTO download_execution(job_id,source_json,provider) VALUES (?1,?2,?3)",
                    params![
                        id,
                        serde_json::to_string(&source).map_err(io_error)?,
                        provider.as_str()
                    ],
                )
                .map_err(io_error)?;
            job.destination_path = content_root()
                .join(&job.platform)
                .join(&job.id)
                .to_string_lossy()
                .to_string();
            DatabaseService::get_connection()?
                .execute(
                    "UPDATE download_jobs SET destination_path=?1 WHERE id=?2",
                    params![job.destination_path, id],
                )
                .map_err(io_error)?;
            source
        }
    };
    if !guard.is_empty() {
        status(&id, "queued", "queued", None)?;
        return DownloadService::get_job(&id)?.ok_or(EmuBoxError::NotFound(id));
    }
    let local_content = Path::new(&job.destination_path).exists();
    let provider = if local_content {
        None
    } else {
        Some(crate::services::download_resolver::resolve(&source)?)
    };
    let control = TransferControl::default();
    status(
        &id,
        "downloading",
        if local_content {
            "preparing"
        } else {
            "transferring"
        },
        None,
    )?;
    guard.insert(id.clone(), control.clone());
    thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run(&job, &source, provider, &control)
        }))
        .unwrap_or_else(|_| {
            Err(EmuBoxError::ProcessFailed(
                "Proveedor interrumpido inesperadamente".into(),
            ))
        });
        let mut guard = active().lock().unwrap();
        if control.cancelled.load(Ordering::Relaxed) {
            let cleanup = clean_staging(&job).err().map(|error| error.to_string());
            let _ = status(&job.id, "cancelled", "cancelled", cleanup.as_deref());
        } else if control.paused.load(Ordering::Relaxed) {
            let _ = status(&job.id, "paused", "paused", None);
        } else if let Err(error) = result {
            let cleanup = clean_staging(&job).err();
            let failure = match cleanup {
                Some(cleanup_error) => {
                    format!("{error}; no se pudo limpiar el contenido parcial: {cleanup_error}")
                }
                None => error.to_string(),
            };
            let _ = status(&job.id, "failed", "failed", Some(&failure));
        }
        guard.remove(&job.id);
        drop(guard);
        if let Ok(jobs) = DownloadService::list_jobs() {
            for next in jobs
                .into_iter()
                .rev()
                .filter(|job| matches!(job.status, DownloadStatus::Queued))
            {
                if start(next.id.clone()).is_ok() {
                    break;
                }
                let _ = status(
                    &next.id,
                    "failed",
                    "failed",
                    Some("No se pudo iniciar el proveedor de la cola"),
                );
            }
        }
    });
    drop(guard);
    DownloadService::get_job(&id)?.ok_or(EmuBoxError::NotFound(id))
}

pub fn pause(id: &str) -> Result<DownloadJob, EmuBoxError> {
    let guard = active().lock().map_err(io_error)?;
    let job = DownloadService::get_job(id)?.ok_or_else(|| EmuBoxError::NotFound(id.into()))?;
    if matches!(
        job.status,
        DownloadStatus::Completed | DownloadStatus::Downloaded | DownloadStatus::Cancelled
    ) {
        return Ok(job);
    }
    if let Some(control) = guard.get(id) {
        control.paused.store(true, Ordering::Relaxed);
    }
    status(id, "paused", "paused", None)?;
    DownloadService::get_job(id)?.ok_or_else(|| EmuBoxError::NotFound(id.into()))
}

pub fn cancel(id: &str) -> Result<DownloadJob, EmuBoxError> {
    let guard = active().lock().map_err(io_error)?;
    let job = DownloadService::get_job(id)?.ok_or_else(|| EmuBoxError::NotFound(id.into()))?;
    if matches!(
        job.status,
        DownloadStatus::Completed | DownloadStatus::Downloaded
    ) {
        return Ok(job);
    }
    if let Some(control) = guard.get(id) {
        control.cancelled.store(true, Ordering::Relaxed);
    }
    status(id, "cancelled", "cancelled", None)?;
    if !guard.contains_key(id) {
        clean_staging(&job)?;
    }
    DownloadService::get_job(id)?.ok_or_else(|| EmuBoxError::NotFound(id.into()))
}
