use crate::errors::EmuBoxError;
use std::{fs, io::Read, path::Path, time::Duration};

const ONEFICHIER_TOKEN: &str = "/etc/emubox/credentials/1fichier.token";

fn require_account_opt_in(value: Option<&str>) -> Result<(), EmuBoxError> {
    if value != Some("1") {
        return Err(configuration_error("Fuente con cuenta desactivada: EmuBox funciona sin registro. Selecciona otra fuente HTTP publica, Pixeldrain publico o BitTorrent; no se sustituye la fuente automaticamente"));
    }
    Ok(())
}

fn configuration_error(message: &str) -> EmuBoxError {
    EmuBoxError::InvalidConfiguration(message.into())
}

fn private_token(path: &Path) -> Result<String, EmuBoxError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| configuration_error("1fichier requiere credencial privada en /etc/emubox/credentials/1fichier.token y una cuenta con acceso API de descarga"))?;
    if !metadata.is_file() || metadata.len() > 4096 {
        return Err(configuration_error("Credencial 1fichier invalida"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(configuration_error(
                "La credencial 1fichier debe tener permisos 0600 o 0400",
            ));
        }
    }
    let token = fs::read_to_string(path)
        .map_err(|_| configuration_error("No se puede leer la credencial 1fichier"))?;
    let token = token.trim();
    if token.is_empty() || !token.bytes().all(|byte| byte.is_ascii_graphic()) {
        return Err(configuration_error("Credencial 1fichier vacia o invalida"));
    }
    Ok(token.into())
}

pub fn check_configuration(uri: &str) -> Result<(), EmuBoxError> {
    if connector(uri) == Some("1fichier_account_api") {
        require_account_opt_in(
            std::env::var("EMUBOX_ALLOW_ACCOUNT_DOWNLOADS")
                .ok()
                .as_deref(),
        )?;
        private_token(Path::new(ONEFICHIER_TOKEN))?;
    }
    Ok(())
}

pub fn connector(uri: &str) -> Option<&'static str> {
    let url = reqwest::Url::parse(uri).ok()?;
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
    {
        return None;
    }
    let host = url.host_str()?.trim_start_matches("www.");
    if host == "1fichier.com"
        && url.path() == "/"
        && url.query().is_some_and(|query| {
            !query.is_empty() && query.bytes().all(|byte| byte.is_ascii_alphanumeric())
        })
    {
        return Some("1fichier_account_api");
    }
    let segments: Vec<_> = url.path_segments()?.collect();
    (host == "pixeldrain.com"
        && segments.len() == 2
        && segments[0] == "u"
        && !segments[1].is_empty()
        && segments[1]
            .chars()
            .all(|character| character.is_ascii_alphanumeric()))
    .then_some("pixeldrain_public_file")
}

pub fn resolve_uri(uri: &str) -> Result<String, EmuBoxError> {
    match connector(uri) {
        Some("1fichier_account_api") => resolve_onefichier(uri),
        Some("pixeldrain_public_file") => {
            let url = reqwest::Url::parse(uri)
                .map_err(|_| EmuBoxError::InvalidConfiguration("URL Pixeldrain invalida".into()))?;
            let id = url
                .path_segments()
                .and_then(|mut segments| segments.nth(1))
                .unwrap();
            Ok(format!("https://pixeldrain.com/api/file/{id}"))
        }
        _ => Ok(uri.into()),
    }
}

fn onefichier_response(bytes: &[u8]) -> Result<String, EmuBoxError> {
    #[derive(serde::Deserialize)]
    struct Response {
        status: String,
        url: Option<String>,
    }
    let response: Response = serde_json::from_slice(bytes)
        .map_err(|_| configuration_error("Respuesta API 1fichier invalida"))?;
    if response.status != "OK" {
        return Err(configuration_error("1fichier denego la descarga: comprueba cuenta, cuota y permisos del archivo; no se reintenta automaticamente"));
    }
    let url = response
        .url
        .and_then(|value| reqwest::Url::parse(&value).ok())
        .ok_or_else(|| configuration_error("1fichier no devolvio un enlace de descarga valido"))?;
    let allowed_host = url
        .host_str()
        .is_some_and(|host| host == "1fichier.com" || host.ends_with(".1fichier.com"));
    if url.scheme() != "https"
        || !allowed_host
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
    {
        return Err(configuration_error(
            "1fichier devolvio un destino no autorizado",
        ));
    }
    Ok(url.into())
}

fn resolve_onefichier(uri: &str) -> Result<String, EmuBoxError> {
    require_account_opt_in(
        std::env::var("EMUBOX_ALLOW_ACCOUNT_DOWNLOADS")
            .ok()
            .as_deref(),
    )?;
    let token = private_token(Path::new(ONEFICHIER_TOKEN))?;
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(20))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| configuration_error("No se pudo iniciar el cliente API 1fichier"))?;
    let body = serde_json::to_vec(&serde_json::json!({"url": uri, "cdn": 0, "no_ssl": 0}))
        .map_err(|_| configuration_error("Solicitud API invalida"))?;
    let response = client
        .post("https://api.1fichier.com/v1/download/get_token.cgi")
        .bearer_auth(token)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(body)
        .send()
        .map_err(|_| {
            configuration_error("API 1fichier no disponible; no se reintenta automaticamente")
        })?;
    if !response.status().is_success() {
        return Err(configuration_error("API 1fichier rechazo la solicitud: comprueba credencial, suscripcion o limite de solicitudes"));
    }
    let mut bytes = Vec::new();
    response
        .take(65537)
        .read_to_end(&mut bytes)
        .map_err(|_| configuration_error("Respuesta API 1fichier incompleta"))?;
    if bytes.len() > 65536 {
        return Err(configuration_error(
            "Respuesta API 1fichier demasiado grande",
        ));
    }
    onefichier_response(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn account_connectors_are_disabled_without_explicit_opt_in() {
        for value in [None, Some(""), Some("0"), Some("true")] {
            assert!(require_account_opt_in(value).is_err());
        }
        assert!(require_account_opt_in(Some("1")).is_ok());
    }
    #[test]
    fn account_api_rejects_ambiguous_locators_and_untrusted_destinations() {
        assert_eq!(
            connector("https://1fichier.com/?abc123"),
            Some("1fichier_account_api")
        );
        for uri in [
            "ftp://1fichier.com/?abc123",
            "https://1fichier.com/?abc123&password=x",
            "https://1fichier.com/dir/abc",
            "https://1fichier.com.evil.test/?abc123",
            "https://user:secret@1fichier.com/?abc123",
        ] {
            assert_eq!(connector(uri), None);
        }
        assert_eq!(
            onefichier_response(br#"{"status":"OK","url":"https://a-1.1fichier.com/token"}"#)
                .unwrap(),
            "https://a-1.1fichier.com/token"
        );
        for body in [
            br#"{"status":"KO","message":"secret"}"#.as_slice(),
            br#"{"status":"OK","url":"http://a.1fichier.com/token"}"#,
            br#"{"status":"OK","url":"https://evil.test/token"}"#,
            br#"{"status":"OK"}"#,
        ] {
            let error = onefichier_response(body).unwrap_err().to_string();
            assert!(!error.contains("secret"));
        }
    }

    #[cfg(unix)]
    #[test]
    fn credential_must_be_a_private_regular_file() {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let root =
            std::env::temp_dir().join(format!("emubox-connector-token-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("token");
        fs::write(&path, "test-only-token").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(private_token(&path).is_err());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(private_token(&path).unwrap(), "test-only-token");
        symlink(&path, root.join("link")).unwrap();
        assert!(private_token(&root.join("link")).is_err());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn public_api_does_not_resolve_lists_or_other_domains() {
        assert_eq!(
            resolve_uri("https://pixeldrain.com/u/Abcd1234").unwrap(),
            "https://pixeldrain.com/api/file/Abcd1234"
        );
        assert_eq!(connector("https://pixeldrain.com/l/Abcd1234"), None);
        assert_eq!(
            connector("https://pixeldrain.com.other.test/u/Abcd1234"),
            None
        );
    }
}
