use super::{http::HttpProvider, io_error, DownloadProvider};
use crate::{
    errors::EmuBoxError,
    models::{TransferOutcome, TransferProgress, TransferRequest},
};
use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::{
    blocking::{multipart, Client, Response},
    header::{COOKIE, ORIGIN, REFERER, SET_COOKIE},
};
use ring::{
    pbkdf2,
    rand::{SecureRandom, SystemRandom},
};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    num::NonZeroU32,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Component, Path, PathBuf},
    process::{Child, Command, Stdio},
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

fn session_cookie(headers: &reqwest::header::HeaderMap) -> Option<String> {
    headers
        .get_all(SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .filter_map(|value| value.split(';').next())
        .map(str::trim)
        .find(|value| {
            value.split_once('=').is_some_and(|(name, cookie)| {
                !cookie.is_empty() && (name == "SID" || name.starts_with("QBT_SID_"))
            })
        })
        .map(str::to_owned)
}

fn added_one(text: &str) -> bool {
    if matches!(text.trim(), "" | "Ok.") {
        return true;
    }
    serde_json::from_str::<serde_json::Value>(text)
        .ok()
        .is_some_and(|value| {
            value["failure_count"].as_u64() == Some(0)
                && value["success_count"].as_u64().unwrap_or(0)
                    + value["pending_count"].as_u64().unwrap_or(0)
                    == 1
        })
}

fn private_dir(path: &Path) -> Result<(), EmuBoxError> {
    if !path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(failure("Directorio qBittorrent invalido"));
    }
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => (),
            Ok(_) => {
                return Err(failure(
                    "Directorio qBittorrent contiene enlaces o archivos inesperados",
                ))
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir(&current).map_err(io_error)?;
                fs::set_permissions(&current, fs::Permissions::from_mode(0o700))
                    .map_err(io_error)?;
            }
            Err(error) => return Err(io_error(error)),
        }
    }
    Ok(())
}

fn credentials() -> Result<(String, String), EmuBoxError> {
    let random = SystemRandom::new();
    let mut bytes = [0u8; 32];
    let mut salt = [0u8; 16];
    random
        .fill(&mut bytes)
        .map_err(|_| failure("No se pudo generar credencial local"))?;
    random
        .fill(&mut salt)
        .map_err(|_| failure("No se pudo generar sal local"))?;
    let password = STANDARD.encode(bytes);
    let mut hash = [0u8; 64];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA512,
        NonZeroU32::new(100000).unwrap(),
        &salt,
        password.as_bytes(),
        &mut hash,
    );
    Ok((
        password,
        format!("{}:{}", STANDARD.encode(salt), STANDARD.encode(hash)),
    ))
}

fn profile_config(port: u16, hash: &str, discovery: bool) -> String {
    let web = format!("Address=127.0.0.1\nPort={port}\nUsername=emubox\nPassword_PBKDF2=\"@ByteArray({hash})\"\nLocalHostAuth=true\nAuthSubnetWhitelistEnabled=false\nCSRFProtection=true\nHostHeaderValidation=true\nServerDomains=127.0.0.1\nUseUPnP=false\n");
    let legacy = web
        .lines()
        .map(|line| format!("WebUI\\{line}\n"))
        .collect::<String>();
    format!("[LegalNotice]\nAccepted=true\n[Preferences]\n{legacy}Connection\\UPnP=false\n[WebUI]\n{web}[BitTorrent]\nSession\\DHTEnabled={discovery}\nSession\\PeXEnabled={discovery}\nSession\\LSDEnabled=false\nSession\\GlobalDLSpeedLimit=0\nSession\\GlobalUPSpeedLimit=64\nSession\\AddTorrentStopped=true\nSession\\AddTorrentPaused=true\n[AutoRun]\nenabled=false\nOnTorrentAdded\\Enabled=false\n")
}

struct EngineProcess {
    child: Child,
    client: Client,
    endpoint: String,
    cookie: String,
    _lock: fs::File,
}

impl EngineProcess {
    fn request(&self, route: &str) -> reqwest::blocking::RequestBuilder {
        self.client
            .post(format!("{}/api/v2/{route}", self.endpoint))
            .header(ORIGIN, &self.endpoint)
            .header(REFERER, format!("{}/", self.endpoint))
            .header(COOKIE, &self.cookie)
    }
    fn call(&self, route: &str, fields: &[(&str, &str)]) -> Result<String, EmuBoxError> {
        let response = self
            .request(route)
            .form(fields)
            .send()
            .map_err(|_| failure("API local qBittorrent no responde"))?;
        response_text(response)
    }
    fn start(root: &Path, discovery: bool) -> Result<Self, EmuBoxError> {
        private_dir(root)?;
        let lock_path = root.join("engine.lock");
        if fs::symlink_metadata(&lock_path)
            .is_ok_and(|metadata| !metadata.is_file() || metadata.file_type().is_symlink())
        {
            return Err(failure("Bloqueo qBittorrent inseguro"));
        }
        let lock = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .mode(0o600)
            .open(lock_path)
            .map_err(io_error)?;
        lock.try_lock()
            .map_err(|_| failure("Este trabajo ya tiene un motor qBittorrent activo"))?;
        let profile = root.join("profile");
        let config_dir = profile.join("qBittorrent/config");
        private_dir(&config_dir)?;
        let config = config_dir.join("qBittorrent.conf");
        if fs::symlink_metadata(&config)
            .is_ok_and(|metadata| !metadata.is_file() || metadata.file_type().is_symlink())
        {
            return Err(failure("Configuracion qBittorrent insegura"));
        }
        let listener = TcpListener::bind("127.0.0.1:0").map_err(io_error)?;
        let port = listener.local_addr().map_err(io_error)?.port();
        let (password, hash) = credentials()?;
        fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(config)
            .map_err(io_error)?
            .write_all(profile_config(port, &hash, discovery).as_bytes())
            .map_err(io_error)?;
        let client = Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(3))
            .build()
            .map_err(io_error)?;
        drop(listener);
        let mut command = Command::new("/usr/bin/nice");
        command
            .args([
                "-n",
                "5",
                "/usr/bin/ionice",
                "-c",
                "2",
                "-n",
                "7",
                "--",
                "/usr/bin/qbittorrent-nox",
            ])
            .arg(format!("--profile={}", profile.display()))
            .arg(format!("--webui-port={port}"))
            .arg("--confirm-legal-notice")
            .env("LC_ALL", "C")
            .env_remove("QT_PLUGIN_PATH")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        use std::os::unix::process::CommandExt;
        let parent = std::process::id() as libc::pid_t;
        unsafe {
            command.pre_exec(move || {
                if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                if libc::getppid() != parent {
                    return Err(std::io::Error::from_raw_os_error(libc::ECHILD));
                }
                Ok(())
            });
        }
        let child = command
            .spawn()
            .map_err(|_| failure("Instala qbittorrent-nox (>=5.0), coreutils y util-linux"))?;
        let mut engine = Self {
            child,
            client,
            endpoint: format!("http://127.0.0.1:{port}"),
            cookie: String::new(),
            _lock: lock,
        };
        let deadline = Instant::now() + Duration::from_secs(15);
        while Instant::now() < deadline && engine.child.try_wait().map_err(io_error)?.is_none() {
            if let Ok(response) = engine
                .request("auth/login")
                .form(&[("username", "emubox"), ("password", password.as_str())])
                .send()
            {
                let cookie = session_cookie(response.headers());
                if response.status().is_success()
                    && cookie.is_some()
                    && matches!(response_text(response)?.trim(), "" | "Ok.")
                {
                    engine.cookie = cookie.unwrap();
                    let version = engine.call("app/version", &[])?;
                    if version
                        .trim_start_matches('v')
                        .split('.')
                        .next()
                        .and_then(|value| value.parse::<u32>().ok())
                        .unwrap_or(0)
                        < 5
                    {
                        return Err(failure("Se requiere qBittorrent 5.0 o superior"));
                    }
                    engine.call("app/setPreferences", &[("json", &json!({"dl_limit":0,"limit_utp_rate":true,"up_limit":65536,"dht":discovery,"pex":discovery,"lsd":false,"upnp":false,"autorun_enabled":false,"autorun_on_torrent_added_enabled":false}).to_string())])?;
                    return Ok(engine);
                }
            }
            thread::sleep(Duration::from_millis(150));
        }
        Err(failure(
            "qBittorrent no confirmo su API privada autenticada; no se usa la instancia personal",
        ))
    }
    fn shutdown(&mut self) -> Result<(), EmuBoxError> {
        let _ = self.call("torrents/stop", &[("hashes", "all")]);
        let _ = self.call("app/shutdown", &[]);
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            if self.child.try_wait().map_err(io_error)?.is_some() {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(100));
        }
        Err(failure(
            "qBittorrent no termino dentro del plazo; datos conservados para revalidacion",
        ))
    }
}
impl Drop for EngineProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn response_text(response: Response) -> Result<String, EmuBoxError> {
    if !response.status().is_success() {
        return Err(failure(format!(
            "qBittorrent devolvio HTTP {}",
            response.status()
        )));
    }
    let mut bytes = Vec::new();
    response
        .take(8 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| failure("Respuesta qBittorrent incompleta"))?;
    if bytes.len() > 8 * 1024 * 1024 {
        return Err(failure("Respuesta qBittorrent excesiva"));
    }
    String::from_utf8(bytes).map_err(|_| failure("Respuesta qBittorrent no UTF-8"))
}

#[derive(Deserialize)]
struct Status {
    hash: String,
    state: String,
    amount_left: i64,
    total_size: i64,
    completed: u64,
    dlspeed: u64,
}

fn validated_files(root: &Path, text: &str) -> Result<Vec<PathBuf>, EmuBoxError> {
    #[derive(Deserialize)]
    struct File {
        name: String,
        progress: f64,
    }
    let files: Vec<File> = serde_json::from_str(text)
        .map_err(|_| failure("Lista de archivos qBittorrent invalida"))?;
    if files.is_empty() {
        return Err(failure("Torrent completo sin archivos"));
    }
    files
        .into_iter()
        .map(|file| {
            let relative = Path::new(&file.name);
            if relative.as_os_str().is_empty()
                || relative
                    .components()
                    .any(|component| !matches!(component, Component::Normal(_)))
                || file.progress < 1.0
            {
                return Err(failure("Archivo torrent incompleto o fuera del trabajo"));
            }
            let path = root.join(relative);
            let resolved = fs::canonicalize(&path).map_err(io_error)?;
            if !resolved.starts_with(root)
                || !resolved.is_file()
                || fs::symlink_metadata(&path)
                    .map_err(io_error)?
                    .file_type()
                    .is_symlink()
            {
                return Err(failure(
                    "qBittorrent intento publicar una ruta no confinada",
                ));
            }
            Ok(path)
        })
        .collect()
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
        let existing: Vec<Status> = serde_json::from_str(&engine.call("torrents/info", &[])?)
            .map_err(|_| failure("Estado qBittorrent invalido"))?;
        if existing.is_empty() {
            let response = engine
                .request("torrents/add")
                .multipart(form)
                .send()
                .map_err(|_| failure("No se pudo enviar el torrent a qBittorrent"))?;
            if !added_one(&response_text(response)?) {
                return Err(failure("qBittorrent rechazo el descriptor o magnet"));
            }
        } else if existing.len() != 1 {
            return Err(failure(
                "Perfil qBittorrent contiene torrents ajenos al trabajo",
            ));
        }
        let deadline = Instant::now() + Duration::from_secs(15);
        let hash = loop {
            let entries: Vec<Status> = serde_json::from_str(&engine.call("torrents/info", &[])?)
                .map_err(|_| failure("Estado qBittorrent invalido"))?;
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
        engine.call("torrents/start", &[("hashes", &hash)])?;
        loop {
            if request.control.interrupted() {
                engine.shutdown()?;
                return Ok(TransferOutcome::Interrupted);
            }
            let entries: Vec<Status> =
                serde_json::from_str(&engine.call("torrents/info", &[("hashes", &hash)])?)
                    .map_err(|_| failure("Estado qBittorrent invalido"))?;
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
                let files = validated_files(
                    &payload,
                    &engine.call("torrents/files", &[("hash", &hash)])?,
                )?;
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
    #[test]
    fn credentials_and_config_keep_local_auth_and_unlimited_downloads() {
        let metadata: Status = serde_json::from_str(r#"{"hash":"fixture","state":"metaDL","amount_left":-1,"total_size":-1,"completed":0,"dlspeed":0}"#).unwrap();
        assert!(u64::try_from(metadata.total_size).is_err());
        assert!(added_one(
            r#"{"success_count":1,"failure_count":0,"pending_count":0}"#
        ));
        assert!(!added_one(
            r#"{"success_count":0,"failure_count":1,"pending_count":0}"#
        ));
        assert!(!added_one("Fails."));
        let mut headers = reqwest::header::HeaderMap::new();
        headers.append(SET_COOKIE, "other=value; Path=/".parse().unwrap());
        headers.append(
            SET_COOKIE,
            "QBT_SID_fixture=opaque; HttpOnly; Path=/".parse().unwrap(),
        );
        assert_eq!(
            session_cookie(&headers).as_deref(),
            Some("QBT_SID_fixture=opaque")
        );
        headers.insert(SET_COOKIE, "SID=legacy; HttpOnly".parse().unwrap());
        assert_eq!(session_cookie(&headers).as_deref(), Some("SID=legacy"));
        let (password, hash) = credentials().unwrap();
        let (salt, expected) = hash.split_once(':').unwrap();
        assert!(pbkdf2::verify(
            pbkdf2::PBKDF2_HMAC_SHA512,
            NonZeroU32::new(100000).unwrap(),
            &STANDARD.decode(salt).unwrap(),
            password.as_bytes(),
            &STANDARD.decode(expected).unwrap()
        )
        .is_ok());
        let config = profile_config(12345, &hash, false);
        assert!(config.contains("Address=127.0.0.1"));
        assert!(config.contains("LocalHostAuth=true"));
        assert!(config.contains("GlobalDLSpeedLimit=0"));
        assert!(!config.contains(&password));
    }
    #[test]
    fn completed_files_reject_escape_and_incomplete_data() {
        let root = std::env::temp_dir().join(format!("emubox-qbit-files-{}", std::process::id()));
        private_dir(&root).unwrap();
        fs::write(root.join("valid.bin"), b"data").unwrap();
        assert_eq!(
            validated_files(&root, r#"[{"name":"valid.bin","progress":1}]"#)
                .unwrap()
                .len(),
            1
        );
        for text in [
            r#"[{"name":"../escape","progress":1}]"#,
            r#"[{"name":"/etc/passwd","progress":1}]"#,
            r#"[{"name":"valid.bin","progress":0.5}]"#,
            "[]",
        ] {
            assert!(validated_files(&root, text).is_err());
        }
        fs::remove_dir_all(root).unwrap();
    }

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
