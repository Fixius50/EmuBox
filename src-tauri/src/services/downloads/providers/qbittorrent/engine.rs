use super::api;
use super::failure;
use crate::{errors::EmuBoxError, services::downloads::providers::io_error};
use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::{
    blocking::{Client, Response},
    header::{COOKIE, ORIGIN, REFERER},
};
use ring::{
    pbkdf2,
    rand::{SecureRandom, SystemRandom},
};
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

pub(super) fn private_dir(path: &Path) -> Result<(), EmuBoxError> {
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

pub(super) struct EngineProcess {
    child: Child,
    client: Client,
    endpoint: String,
    cookie: String,
    _lock: fs::File,
}

impl EngineProcess {
    pub(super) fn request(&self, route: &str) -> reqwest::blocking::RequestBuilder {
        self.client
            .post(format!("{}/api/v2/{route}", self.endpoint))
            .header(ORIGIN, &self.endpoint)
            .header(REFERER, format!("{}/", self.endpoint))
            .header(COOKIE, &self.cookie)
    }

    pub(super) fn call(&self, route: &str, fields: &[(&str, &str)]) -> Result<String, EmuBoxError> {
        let response = self
            .request(route)
            .form(fields)
            .send()
            .map_err(|_| failure("API local qBittorrent no responde"))?;
        response_text(response)
    }

    pub(super) fn set_cookie(&mut self, cookie: String) {
        self.cookie = cookie;
    }

    pub(super) fn start(root: &Path, discovery: bool) -> Result<Self, EmuBoxError> {
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
            if api::authenticate(&mut engine, password.as_str())? {
                let version = api::version(&engine)?;
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
                api::apply_preferences(&engine, discovery)?;
                return Ok(engine);
            }
            thread::sleep(Duration::from_millis(150));
        }
        Err(failure(
            "qBittorrent no confirmo su API privada autenticada; no se usa la instancia personal",
        ))
    }

    pub(super) fn shutdown(&mut self) -> Result<(), EmuBoxError> {
        api::stop_all(self);
        api::shutdown_app(self);
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

pub(super) fn response_text(response: Response) -> Result<String, EmuBoxError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credentials_and_config_keep_local_auth_and_unlimited_downloads() {
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
}
