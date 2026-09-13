use super::{DownloadProvider, io_error, safe_name};
use crate::{errors::EmuBoxError, models::{TransferRequest, TransferOutcome, TransferProgress}};
use reqwest::{header, StatusCode};
use serde::{Deserialize, Serialize};
use std::{fs::{self, OpenOptions}, io::Write, time::{Duration, Instant}};

pub struct HttpProvider;

#[derive(Serialize, Deserialize)]
struct ResumeIdentity { uri: String, etag: Option<String> }

fn network(error: reqwest::Error) -> EmuBoxError {
    EmuBoxError::IpcError(if error.is_timeout() { "HTTP: tiempo de espera agotado; el parcial se conserva" }
        else { "HTTP: fallo de conexion, TLS o lectura; no se ha desactivado la validacion del certificado" }.into())
}

pub fn content_range(value: &str) -> Option<(u64, u64, u64)> {
    let (range, total) = value.strip_prefix("bytes ")?.split_once('/')?;
    let (start, end) = range.split_once('-')?;
    let (start, end, total) = (start.parse().ok()?, end.parse().ok()?, total.parse().ok()?);
    (start <= end && end < total).then_some((start, end, total))
}

impl DownloadProvider for HttpProvider {
    fn transfer(&self, request: &TransferRequest<'_>, progress: &mut dyn FnMut(TransferProgress) -> Result<(), EmuBoxError>) -> Result<TransferOutcome, EmuBoxError> {
        tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(io_error)?
            .block_on(transfer(request, progress))
    }
}

async fn transfer(request: &TransferRequest<'_>, progress: &mut dyn FnMut(TransferProgress) -> Result<(), EmuBoxError>) -> Result<TransferOutcome, EmuBoxError> {
    if request.control.interrupted() { return Ok(TransferOutcome::Interrupted); }
    let url = reqwest::Url::parse(request.uri).map_err(|_| EmuBoxError::InvalidConfiguration("URI HTTP invalida".into()))?;
    if !matches!(url.scheme(), "http" | "https") || !url.username().is_empty() || url.password().is_some() {
        return Err(EmuBoxError::InvalidConfiguration("HTTP requiere URL sin credenciales embebidas".into()));
    }
    fs::create_dir_all(request.directory).map_err(io_error)?;
    let partial = request.directory.join("payload.part");
    let identity_path = request.directory.join("resume.json");
    let identity = fs::read(&identity_path).ok().and_then(|bytes| serde_json::from_slice::<ResumeIdentity>(&bytes).ok());
    let validator = identity.as_ref().filter(|identity| identity.uri == request.uri).and_then(|identity| identity.etag.as_deref()).filter(|etag| !etag.starts_with("W/"));
    let offset = if validator.is_some() { partial.metadata().map(|metadata| metadata.len()).unwrap_or(0) } else { 0 };
    let client = reqwest::Client::builder().connect_timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::limited(5)).build().map_err(network)?;
    let mut builder = client.get(url).header(header::ACCEPT_ENCODING, "identity");
    if offset > 0 { builder = builder.header(header::RANGE, format!("bytes={offset}-")).header(header::IF_RANGE, validator.unwrap()); }
    let mut response = tokio::time::timeout(Duration::from_secs(30), builder.send()).await
        .map_err(|_| EmuBoxError::IpcError("HTTP: timeout de cabeceras".into()))?.map_err(network)?;
    if !matches!(response.url().scheme(), "http" | "https") { return Err(EmuBoxError::InvalidConfiguration("Redireccion a protocolo no permitido".into())); }
    if response.status() == StatusCode::RANGE_NOT_SATISFIABLE {
        let _ = fs::remove_file(&identity_path);
        return Err(EmuBoxError::IpcError("HTTP 416: parcial obsoleto; reanudar reiniciara la transferencia".into()));
    }
    if !response.status().is_success() { return Err(EmuBoxError::IpcError(format!("HTTP {}: fuente no accesible", response.status().as_u16()))); }
    let media = response.headers().get(header::CONTENT_TYPE).and_then(|value| value.to_str().ok()).unwrap_or("").to_ascii_lowercase();
    if media.contains("text/html") || media.contains("application/xhtml") { return Err(EmuBoxError::InvalidConfiguration("La fuente devuelve HTML; requiere un conector autorizado".into())); }
    let etag = response.headers().get(header::ETAG).and_then(|value| value.to_str().ok()).map(str::to_string);
    let filename = response.headers().get(header::CONTENT_DISPOSITION).and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').find_map(|part| part.trim().strip_prefix("filename=").map(|name| name.trim_matches('"'))))
        .map(safe_name).unwrap_or_else(|| safe_name(request.filename));
    let (mut downloaded, total) = if response.status() == StatusCode::PARTIAL_CONTENT {
        let range = response.headers().get(header::CONTENT_RANGE).and_then(|value| value.to_str().ok()).and_then(content_range)
            .ok_or_else(|| EmuBoxError::IpcError("HTTP 206 sin Content-Range valido".into()))?;
        if range.0 != offset || range.1 != range.2 - 1 || (offset > 0 && etag.as_deref() != validator) {
            return Err(EmuBoxError::IpcError("HTTP: rango o identidad del parcial no coinciden".into()));
        }
        (offset, Some(range.2))
    } else { (0, response.content_length()) };
    let resumed = downloaded;
    let mut file = OpenOptions::new().create(true).write(true).truncate(downloaded == 0).append(downloaded > 0).open(&partial).map_err(io_error)?;
    let metadata = serde_json::to_vec(&ResumeIdentity { uri: request.uri.into(), etag }).map_err(io_error)?;
    fs::write(&identity_path, metadata).map_err(io_error)?;
    let started = Instant::now();
    let mut reported = Instant::now();
    loop {
        if request.control.interrupted() { file.sync_all().map_err(io_error)?; return Ok(TransferOutcome::Interrupted); }
        let chunk = tokio::time::timeout(Duration::from_secs(30), response.chunk()).await
            .map_err(|_| EmuBoxError::IpcError("HTTP: timeout de lectura; parcial conservado".into()))?.map_err(network)?;
        let Some(chunk) = chunk else { break; };
        if downloaded == 0 {
            let prefix = String::from_utf8_lossy(&chunk[..chunk.len().min(512)]).trim_start().to_ascii_lowercase();
            if prefix.starts_with("<!doctype html") || prefix.starts_with("<html") { return Err(EmuBoxError::InvalidConfiguration("Contenido HTML no descargable como juego".into())); }
        }
        file.write_all(&chunk).map_err(io_error)?;
        downloaded += chunk.len() as u64;
        if request.max_bytes.is_some_and(|limit| downloaded > limit) { return Err(EmuBoxError::InvalidConfiguration("El descriptor supera el limite permitido".into())); }
        if total.is_some_and(|total| downloaded > total) { return Err(EmuBoxError::IpcError("HTTP: contenido excede el tamano declarado".into())); }
        if reported.elapsed() >= Duration::from_millis(500) {
            progress(TransferProgress { downloaded, total, speed: ((downloaded - resumed) as f64 / started.elapsed().as_secs_f64().max(0.001)) as u64 })?;
            reported = Instant::now();
        }
    }
    if total.is_some_and(|total| downloaded != total) || downloaded == 0 { return Err(EmuBoxError::IpcError("HTTP: contenido vacio o incompleto".into())); }
    file.sync_all().map_err(io_error)?;
    drop(file);
    if request.control.interrupted() { return Ok(TransferOutcome::Interrupted); }
    let artifact = request.directory.join(filename);
    fs::rename(&partial, &artifact).map_err(io_error)?;
    progress(TransferProgress { downloaded, total: Some(downloaded), speed: 0 })?;
    Ok(TransferOutcome::Complete(vec![artifact]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TransferControl;
    use std::{net::TcpListener, io::{Read, Write}, thread};

    #[test]
    fn validates_range_syntax() {
        assert_eq!(content_range("bytes 4-7/8"), Some((4,7,8)));
        assert_eq!(content_range("bytes 7-4/8"), None);
        assert_eq!(content_range("bytes 0-8/8"), None);
    }

    #[test]
    fn local_http_transfer_does_not_publish_or_install() {
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let uri = format!("http://{}/content.bin", server.local_addr().unwrap());
        let worker = thread::spawn(move || {
            let (mut stream, _) = server.accept().unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut buffer = [0; 4096]; let _ = stream.read(&mut buffer).unwrap();
            stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\nETag: \"v1\"\r\nConnection: close\r\n\r\ndata").unwrap();
        });
        let directory = std::env::temp_dir().join(format!("emubox-http-{}", std::process::id()));
        let control = TransferControl::default();
        let result = HttpProvider.transfer(&TransferRequest { uri: &uri, directory: &directory, filename: "content.bin", control: &control, max_bytes: None }, &mut |_| Ok(())).unwrap();
        let TransferOutcome::Complete(files) = result else { panic!("transfer interrupted"); };
        assert_eq!(fs::read(&files[0]).unwrap(), b"data");
        worker.join().unwrap(); fs::remove_dir_all(directory).unwrap();
    }
}