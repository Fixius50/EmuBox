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
    collections::{HashMap, HashSet},
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

pub fn delete_cancelled(id: &str) -> Result<(), EmuBoxError> {
    let guard = active().lock().map_err(io_error)?;
    let job = DownloadService::get_job(id)?.ok_or_else(|| EmuBoxError::NotFound(id.into()))?;
    let valid = |value: &str| !value.is_empty() && value.chars().all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'));
    if !matches!(job.status, DownloadStatus::Cancelled) || guard.contains_key(id)
        || !valid(&job.id) || !valid(&job.platform)
        || Path::new(&job.destination_path) != content_root().join(&job.platform).join(&job.id)
        || fs::symlink_metadata(&job.destination_path).is_ok()
    {
        return Err(EmuBoxError::InvalidConfiguration("Solo se puede borrar un trabajo cancelado sin contenido publicado".into()));
    }
    let platform = content_root().join(&job.platform);
    for directory in [&platform, &platform.join(".emubox-staging")] {
        if fs::symlink_metadata(directory).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
            return Err(EmuBoxError::InvalidConfiguration("Ruta de descarga enlazada fuera del almacenamiento gestionado".into()));
        }
    }
    clean_staging(&job)?;
    let mut connection = DatabaseService::get_connection()?;
    let transaction = connection.transaction().map_err(io_error)?;
    transaction.execute("DELETE FROM download_execution WHERE job_id=?1", [id]).map_err(io_error)?;
    transaction.execute("DELETE FROM download_jobs WHERE id=?1 AND status='cancelled'", [id]).map_err(io_error)?;
    transaction.commit().map_err(io_error)
}

pub fn uninstall(game_id: &str) -> Result<(), EmuBoxError> {
    let guard = active().lock().map_err(io_error)?;
    let game = crate::services::GameService::get_game_by_id(game_id.to_string())?
        .ok_or_else(|| EmuBoxError::NotFound("Juego no encontrado".into()))?;
    let rom = Path::new(game.rom_path.as_deref().ok_or_else(|| EmuBoxError::InvalidConfiguration("El juego no esta instalado".into()))?);
    if crate::services::ProcessService::get_running_game()?.is_some_and(|running| running.game_id == game_id) {
        return Err(EmuBoxError::ProcessFailed("Cierra el juego antes de desinstalarlo".into()));
    }
    let root = content_root().join(&game.platform);
    if fs::symlink_metadata(&root).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(EmuBoxError::InvalidConfiguration("Ruta de instalacion enlazada fuera del almacenamiento gestionado".into()));
    }
    let job_id = rom.strip_prefix(&root).ok()
        .and_then(|relative| relative.components().next())
        .filter(|component| matches!(component, std::path::Component::Normal(_)))
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .ok_or_else(|| EmuBoxError::InvalidConfiguration("Instalacion no gestionada por EmuBox".into()))?;
    let job = DownloadService::get_job(&job_id)?.ok_or_else(|| EmuBoxError::InvalidConfiguration("Falta el trabajo propietario de la instalacion".into()))?;
    let destination = root.join(&job_id);
    if job.game_id != game_id || !matches!(job.status, DownloadStatus::Completed)
        || job.destination_path != destination.to_string_lossy()
        || guard.contains_key(&job_id)
        || !job_id.chars().all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        return Err(EmuBoxError::InvalidConfiguration("Instalacion no gestionada o en uso".into()));
    }
    let metadata = fs::symlink_metadata(&destination).map_err(io_error)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(EmuBoxError::InvalidConfiguration("Directorio gestionado invalido".into()));
    }
    let package = publication::validated_package(&job)?;
    if package.launch.as_ref().map(|path| destination.join(path)) != Some(rom.to_path_buf()) {
        return Err(EmuBoxError::InvalidConfiguration("El juego no corresponde al paquete gestionado".into()));
    }
    let mut allowed = HashSet::from([destination.join(".emubox-managed")]);
    for file in &package.files {
        let mut path = destination.join(file);
        while path != destination {
            allowed.insert(path.clone());
            path = path.parent().ok_or_else(|| EmuBoxError::InvalidConfiguration("Ruta de paquete invalida".into()))?.to_path_buf();
        }
    }
    for entry in walkdir::WalkDir::new(&destination).follow_links(false) {
        let entry = entry.map_err(io_error)?;
        if entry.path() != destination && (!allowed.contains(entry.path()) || entry.file_type().is_symlink()) {
            return Err(EmuBoxError::InvalidConfiguration("El paquete contiene archivos ajenos; no se borra".into()));
        }
    }
    let pending = root.join(format!(".uninstall-{job_id}"));
    if pending.exists() || fs::symlink_metadata(&pending).is_ok() {
        return Err(EmuBoxError::InvalidConfiguration("Desinstalacion pendiente existente".into()));
    }
    let mut connection = DatabaseService::get_connection()?;
    fs::rename(&destination, &pending).map_err(io_error)?;
    let result = (|| -> Result<(), EmuBoxError> {
        let transaction = connection.transaction().map_err(io_error)?;
        let changed = transaction.execute("UPDATE games SET rom_path=NULL,file_size_bytes=0 WHERE id=?1 AND rom_path=?2", params![game_id, rom.to_string_lossy()]).map_err(io_error)?;
        if changed != 1 { return Err(EmuBoxError::StorageUnavailable("La instalacion cambio durante la desinstalacion".into())); }
        transaction.execute("DELETE FROM download_execution WHERE job_id=?1", [&job_id]).map_err(io_error)?;
        transaction.execute("DELETE FROM download_jobs WHERE id=?1", [&job_id]).map_err(io_error)?;
        transaction.commit().map_err(io_error)
    })();
    if result.is_err() {
        fs::rename(&pending, &destination).map_err(io_error)?;
        return result;
    }
    fs::remove_dir_all(&pending).map_err(io_error)
}
