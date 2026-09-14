use super::{active, clean_staging};
use crate::{
    errors::EmuBoxError,
    models::{DownloadJob, DownloadSource, PublishedDownload, TransferControl},
    services::{db_service::DatabaseService, download_providers::io_error},
};
use crate::{models::DownloadStatus, services::download_service::DownloadService};
use rusqlite::params;
use sha2::{Digest, Sha256};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

fn validated_package(job: &DownloadJob) -> Result<PublishedDownload, EmuBoxError> {
    let destination = Path::new(&job.destination_path);
    let package: PublishedDownload =
        serde_json::from_slice(&fs::read(destination.join(".emubox-managed")).map_err(io_error)?)
            .map_err(io_error)?;
    let json: String = DatabaseService::get_connection()?
        .query_row(
            "SELECT source_json FROM download_execution WHERE job_id=?1",
            params![job.id],
            |row| row.get(0),
        )
        .map_err(io_error)?;
    let source: DownloadSource = serde_json::from_str(&json).map_err(io_error)?;
    let digest = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&source).map_err(io_error)?)
    );
    if package.job_id != job.id || package.source_digest != digest {
        return Err(EmuBoxError::StorageUnavailable(
            "El paquete no pertenece a este trabajo".into(),
        ));
    }
    for path in &package.files {
        if path.as_os_str().is_empty()
            || path.is_absolute()
            || path
                .components()
                .any(|component| !matches!(component, std::path::Component::Normal(_)))
        {
            return Err(EmuBoxError::StorageUnavailable(
                "Ruta de paquete invalida".into(),
            ));
        }
    }
    let files: Vec<_> = package
        .files
        .iter()
        .map(|path| destination.join(path))
        .collect();
    crate::services::download_preparation::verify(
        &files,
        destination,
        None,
        &TransferControl::default(),
    )?;
    if files.iter().any(|path| !path.is_file()) {
        return Err(EmuBoxError::StorageUnavailable("Paquete incompleto".into()));
    }
    Ok(package)
}

fn candidate_paths(job: &DownloadJob, package: &PublishedDownload) -> Vec<String> {
    if package.installation.is_some() {
        return crate::services::installer_preparation::candidates(
            &job.platform,
            Path::new(&job.destination_path),
            package,
        );
    }
    package
        .files
        .iter()
        .filter(|path| {
            crate::services::download_preparation::launch_target(&job.platform, &[(*path).clone()])
                .is_some()
        })
        .map(|path| path.to_string_lossy().into_owned())
        .collect()
}

pub fn candidates(id: &str) -> Result<Vec<String>, EmuBoxError> {
    let guard = active().lock().map_err(io_error)?;
    let job = DownloadService::get_job(id)?.ok_or_else(|| EmuBoxError::NotFound(id.into()))?;
    if guard.contains_key(id)
        || !matches!(
            job.status,
            DownloadStatus::Downloaded | DownloadStatus::Completed
        )
    {
        return Err(EmuBoxError::ProcessFailed(
            "El paquete no esta disponible para seleccion local".into(),
        ));
    }
    Ok(candidate_paths(&job, &validated_package(&job)?))
}

fn write_package(destination: &Path, package: &PublishedDownload) -> Result<(), EmuBoxError> {
    let attempt = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(io_error)?
        .as_nanos();
    let marker = destination.join(format!(".emubox-managed-{attempt}.next"));
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&marker)
        .map_err(io_error)?;
    use std::io::Write;
    output
        .write_all(&serde_json::to_vec(package).map_err(io_error)?)
        .map_err(io_error)?;
    output.sync_all().map_err(io_error)?;
    fs::rename(marker, destination.join(".emubox-managed")).map_err(io_error)?;
    fs::File::open(destination)
        .and_then(|file| file.sync_all())
        .map_err(io_error)
}

pub fn select_candidate(id: &str, path: &str) -> Result<DownloadJob, EmuBoxError> {
    let guard = active().lock().map_err(io_error)?;
    let job = DownloadService::get_job(id)?.ok_or_else(|| EmuBoxError::NotFound(id.into()))?;
    if guard.contains_key(id)
        || !matches!(
            job.status,
            DownloadStatus::Downloaded | DownloadStatus::Completed
        )
    {
        return Err(EmuBoxError::ProcessFailed(
            "Espera a que termine la preparacion local".into(),
        ));
    }
    let mut package = validated_package(&job)?;
    if !candidate_paths(&job, &package)
        .iter()
        .any(|candidate| candidate == path)
    {
        return Err(EmuBoxError::InvalidConfiguration(
            "El archivo no es un candidato del paquete".into(),
        ));
    }
    package.launch = Some(path.into());
    package.preparation_reason = None;
    write_package(Path::new(&job.destination_path), &package)?;
    finish(&job, &package)?;
    DownloadService::get_job(id)?.ok_or_else(|| EmuBoxError::NotFound(id.into()))
}

pub(super) fn prepare_published(
    job: &DownloadJob,
    source: &DownloadSource,
    destination: &Path,
    package: &mut PublishedDownload,
    files: &[PathBuf],
    control: &TransferControl,
) -> Result<(), EmuBoxError> {
    if package.installation.is_some() || candidate_paths(job, package).len() > 1 {
        let _guard = active().lock().map_err(io_error)?;
        if !control.interrupted() {
            package.preparation_reason =
                Some("Selecciona el archivo de lanzamiento del paquete".into());
            finish(job, package)?;
        }
        return Ok(());
    }
    let root = destination
        .parent()
        .ok_or_else(|| EmuBoxError::InvalidConfiguration("Destino invalido".into()))?
        .join(".emubox-staging")
        .join(&job.id);
    fs::create_dir_all(&root).map_err(io_error)?;
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).map_err(io_error)?;
    let prepared = crate::services::download_preparation::verify(
        files,
        destination,
        source.checksum.as_deref(),
        control,
    )
    .and_then(|()| crate::services::download_preparation::prepare(files, &root, control));
    let prepared = match prepared {
        Ok(files) => files,
        Err(_) if control.interrupted() => return Ok(()),
        Err(error) => {
            let _guard = active().lock().map_err(io_error)?;
            if !control.interrupted() {
                package.preparation_reason = Some(format!(
                    "Originales conservados; preparacion no completada: {error}"
                ));
                finish(job, package)?;
                clean_staging(job)?;
            }
            return Ok(());
        }
    };
    let mut installation =
        crate::services::installer_preparation::prepared_metadata(files, &prepared, &root);
    let target = crate::services::download_preparation::launch_target(&job.platform, &prepared);
    let _guard = active().lock().map_err(io_error)?;
    if control.interrupted() {
        return Ok(());
    }
    let has_candidates = prepared.iter().any(|path| {
        crate::services::download_preparation::launch_target(&job.platform, &[path.clone()])
            .is_some()
    });
    if has_candidates || installation.is_some() {
        if prepared.iter().all(|path| path.starts_with(&root)) {
            let attempt = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(io_error)?
                .as_nanos();
            let published_root = destination.join(format!("prepared-{attempt}"));
            fs::create_dir(&published_root).map_err(io_error)?;
            fs::rename(root.join("extracted"), published_root.join("content")).map_err(io_error)?;
            if let Some(metadata) = installation.as_mut() {
                metadata.root = published_root
                    .join("content")
                    .strip_prefix(destination)
                    .map_err(io_error)?
                    .to_path_buf();
            }
            for path in &prepared {
                let relative = published_root
                    .join("content")
                    .join(
                        path.strip_prefix(root.join("extracted"))
                            .map_err(io_error)?,
                    )
                    .strip_prefix(destination)
                    .map_err(io_error)?
                    .to_path_buf();
                if Some(path) == target.as_ref() {
                    package.launch = Some(relative.clone());
                }
                package.files.push(relative);
            }
        } else if let Some(target) = target {
            package.launch = Some(
                target
                    .strip_prefix(destination)
                    .map_err(io_error)?
                    .to_path_buf(),
            );
        }
        package.installation = installation;
        package.preparation_reason = if package.launch.is_some() {
            None
        } else {
            Some("Selecciona el archivo de lanzamiento del paquete".into())
        };
        write_package(destination, package)?;
    } else {
        package.preparation_reason = Some("Originales conservados; no hay un candidato de lanzamiento unico. Requiere seleccion o instalacion especifica".into());
    }
    finish(job, package)?;
    clean_staging(job)
}

pub(super) fn finish(job: &DownloadJob, package: &PublishedDownload) -> Result<(), EmuBoxError> {
    let destination = PathBuf::from(&job.destination_path);
    let mut connection = DatabaseService::get_connection()?;
    let transaction = connection.transaction().map_err(io_error)?;
    if let Some(filename) = &package.launch {
        if !package.files.contains(filename) {
            return Err(EmuBoxError::InvalidConfiguration(
                "Destino de lanzamiento no pertenece al paquete".into(),
            ));
        }
        let path = destination.join(filename);
        transaction
            .execute(
                "UPDATE games SET rom_path=?1,file_size_bytes=?2 WHERE id=?3",
                params![
                    path.to_string_lossy(),
                    fs::metadata(&path).map_err(io_error)?.len(),
                    job.game_id
                ],
            )
            .map_err(io_error)?;
    }
    transaction.execute("UPDATE download_jobs SET status=?1,progress=1,speed_bytes_per_second=0,error=?2 WHERE id=?3",params![if package.launch.is_some() {"completed"} else {"downloaded"},if package.launch.is_some() {None} else {Some(package.preparation_reason.as_deref().unwrap_or("Contenido descargado; requiere preparacion o seleccion de archivo"))},job.id]).map_err(io_error)?;
    transaction
        .execute(
            "UPDATE download_execution SET phase=?1,artifacts_json=?2 WHERE job_id=?3",
            params![
                if package.launch.is_some() {
                    "ready"
                } else {
                    "preparation_required"
                },
                serde_json::to_string(&package.files).map_err(io_error)?,
                job.id
            ],
        )
        .map_err(io_error)?;
    transaction.commit().map_err(io_error)?;
    Ok(())
}
