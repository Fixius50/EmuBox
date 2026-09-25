use super::{
    active,
    publication::{finish, prepare_published},
};
use crate::models::{ProviderId, TransferOutcome, TransferRequest};
use crate::{
    errors::EmuBoxError,
    models::{DownloadJob, DownloadSource, PublishedDownload, TransferControl},
    services::{db_service::DatabaseService, download_providers::io_error},
};
use rusqlite::params;
use sha2::{Digest, Sha256};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf, time::{Duration, Instant}};

pub(super) fn run(
    job: &DownloadJob,
    source: &DownloadSource,
    provider: Option<ProviderId>,
    control: &TransferControl,
) -> Result<(), EmuBoxError> {
    if !job
        .id
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        return Err(EmuBoxError::InvalidConfiguration(
            "Identificador de trabajo invalido".into(),
        ));
    }
    let destination = PathBuf::from(&job.destination_path);
    let digest = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(source).map_err(io_error)?)
    );
    if destination.exists() {
        let mut package: PublishedDownload = serde_json::from_slice(
            &fs::read(destination.join(".emubox-managed")).map_err(io_error)?,
        )
        .map_err(io_error)?;
        if package.job_id != job.id || package.source_digest != digest {
            return Err(EmuBoxError::StorageUnavailable(
                "El destino pertenece a otro contenido".into(),
            ));
        }
        for path in &package.files {
            if path.is_absolute()
                || path
                    .components()
                    .any(|component| !matches!(component, std::path::Component::Normal(_)))
                || !destination.join(path).is_file()
            {
                return Err(EmuBoxError::StorageUnavailable(
                    "Publicacion incompleta o ruta invalida".into(),
                ));
            }
        }
        let files: Vec<_> = package
            .files
            .iter()
            .map(|path| destination.join(path))
            .collect();
        crate::services::download_preparation::verify(&files, &destination, None, control)?;
        if package.launch.is_none() {
            return prepare_published(job, source, &destination, &mut package, &files, control);
        }
        let _guard = active().lock().map_err(io_error)?;
        if !control.interrupted() {
            finish(job, &package)?;
        }
        return Ok(());
    }
    let provider = provider.ok_or_else(|| {
        EmuBoxError::StorageUnavailable(
            "El contenido local desaparecio; no se inicia una descarga implicitamente".into(),
        )
    })?;
    let root = destination
        .parent()
        .ok_or_else(|| EmuBoxError::InvalidConfiguration("Destino sin plataforma".into()))?
        .join(".emubox-staging")
        .join(&job.id);
    fs::create_dir_all(&root).map_err(io_error)?;
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).map_err(io_error)?;
    let filename = reqwest::Url::parse(&source.uri)
        .ok()
        .and_then(|url| {
            url.path_segments()
                .and_then(|mut segments| segments.rfind(|segment| !segment.is_empty()))
                .map(str::to_string)
        })
        .unwrap_or_else(|| "content.bin".into());
    let resolved_uri = crate::services::download_connectors::resolve_uri(&source.uri)?;
    let seed_completed_torrents = provider == ProviderId::BitTorrent
        && crate::services::config_service::get_settings()
            .ok()
            .and_then(|settings| settings.system)
            .and_then(|system| {
                system
                    .get("seedCompletedTorrents")
                    .and_then(|value| value.as_bool())
            })
            .unwrap_or(false);
    let progress_connection = DatabaseService::get_connection()?;
            let mut last_sample = Instant::now();
            let mut last_phase = String::new();
            crate::services::infrastructure::telemetry::event(log::Level::Info, "download", "download.transfer", "Transferencia iniciada", serde_json::json!({"provider": provider.as_str(), "phase": "starting"}));
    let transfer = crate::services::download_providers::provider(provider, seed_completed_torrents)
        .transfer(
            &TransferRequest {
                uri: &resolved_uri,
                directory: &root,
                filename: &filename,
                control,
                max_bytes: None,
            },
            &mut |progress| {
                let ratio = if progress.phase == "seeding" {
                    1.0
                } else {
                    progress
                        .total
                        .map(|total| {
                            (progress.downloaded as f64 / total.max(1) as f64).min(0.99)
                        })
                        .unwrap_or(0.0)
                };
                if last_phase != progress.phase || last_sample.elapsed() >= Duration::from_secs(5) {
                    crate::services::infrastructure::telemetry::event(log::Level::Info, "download", "download.transfer", "Progreso de transferencia", serde_json::json!({"provider": provider.as_str(), "phase": progress.phase, "percent": (ratio * 100.0).round() as u8}));
                    last_phase = progress.phase.to_string();
                    last_sample = Instant::now();
                }
                progress_connection
                    .prepare_cached("UPDATE download_execution SET phase=?1 WHERE job_id=?2")
                    .map_err(io_error)?
                    .execute(params![progress.phase, job.id])
                    .map_err(io_error)?;
                progress_connection
                    .prepare_cached("UPDATE download_jobs SET progress=?1,downloaded_bytes=?2,total_bytes=?3,speed_bytes_per_second=?4 WHERE id=?5 AND status='downloading'")
                    .map_err(io_error)?
                    .execute(params![ratio,progress.downloaded,progress.total,progress.speed,job.id])
                    .map_err(io_error)?;
                Ok(())
            },
        );
    crate::services::infrastructure::telemetry::event(log::Level::Info, "download", "download.transfer", "Transferencia terminada", serde_json::json!({"provider": provider.as_str(), "phase": if transfer.is_err() { "error" } else if control.interrupted() { "interrupted" } else { "finished" }}));
    let transfer = transfer?;
    drop(progress_connection);
    let TransferOutcome::Complete(files) = transfer else {
        return Ok(());
    };
    DatabaseService::get_connection()?
        .execute(
            "UPDATE download_execution SET phase='verifying' WHERE job_id=?1",
            params![job.id],
        )
        .map_err(io_error)?;
    crate::services::infrastructure::telemetry::event(log::Level::Info, "download", "download.transfer", "Verificando descarga", serde_json::json!({"provider": provider.as_str(), "phase": "verifying"}));
    crate::services::download_preparation::verify(
        &files,
        &root,
        source.checksum.as_deref(),
        control,
    )?;
    if control.interrupted() {
        return Ok(());
    }
    DatabaseService::get_connection()?
        .execute(
            "UPDATE download_execution SET phase='preparing' WHERE job_id=?1",
            params![job.id],
        )
        .map_err(io_error)?;
    crate::services::infrastructure::telemetry::event(log::Level::Info, "download", "download.transfer", "Preparando descarga", serde_json::json!({"provider": provider.as_str(), "phase": "preparing"}));
    let mut installation = None;
    let (files, preparation_reason) =
        match crate::services::download_preparation::prepare(&files, &root, control) {
            Ok(prepared) => {
                installation = crate::services::installer_preparation::prepared_metadata(
                    &files, &prepared, &root,
                );
                let mut prepared = prepared;
                if installation.is_some() {
                    prepared.extend(files.iter().cloned());
                }
                (prepared, None)
            }
            Err(_) if control.interrupted() => return Ok(()),
            Err(error) => (
                files,
                Some(format!(
                    "Contenido obtenido; preparacion no completada: {error}"
                )),
            ),
        };
    let target = crate::services::download_preparation::launch_target(&job.platform, &files);
    let prepared = root.join("publish");
    if prepared.exists() {
        fs::remove_dir_all(&prepared).map_err(io_error)?;
    }
    fs::create_dir(&prepared).map_err(io_error)?;
    let mut published = Vec::new();
    let mut launch = None;
    for file in &files {
        if control.interrupted() {
            return Ok(());
        }
        let relative = file.strip_prefix(&root).map_err(io_error)?.to_path_buf();
        let published_file = prepared.join(&relative);
        fs::create_dir_all(published_file.parent().unwrap()).map_err(io_error)?;
        fs::copy(file, &published_file).map_err(io_error)?;
        fs::File::open(&published_file)
            .and_then(|file| file.sync_all())
            .map_err(io_error)?;
        if target.as_ref() == Some(file) {
            launch = Some(relative.clone());
        }
        published.push(relative);
    }
    let package = PublishedDownload {
        job_id: job.id.clone(),
        source_digest: digest,
        files: published,
        launch,
        preparation_reason,
        installation,
    };
    fs::write(
        prepared.join(".emubox-managed"),
        serde_json::to_vec(&package).map_err(io_error)?,
    )
    .map_err(io_error)?;
    let _guard = active().lock().map_err(io_error)?;
    if control.interrupted() {
        return Ok(());
    }
    if destination.exists() {
        return Err(EmuBoxError::StorageUnavailable(
            "Destino ya existe; no se sobrescribe contenido".into(),
        ));
    }
    fs::create_dir_all(
        destination
            .parent()
            .ok_or_else(|| EmuBoxError::InvalidConfiguration("Destino invalido".into()))?,
    )
    .map_err(io_error)?;
    fs::rename(&prepared, &destination).map_err(io_error)?;
    finish(job, &package)?;
    let _ = fs::remove_dir_all(&root);
    Ok(())
}
