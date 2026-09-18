use super::{http::HttpProvider, io_error, DownloadProvider};
use crate::{
    errors::EmuBoxError,
    models::{TransferOutcome, TransferProgress, TransferRequest},
};
mod api;
mod engine;
mod files;
use api::{add_torrent, completed_files, start_torrent, statuses};
use engine::{private_dir, EngineProcess};
use reqwest::blocking::multipart;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    os::unix::fs::OpenOptionsExt,
    thread,
    time::{Duration, Instant},
};

pub struct QbittorrentProvider {
    pub discovery: bool,
}
impl Default for QbittorrentProvider {
    fn default() -> Self {
        Self { discovery: true }
    }
}

fn failure(message: impl Into<String>) -> EmuBoxError {
    EmuBoxError::ProcessFailed(message.into())
}

impl DownloadProvider for QbittorrentProvider {
    fn transfer(
        &self,
        request: &TransferRequest<'_>,
        progress: &mut dyn FnMut(TransferProgress) -> Result<(), EmuBoxError>,
    ) -> Result<TransferOutcome, EmuBoxError> {
        if request.control.interrupted() {
            return Ok(TransferOutcome::Interrupted);
        }
        let root = request.directory.join("qbittorrent");
        private_dir(&root)?;
        let identity = format!("{:x}", Sha256::digest(request.uri.as_bytes()));
        let marker = root.join("source.sha256");
        if marker.exists() {
            if fs::symlink_metadata(&marker)
                .map_err(io_error)?
                .file_type()
                .is_symlink()
                || fs::read_to_string(&marker).map_err(io_error)? != identity
            {
                return Err(failure("El perfil qBittorrent pertenece a otra fuente"));
            }
        } else {
            fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(marker)
                .map_err(io_error)?
                .write_all(identity.as_bytes())
                .map_err(io_error)?;
        }
        let payload = root.join("payload");
        private_dir(&payload)?;
        let payload = fs::canonicalize(payload).map_err(io_error)?;
        let descriptor = if request.uri.starts_with("magnet:") {
            None
        } else {
            match HttpProvider.transfer(
                &TransferRequest {
                    uri: request.uri,
                    directory: &root.join("metadata"),
                    filename: "descriptor.torrent",
                    control: request.control,
                    max_bytes: Some(16 * 1024 * 1024),
                },
                &mut |_| Ok(()),
            )? {
                TransferOutcome::Complete(files) => Some(fs::read(&files[0]).map_err(io_error)?),
                TransferOutcome::Interrupted => return Ok(TransferOutcome::Interrupted),
            }
        };
        let mut engine = EngineProcess::start(&root, self.discovery)?;
        let mut form = multipart::Form::new()
            .text("savepath", payload.to_string_lossy().into_owned())
            .text("autoTMM", "false")
            .text("contentLayout", "Original")
            .text("stopped", "true")
            .text("paused", "true");
        form = match descriptor {
            Some(bytes) => form.part(
                "torrents",
                multipart::Part::bytes(bytes).file_name("content.torrent"),
            ),
            None => form.text("urls", request.uri.to_owned()),
        };
        let existing = statuses(&engine, None)?;
        if existing.is_empty() {
            add_torrent(&engine, form)?;
        } else if existing.len() != 1 {
            return Err(failure(
                "Perfil qBittorrent contiene torrents ajenos al trabajo",
            ));
        }
        let deadline = Instant::now() + Duration::from_secs(15);
        let hash = loop {
            let entries = statuses(&engine, None)?;
            if entries.len() == 1 {
                break entries[0].hash.clone();
            }
            if entries.len() > 1 || Instant::now() >= deadline {
                return Err(failure(
                    "Perfil qBittorrent no contiene exactamente el torrent del trabajo",
                ));
            }
            if request.control.interrupted() {
                engine.shutdown()?;
                return Ok(TransferOutcome::Interrupted);
            }
            thread::sleep(Duration::from_millis(150));
        };
        start_torrent(&engine, &hash)?;
        loop {
            if request.control.interrupted() {
                engine.shutdown()?;
                return Ok(TransferOutcome::Interrupted);
            }
            let entries = statuses(&engine, Some(&hash))?;
            let status = entries
                .first()
                .ok_or_else(|| failure("El torrent desaparecio del perfil del trabajo"))?;
            if matches!(status.state.as_str(), "error" | "missingFiles" | "unknown") {
                return Err(failure(format!("qBittorrent: {}", status.state)));
            }
            let total = u64::try_from(status.total_size)
                .ok()
                .filter(|size| *size > 0);
            if request
                .max_bytes
                .is_some_and(|limit| total.is_some_and(|size| size > limit))
            {
                return Err(failure("Contenido supera el limite permitido"));
            }
            progress(TransferProgress {
                downloaded: status.completed,
                total,
                speed: status.dlspeed,
            })?;
            if request.control.interrupted() {
                engine.shutdown()?;
                return Ok(TransferOutcome::Interrupted);
            }
            if status.total_size > 0
                && status.amount_left == 0
                && matches!(
                    status.state.as_str(),
                    "uploading" | "stalledUP" | "queuedUP" | "stoppedUP" | "pausedUP" | "forcedUP"
                )
            {
                let files = completed_files(&engine, &payload, &hash)?;
                engine.shutdown()?;
                return Ok(TransferOutcome::Complete(files));
            }
            thread::sleep(Duration::from_millis(500));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::Read,
        net::TcpListener,
        thread,
        time::{Duration, Instant},
    };

    #[test]
    #[ignore = "Requires installed qBittorrent-nox >=5; transfers only four fixture bytes over loopback"]
    fn local_qbittorrent_downloads_private_fixture() {
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = server.local_addr().unwrap();
        server.set_nonblocking(true).unwrap();
        let seed = format!("http://{address}/content.bin");
        let mut torrent =
            b"d4:infod6:lengthi4e4:name11:content.bin12:piece lengthi16384e6:pieces20:".to_vec();
        for pair in "a17c9aaa61e80a1bf71d0d850af4e5baa9800bbd"
            .as_bytes()
            .chunks(2)
        {
            torrent.push(u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap());
        }
        torrent.extend_from_slice(
            format!("7:privatei1ee8:url-listl{}:{seed}ee", seed.len()).as_bytes(),
        );
        let control = crate::models::TransferControl::default();
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let worker_stop = stop.clone();
        let worker_control = control.clone();
        let worker = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(35);
            while Instant::now() < deadline
                && !worker_stop.load(std::sync::atomic::Ordering::Relaxed)
            {
                if let Ok((mut stream, _)) = server.accept() {
                    stream
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .unwrap();
                    let mut request = [0u8; 8192];
                    let count = stream.read(&mut request).unwrap();
                    let body = if request[..count].starts_with(b"GET /descriptor.torrent ") {
                        torrent.as_slice()
                    } else {
                        b"data"
                    };
                    write!(
                        stream,
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    )
                    .unwrap();
                    stream.write_all(body).unwrap();
                } else {
                    thread::sleep(Duration::from_millis(10));
                }
            }
            worker_control
                .cancelled
                .store(true, std::sync::atomic::Ordering::Relaxed);
        });
        let root =
            std::env::temp_dir().join(format!("emubox-qbit-integration-{}", std::process::id()));
        let request = TransferRequest {
            uri: &format!("http://{address}/descriptor.torrent"),
            directory: &root,
            filename: "descriptor.torrent",
            control: &control,
            max_bytes: None,
        };
        let provider = QbittorrentProvider { discovery: false };
        let paused = provider.transfer(&request, &mut |_| {
            control
                .paused
                .store(true, std::sync::atomic::Ordering::Relaxed);
            Ok(())
        });
        assert!(matches!(paused, Ok(TransferOutcome::Interrupted)));
        control
            .paused
            .store(false, std::sync::atomic::Ordering::Relaxed);
        let result = provider.transfer(&request, &mut |_| Ok(()));
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        worker.join().unwrap();
        let TransferOutcome::Complete(files) = result.unwrap() else {
            panic!("Fixture did not complete")
        };
        assert_eq!(files.len(), 1);
        assert_eq!(fs::read(&files[0]).unwrap(), b"data");
        fs::remove_dir_all(root).unwrap();
    }
}
