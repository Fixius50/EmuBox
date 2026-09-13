use crate::{errors::EmuBoxError, models::{DownloadJob, DownloadSource, DownloadStatus, ProviderId, PublishedDownload, TransferControl, TransferOutcome, TransferRequest}};
use sha2::{Digest, Sha256};
use super::{db_service::DatabaseService, download_service::DownloadService, download_providers::io_error};
use rusqlite::{params, OptionalExtension};
use std::{collections::HashMap, fs, os::unix::fs::PermissionsExt, path::{Path, PathBuf}, sync::{Mutex, OnceLock, atomic::Ordering}, thread};

static ACTIVE: OnceLock<Mutex<HashMap<String, TransferControl>>> = OnceLock::new();
fn active() -> &'static Mutex<HashMap<String, TransferControl>> { ACTIVE.get_or_init(|| Mutex::new(HashMap::new())) }

pub fn content_root() -> PathBuf {
    #[cfg(test)]
    return std::env::temp_dir().join(format!("emubox-downloads-tests-{}",std::process::id()));
    #[cfg(not(test))]
    PathBuf::from(super::paths::games_dir())
}

pub fn recover() -> Result<(), EmuBoxError> {
    let _guard = active().lock().map_err(io_error)?;
    DatabaseService::get_connection()?.execute("UPDATE download_jobs SET status='paused', speed_bytes_per_second=0, error='Trabajo interrumpido; reanudar explicitamente' WHERE status IN ('downloading','queued')", []).map_err(io_error)?;
    Ok(())
}

fn status(id: &str, state: &str, phase: &str, error: Option<&str>) -> Result<(), EmuBoxError> {
    let mut connection = DatabaseService::get_connection()?;
    let transaction = connection.transaction().map_err(io_error)?;
    transaction.execute("UPDATE download_jobs SET status=?1,error=?2,speed_bytes_per_second=0 WHERE id=?3", params![state,error,id]).map_err(io_error)?;
    transaction.execute("UPDATE download_execution SET phase=?1 WHERE job_id=?2", params![phase,id]).map_err(io_error)?;
    transaction.commit().map_err(io_error)
}

pub fn start(id: String) -> Result<DownloadJob, EmuBoxError> {
    let mut guard = active().lock().map_err(io_error)?;
    let mut job = DownloadService::get_job(&id)?.ok_or_else(|| EmuBoxError::NotFound(id.clone()))?;
    if matches!(job.status, DownloadStatus::Completed | DownloadStatus::Downloaded) {
        if !Path::new(&job.destination_path).exists() { return Err(EmuBoxError::NotFound("El contenido descargado ya no existe; crea un trabajo nuevo".into())); }
        return Ok(job);
    }
    if matches!(job.status, DownloadStatus::Cancelled) { return Err(EmuBoxError::InvalidConfiguration("Trabajo cancelado; selecciona la fuente para crear otro".into())); }
    if let Some(control) = guard.get(&id) {
        if control.paused.load(Ordering::Relaxed) { return Err(EmuBoxError::ProcessFailed("Pausa en curso; espera a que el proveedor cierre la transferencia".into())); }
        return Ok(job);
    }
    if !guard.is_empty() { status(&id,"queued","queued",None)?; return DownloadService::get_job(&id)?.ok_or(EmuBoxError::NotFound(id)); }
    let source_json: Option<String> = DatabaseService::get_connection()?.query_row("SELECT source_json FROM download_execution WHERE job_id=?1", params![id], |row| row.get(0)).optional().map_err(io_error)?;
    let source: DownloadSource = match source_json {
        Some(json) => serde_json::from_str(&json).map_err(io_error)?,
        None => {
            let source = DownloadService::list_sources(&job.game_id)?.into_iter().find(|option| option.source.id == job.source_id)
                .ok_or_else(|| EmuBoxError::NotFound("Fuente del trabajo antiguo no disponible".into()))?.source;
            let provider = super::download_resolver::resolve(&source)?;
            DatabaseService::get_connection()?.execute("INSERT INTO download_execution(job_id,source_json,provider) VALUES (?1,?2,?3)", params![id,serde_json::to_string(&source).map_err(io_error)?,provider.as_str()]).map_err(io_error)?;
            job.destination_path = content_root().join(&job.platform).join(&job.id).to_string_lossy().to_string();
            DatabaseService::get_connection()?.execute("UPDATE download_jobs SET destination_path=?1 WHERE id=?2",params![job.destination_path,id]).map_err(io_error)?;
            source
        }
    };
    let provider = super::download_resolver::resolve(&source)?;
    let control = TransferControl::default();
    status(&id,"downloading","transferring",None)?;
    guard.insert(id.clone(),control.clone());
    thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(&job,&source,provider,&control)))
            .unwrap_or_else(|_| Err(EmuBoxError::ProcessFailed("Proveedor interrumpido inesperadamente".into())));
        let mut guard = active().lock().unwrap();
        if control.cancelled.load(Ordering::Relaxed) { let _ = status(&job.id,"cancelled","cancelled",None); }
        else if control.paused.load(Ordering::Relaxed) { let _ = status(&job.id,"paused","paused",None); }
        else if let Err(error) = result { let _ = status(&job.id,"failed","failed",Some(&error.to_string())); }
        guard.remove(&job.id); drop(guard);
        if let Ok(jobs) = DownloadService::list_jobs() {
            for next in jobs.into_iter().rev().filter(|job| matches!(job.status, DownloadStatus::Queued)) {
                if start(next.id.clone()).is_ok() { break; }
                let _ = status(&next.id,"failed","failed",Some("No se pudo iniciar el proveedor de la cola"));
            }
        }
    });
    drop(guard);
    DownloadService::get_job(&id)?.ok_or(EmuBoxError::NotFound(id))
}

pub fn pause(id: &str) -> Result<DownloadJob, EmuBoxError> {
    let guard = active().lock().map_err(io_error)?;
    let job = DownloadService::get_job(id)?.ok_or_else(|| EmuBoxError::NotFound(id.into()))?;
    if matches!(job.status, DownloadStatus::Completed | DownloadStatus::Downloaded | DownloadStatus::Cancelled) { return Ok(job); }
    if let Some(control) = guard.get(id) { control.paused.store(true,Ordering::Relaxed); }
    status(id,"paused","paused",None)?;
    DownloadService::get_job(id)?.ok_or_else(|| EmuBoxError::NotFound(id.into()))
}

pub fn cancel(id: &str) -> Result<DownloadJob, EmuBoxError> {
    let guard = active().lock().map_err(io_error)?;
    let job = DownloadService::get_job(id)?.ok_or_else(|| EmuBoxError::NotFound(id.into()))?;
    if matches!(job.status, DownloadStatus::Completed | DownloadStatus::Downloaded) { return Ok(job); }
    if let Some(control) = guard.get(id) { control.cancelled.store(true,Ordering::Relaxed); }
    status(id,"cancelled","cancelled",None)?;
    DownloadService::get_job(id)?.ok_or_else(|| EmuBoxError::NotFound(id.into()))
}

fn run(job: &DownloadJob, source: &DownloadSource, provider: ProviderId, control: &TransferControl) -> Result<(), EmuBoxError> {
    if !job.id.chars().all(|character| character.is_ascii_alphanumeric() || matches!(character,'-'|'_')) { return Err(EmuBoxError::InvalidConfiguration("Identificador de trabajo invalido".into())); }
    let destination = PathBuf::from(&job.destination_path);
    let digest = format!("{:x}",Sha256::digest(serde_json::to_vec(source).map_err(io_error)?));
    if destination.exists() {
        let package: PublishedDownload = serde_json::from_slice(&fs::read(destination.join(".emubox-managed")).map_err(io_error)?).map_err(io_error)?;
        if package.job_id != job.id || package.source_digest != digest { return Err(EmuBoxError::StorageUnavailable("El destino pertenece a otro contenido".into())); }
        for path in &package.files {
            if path.is_absolute() || path.components().any(|component| !matches!(component,std::path::Component::Normal(_))) || !destination.join(path).is_file() { return Err(EmuBoxError::StorageUnavailable("Publicacion incompleta o ruta invalida".into())); }
        }
        let _guard = active().lock().map_err(io_error)?;
        if !control.interrupted() { finish(job,&package)?; }
        return Ok(());
    }
    let root = destination.parent().ok_or_else(|| EmuBoxError::InvalidConfiguration("Destino sin plataforma".into()))?.join(".emubox-staging").join(&job.id);
    fs::create_dir_all(&root).map_err(io_error)?;
    fs::set_permissions(&root,fs::Permissions::from_mode(0o700)).map_err(io_error)?;
    let filename = reqwest::Url::parse(&source.uri).ok().and_then(|url| url.path_segments().and_then(|mut segments| segments.rfind(|segment| !segment.is_empty())).map(str::to_string)).unwrap_or_else(|| "content.bin".into());
    let resolved_uri = super::download_connectors::resolve_uri(&source.uri)?;
    let transfer = super::download_providers::provider(provider).transfer(&TransferRequest { uri:&resolved_uri,directory:&root,filename:&filename,control,max_bytes:None }, &mut |progress| {
        let ratio = progress.total.map(|total| (progress.downloaded as f64 / total.max(1) as f64).min(0.99)).unwrap_or(0.0);
        DatabaseService::get_connection()?.execute("UPDATE download_jobs SET progress=?1,downloaded_bytes=?2,total_bytes=?3,speed_bytes_per_second=?4 WHERE id=?5 AND status='downloading'",params![ratio,progress.downloaded,progress.total,progress.speed,job.id]).map_err(io_error)?;
        Ok(())
    })?;
    let TransferOutcome::Complete(files) = transfer else { return Ok(()); };
    DatabaseService::get_connection()?.execute("UPDATE download_execution SET phase='verifying' WHERE job_id=?1",params![job.id]).map_err(io_error)?;
    super::download_preparation::verify(&files,&root,source.checksum.as_deref(),control)?;
    if control.interrupted() { return Ok(()); }
    DatabaseService::get_connection()?.execute("UPDATE download_execution SET phase='preparing' WHERE job_id=?1",params![job.id]).map_err(io_error)?;
    let (files, preparation_reason) = match super::download_preparation::prepare(&files,&root,control) {
        Ok(prepared) => (prepared,None),
        Err(_) if control.interrupted() => return Ok(()),
        Err(error) => (files,Some(format!("Contenido obtenido; preparacion no completada: {error}"))),
    };
    let target = super::download_preparation::launch_target(&job.platform,&files);
    let prepared = root.join("publish");
    if prepared.exists() { fs::remove_dir_all(&prepared).map_err(io_error)?; }
    fs::create_dir(&prepared).map_err(io_error)?;
    let mut published = Vec::new(); let mut launch = None;
    for file in &files {
        if control.interrupted() { return Ok(()); }
        let relative = file.strip_prefix(&root).map_err(io_error)?.to_path_buf();
        let published_file = prepared.join(&relative);
        fs::create_dir_all(published_file.parent().unwrap()).map_err(io_error)?;
        fs::copy(file,&published_file).map_err(io_error)?;
        fs::File::open(&published_file).and_then(|file| file.sync_all()).map_err(io_error)?;
        if target.as_ref() == Some(file) { launch = Some(relative.clone()); }
        published.push(relative);
    }
    let package = PublishedDownload { job_id:job.id.clone(),source_digest:digest,files:published,launch,preparation_reason };
    fs::write(prepared.join(".emubox-managed"),serde_json::to_vec(&package).map_err(io_error)?).map_err(io_error)?;
    let _guard = active().lock().map_err(io_error)?;
    if control.interrupted() { return Ok(()); }
    if destination.exists() { return Err(EmuBoxError::StorageUnavailable("Destino ya existe; no se sobrescribe contenido".into())); }
    fs::create_dir_all(destination.parent().ok_or_else(|| EmuBoxError::InvalidConfiguration("Destino invalido".into()))?).map_err(io_error)?;
    fs::rename(&prepared,&destination).map_err(io_error)?;
    finish(job,&package)?;
    let _ = fs::remove_dir_all(&root);
    Ok(())
}

fn finish(job: &DownloadJob, package: &PublishedDownload) -> Result<(), EmuBoxError> {
    let destination = PathBuf::from(&job.destination_path);
    let mut connection = DatabaseService::get_connection()?;
    let transaction = connection.transaction().map_err(io_error)?;
    if let Some(filename) = &package.launch {
        if !package.files.contains(filename) { return Err(EmuBoxError::InvalidConfiguration("Destino de lanzamiento no pertenece al paquete".into())); }
        let path = destination.join(filename);
        transaction.execute("UPDATE games SET rom_path=?1,file_size_bytes=?2 WHERE id=?3",params![path.to_string_lossy(),fs::metadata(&path).map_err(io_error)?.len(),job.game_id]).map_err(io_error)?;
    }
    transaction.execute("UPDATE download_jobs SET status=?1,progress=1,speed_bytes_per_second=0,error=?2 WHERE id=?3",params![if package.launch.is_some() {"completed"} else {"downloaded"},if package.launch.is_some() {None} else {Some(package.preparation_reason.as_deref().unwrap_or("Contenido descargado; requiere preparacion o seleccion de archivo"))},job.id]).map_err(io_error)?;
    transaction.execute("UPDATE download_execution SET phase=?1,artifacts_json=?2 WHERE job_id=?3",params![if package.launch.is_some() {"ready"} else {"preparation_required"},serde_json::to_string(&package.files).map_err(io_error)?,job.id]).map_err(io_error)?;
    transaction.commit().map_err(io_error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::{Read,Write},net::TcpListener,time::{Instant,Duration}};

    #[test]
    fn http_jobs_keep_source_snapshot_and_verify_before_publication() {
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let uri = format!("http://{}/content.dat",server.local_addr().unwrap());
        let worker = thread::spawn(move || {
            for _ in 0..2 {
                let (mut stream,_) = server.accept().unwrap();
                stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                let mut buffer=[0u8;4096]; stream.read(&mut buffer).unwrap();
                stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\nETag: \"v1\"\r\nConnection: close\r\n\r\ndata").unwrap();
            }
        });
        for (index,checksum) in [format!("{:x}",Sha256::digest(b"data")),"0".repeat(64)].into_iter().enumerate() {
            let game_id=format!("manager-http-{index}");
            let imported=DownloadService::import_from_json(&serde_json::json!({"downloads":[{"gameId":game_id,"title":game_id,"platform":"ps2","uris":[uri],"checksum":checksum}]}).to_string()).unwrap();
            let source=imported[0].clone();
            let job=DownloadService::create_job(crate::models::CreateDownloadRequest{game_id:game_id.clone(),platform:"ps2".into(),source:source.clone()}).unwrap();
            let mut changed=source; changed.uri="http://127.0.0.1:1/changed".into(); DownloadService::create_source(changed).unwrap();
            start(job.id.clone()).unwrap();
            let deadline=Instant::now()+Duration::from_secs(10);
            let finished=loop {
                let current=DownloadService::get_job(&job.id).unwrap().unwrap();
                if matches!(current.status,DownloadStatus::Downloaded|DownloadStatus::Failed) {break current;}
                assert!(Instant::now()<deadline,"job did not finish"); thread::sleep(Duration::from_millis(10));
            };
            if index==0 {
                assert!(matches!(finished.status,DownloadStatus::Downloaded));
                assert_eq!(finished.provider.as_deref(),Some("http"));
                assert_eq!(fs::read(Path::new(&finished.destination_path).join("content.dat")).unwrap(),b"data");
                assert!(Path::new(&finished.destination_path).join(".emubox-managed").is_file());
                assert!(!super::super::GameService::get_game_by_id(game_id).unwrap().unwrap().installed);
                assert!(matches!(start(job.id).unwrap().status,DownloadStatus::Downloaded));
            } else { assert!(matches!(finished.status,DownloadStatus::Failed)); assert!(!Path::new(&finished.destination_path).exists()); }
        }
        worker.join().unwrap();
    }
}