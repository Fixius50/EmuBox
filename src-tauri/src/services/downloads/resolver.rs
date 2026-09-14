use crate::{
    errors::EmuBoxError,
    models::{DownloadSource, ProviderId},
};

pub fn source_option(source: DownloadSource) -> crate::models::DownloadSourceOption {
    let access = source_access(&source.uri).unwrap_or("unsupported");
    let resolution = resolve(&source);
    let downloadable = resolution.is_ok();
    let provider = resolution
        .as_ref()
        .ok()
        .map(|provider| provider.as_str().to_string());
    let connector = crate::services::download_connectors::connector(&source.uri).map(str::to_string);
    let reason = resolution.err().map(|error| error.to_string()).or_else(|| {
        if connector.as_deref() == Some("1fichier_account_api") { Some("Requiere cuenta autorizada; la API comprobara permisos y cuota al iniciar. Puede consumir creditos CDN segun tu oferta".into()) }
        else if provider.as_deref() == Some("bittorrent") { Some("BitTorrent puede subir piezas durante la descarga (limite 64 KiB/s); sin seeding al completar".into()) }
        else if access == "unverified_http" || connector.is_some() { Some("Acceso sujeto a disponibilidad y limites del servidor; no se eluden restricciones".into()) }
        else { None }
    });
    crate::models::DownloadSourceOption {
        downloadable,
        source,
        access: access.into(),
        reason,
        provider,
        connector,
    }
}

pub fn resolve(source: &DownloadSource) -> Result<ProviderId, EmuBoxError> {
    if !source.available {
        return Err(EmuBoxError::InvalidConfiguration(
            "Fuente marcada como no disponible".into(),
        ));
    }
    if crate::services::download_connectors::connector(&source.uri).is_some() {
        crate::services::download_connectors::check_configuration(&source.uri)?;
        return Ok(ProviderId::Http);
    }
    match source_access(&source.uri) {
        Some("http" | "unverified_http") => Ok(ProviderId::Http),
        Some("magnet" | "torrent") => {
            if source.uri.starts_with("magnet:") && !valid_magnet(&source.uri) {
                return Err(EmuBoxError::InvalidConfiguration(
                    "El proveedor BitTorrent requiere un magnet con infohash btih valido".into(),
                ));
            }
            if crate::services::binary_service::resolve_executable("aria2c").is_none() {
                Err(EmuBoxError::ExecutableMissing(
                    "Proveedor BitTorrent: instala el paquete aria2 (aria2c)".into(),
                ))
            } else {
                Ok(ProviderId::BitTorrent)
            }
        }
        Some("host_page") => {
            let host = reqwest::Url::parse(&source.uri)
                .ok()
                .and_then(|url| url.host_str().map(str::to_string))
                .unwrap_or_default();
            Err(EmuBoxError::InvalidConfiguration(format!(
                "Falta conector autorizado para {host}; no se eluden login ni CAPTCHA"
            )))
        }
        _ => {
            let scheme = reqwest::Url::parse(&source.uri)
                .ok()
                .map(|url| url.scheme().to_string())
                .unwrap_or_else(|| "invalido".into());
            Err(EmuBoxError::InvalidConfiguration(format!(
                "No hay proveedor registrado para el protocolo {scheme}"
            )))
        }
    }
}

pub fn source_access(uri: &str) -> Option<&'static str> {
    let url = reqwest::Url::parse(uri).ok()?;
    if url.scheme() == "magnet" {
        return Some("magnet");
    }
    if matches!(url.scheme(), "javascript" | "data" | "file" | "blob") {
        return None;
    }
    if !matches!(url.scheme(), "http" | "https") {
        return Some("unsupported");
    }
    let host = url.host_str()?.trim_start_matches("www.");
    let path = url.path().to_ascii_lowercase();
    if host == "pixeldrain.com" && path.starts_with("/api/file/") {
        return Some("http");
    }
    if [
        "megadb.net",
        "gofile.io",
        "mediafire.com",
        "1fichier.com",
        "pixeldrain.com",
        "mega.nz",
        "datanodes.to",
        "buzzheavier.com",
        "bzzhr.to",
        "1337x.to",
        "rutor.info",
        "tapochek.net",
        "t.me",
        "vikingfile.com",
        "files.fm",
        "akirabox.com",
        "filekeeper.net",
    ]
    .iter()
    .any(|domain| host == *domain || host.ends_with(&format!(".{domain}")))
    {
        return Some("host_page");
    }
    if path.ends_with(".torrent") {
        return Some("torrent");
    }
    if [
        "zip", "7z", "rar", "iso", "chd", "pkg", "exe", "bin", "gz", "xz", "rvz", "gba", "sfc",
    ]
    .iter()
    .any(|extension| path.ends_with(&format!(".{extension}")))
    {
        Some("http")
    } else {
        Some("unverified_http")
    }
}

fn valid_magnet(uri: &str) -> bool {
    reqwest::Url::parse(uri).ok().is_some_and(|url| {
        url.query_pairs().any(|(key, value)| {
            key == "xt"
                && value.strip_prefix("urn:btih:").is_some_and(|hash| {
                    (hash.len() == 40
                        && hash.chars().all(|character| character.is_ascii_hexdigit()))
                        || (hash.len() == 32
                            && hash.chars().all(|character| {
                                character.is_ascii_alphabetic() || matches!(character, '2'..='7')
                            }))
                })
        })
    })
}
