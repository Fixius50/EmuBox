use super::{http::HttpProvider, io_error, DownloadProvider};
use crate::{
    errors::EmuBoxError,
    models::{TransferOutcome, TransferProgress, TransferRequest},
};
use base64::Engine;
use serde_json::{json, Value};
use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    os::unix::fs::OpenOptionsExt,
    path::PathBuf,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

pub struct BitTorrentProvider {
    pub discovery: bool,
}

impl Default for BitTorrentProvider {
    fn default() -> Self {
        Self { discovery: true }
    }
}

struct EngineProcess {
    child: Child,
    config: PathBuf,
    client: reqwest::blocking::Client,
    endpoint: String,
    token: String,
}

impl EngineProcess {
    fn call(&self, method: &str, mut parameters: Vec<Value>) -> Result<Value, EmuBoxError> {
        parameters.insert(0, Value::String(format!("token:{}", self.token)));
        let body = json!({"jsonrpc":"2.0", "id":"emubox", "method":method, "params":parameters})
            .to_string();
        let response = self
            .client
            .post(&self.endpoint)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .map_err(|_| EmuBoxError::ProcessFailed("Motor BitTorrent no responde".into()))?;
        let bytes = response
            .bytes()
            .map_err(|_| EmuBoxError::ProcessFailed("Respuesta BitTorrent incompleta".into()))?;
        let value: Value = serde_json::from_slice(&bytes)
            .map_err(|_| EmuBoxError::ProcessFailed("Respuesta BitTorrent invalida".into()))?;
        if value.get("error").is_some() {
            return Err(EmuBoxError::ProcessFailed(format!(
                "Operacion BitTorrent rechazada: {method}"
            )));
        }
        value
            .get("result")
            .cloned()
            .ok_or_else(|| EmuBoxError::ProcessFailed("BitTorrent no devolvio resultado".into()))
    }

    fn shutdown(&mut self) {
        let _ = self.call("aria2.shutdown", vec![]);
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            if self.child.try_wait().ok().flatten().is_some() {
                return;
            }
            thread::sleep(Duration::from_millis(50));
        }
    }
}

impl Drop for EngineProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_file(&self.config);
    }
}

impl DownloadProvider for BitTorrentProvider {
    fn transfer(
        &self,
        request: &TransferRequest<'_>,
        progress: &mut dyn FnMut(TransferProgress) -> Result<(), EmuBoxError>,
    ) -> Result<TransferOutcome, EmuBoxError> {
        let executable = crate::services::binary_service::resolve_executable("aria2c")
            .ok_or_else(|| EmuBoxError::ExecutableMissing("BitTorrent requiere aria2c".into()))?;
        fs::create_dir_all(request.directory).map_err(io_error)?;
        let payload = request.directory.join("payload");
        fs::create_dir_all(&payload).map_err(io_error)?;
        let descriptor = if reqwest::Url::parse(request.uri)
            .ok()
            .is_some_and(|url| url.scheme() != "magnet")
        {
            let directory = request.directory.join("metadata");
            let result = HttpProvider.transfer(
                &TransferRequest {
                    uri: request.uri,
                    directory: &directory,
                    filename: "descriptor.torrent",
                    control: request.control,
                    max_bytes: Some(16 * 1024 * 1024),
                },
                &mut |_| Ok(()),
            )?;
            match result {
                TransferOutcome::Complete(files) => Some(fs::read(&files[0]).map_err(io_error)?),
                TransferOutcome::Interrupted => return Ok(TransferOutcome::Interrupted),
            }
        } else {
            None
        };
        if request.control.interrupted() {
            return Ok(TransferOutcome::Interrupted);
        }
        let listener = TcpListener::bind("127.0.0.1:0").map_err(io_error)?;
        let port = listener.local_addr().map_err(io_error)?.port();
        let mut random = [0u8; 32];
        fs::File::open("/dev/urandom")
            .and_then(|mut file| file.read_exact(&mut random))
            .map_err(io_error)?;
        let token: String = random.iter().map(|byte| format!("{byte:02x}")).collect();
        let config = request.directory.join("aria2-private.conf");
        let mut file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o600)
            .open(&config)
            .map_err(io_error)?;
        writeln!(file, "enable-rpc=true\nrpc-listen-all=false\nrpc-listen-port={port}\nrpc-secret={token}\nno-netrc=true\nseed-time=0\nbt-hash-check-seed=false\ncheck-integrity=true\nauto-save-interval=1\nfile-allocation=none\nallow-overwrite=false\nauto-file-renaming=false\nfollow-metalink=false\nmax-concurrent-downloads=1\nmax-overall-upload-limit=64K\nbt-stop-timeout=300\nconsole-log-level=error\nquiet=true\nstop-with-process={}", std::process::id()).map_err(io_error)?;
        writeln!(
            file,
            "enable-dht={}\nenable-dht6={}\nenable-peer-exchange={}\nbt-enable-lpd=false",
            self.discovery, self.discovery, self.discovery
        )
        .map_err(io_error)?;
        drop(file);
        drop(listener);
        let client = reqwest::blocking::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(2))
            .build()
            .map_err(io_error)?;
        let child = Command::new(executable)
            .arg(format!("--conf-path={}", config.display()))
            .arg(format!("--dir={}", payload.display()))
            .arg(format!(
                "--dht-file-path={}",
                request.directory.join("dht.dat").display()
            ))
            .arg(format!(
                "--dht-file-path6={}",
                request.directory.join("dht6.dat").display()
            ))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(io_error)?;
        let mut engine = EngineProcess {
            child,
            config,
            client,
            endpoint: format!("http://127.0.0.1:{port}/jsonrpc"),
            token,
        };
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            if engine.call("aria2.getVersion", vec![]).is_ok() {
                break;
            }
            if Instant::now() >= deadline || engine.child.try_wait().map_err(io_error)?.is_some() {
                return Err(EmuBoxError::ProcessFailed(
                    "No se pudo iniciar aria2 con RPC privado".into(),
                ));
            }
            if request.control.interrupted() {
                engine.shutdown();
                return Ok(TransferOutcome::Interrupted);
            }
            thread::sleep(Duration::from_millis(100));
        }
        let gid = match descriptor {
            Some(bytes) => engine.call(
                "aria2.addTorrent",
                vec![json!(
                    base64::engine::general_purpose::STANDARD.encode(bytes)
                )],
            )?,
            None => engine.call("aria2.addUri", vec![json!([request.uri])])?,
        };
        let mut gid = gid
            .as_str()
            .ok_or_else(|| EmuBoxError::ProcessFailed("Identificador BitTorrent invalido".into()))?
            .to_string();
        loop {
            if request.control.interrupted() {
                engine.shutdown();
                return Ok(TransferOutcome::Interrupted);
            }
            let status = engine.call("aria2.tellStatus", vec![json!(gid)])?;
            if let Some(next) = status["followedBy"]
                .as_array()
                .and_then(|items| items.first())
                .and_then(Value::as_str)
            {
                gid = next.into();
                continue;
            }
            let number = |key: &str| {
                status[key]
                    .as_str()
                    .and_then(|value| value.parse::<u64>().ok())
                    .unwrap_or(0)
            };
            progress(TransferProgress {
                downloaded: number("completedLength"),
                total: (number("totalLength") > 0).then(|| number("totalLength")),
                speed: number("downloadSpeed"),
            })?;
            match status["status"].as_str() {
                Some("complete") => {
                    let root = fs::canonicalize(&payload).map_err(io_error)?;
                    let files = status["files"].as_array().ok_or_else(|| {
                        EmuBoxError::ProcessFailed("BitTorrent no devolvio archivos".into())
                    })?;
                    let mut artifacts = Vec::new();
                    for file in files {
                        let path = PathBuf::from(file["path"].as_str().ok_or_else(|| {
                            EmuBoxError::ProcessFailed("Ruta BitTorrent invalida".into())
                        })?);
                        let resolved = fs::canonicalize(&path).map_err(io_error)?;
                        if !resolved.starts_with(&root)
                            || fs::symlink_metadata(&path)
                                .map_err(io_error)?
                                .file_type()
                                .is_symlink()
                        {
                            return Err(EmuBoxError::InvalidConfiguration(
                                "BitTorrent intento publicar una ruta fuera del trabajo".into(),
                            ));
                        }
                        artifacts.push(path);
                    }
                    engine.shutdown();
                    if artifacts.is_empty() {
                        return Err(EmuBoxError::ProcessFailed(
                            "BitTorrent completo sin contenido".into(),
                        ));
                    }
                    return Ok(TransferOutcome::Complete(artifacts));
                }
                Some("error" | "removed") => {
                    return Err(EmuBoxError::ProcessFailed(format!(
                        "BitTorrent fallo (codigo {}). Verifica seeds, permisos y fuente",
                        number("errorCode")
                    )))
                }
                _ => {}
            }
            thread::sleep(Duration::from_millis(500));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TransferControl;

    #[test]
    #[ignore = "Requires real aria2c; transfers four private test bytes over localhost only"]
    fn local_torrent_descriptor_downloads_payload() {
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
        let worker = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(25);
            while Instant::now() < deadline {
                let Ok((mut stream, _)) = server.accept() else {
                    thread::sleep(Duration::from_millis(10));
                    continue;
                };
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut bytes = [0u8; 8192];
                let length = stream.read(&mut bytes).unwrap();
                let request = String::from_utf8_lossy(&bytes[..length]);
                let descriptor = request.starts_with("GET /descriptor.torrent ");
                let body = if descriptor {
                    torrent.as_slice()
                } else {
                    b"data"
                };
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                stream.write_all(header.as_bytes()).unwrap();
                stream.write_all(body).unwrap();
                if !descriptor {
                    return;
                }
            }
            panic!("BitTorrent did not request local payload");
        });
        let root = std::env::temp_dir().join(format!("emubox-torrent-{}", std::process::id()));
        let control = TransferControl::default();
        let result = BitTorrentProvider { discovery: false }
            .transfer(
                &TransferRequest {
                    uri: &format!("http://{address}/descriptor.torrent"),
                    directory: &root,
                    filename: "descriptor.torrent",
                    control: &control,
                    max_bytes: None,
                },
                &mut |_| Ok(()),
            )
            .unwrap();
        let TransferOutcome::Complete(files) = result else {
            panic!("interrupted");
        };
        assert_eq!(files.len(), 1);
        assert_eq!(fs::read(&files[0]).unwrap(), b"data");
        worker.join().unwrap();
        fs::remove_dir_all(root).unwrap();
    }
}
