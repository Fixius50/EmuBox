use super::{
    engine::{response_text, EngineProcess},
    failure,
    files::{validated_files, Status},
};
use crate::errors::EmuBoxError;
use reqwest::{blocking::multipart, header::SET_COOKIE};
use serde_json::json;
use std::path::{Path, PathBuf};

pub(super) fn authenticate(
    engine: &mut EngineProcess,
    password: &str,
) -> Result<bool, EmuBoxError> {
    let response = match engine
        .request("auth/login")
        .form(&[("username", "emubox"), ("password", password)])
        .send()
    {
        Ok(response) => response,
        Err(_) => return Ok(false),
    };
    let cookie = session_cookie(response.headers());
    if response.status().is_success()
        && cookie.is_some()
        && matches!(response_text(response)?.trim(), "" | "Ok.")
    {
        engine.set_cookie(cookie.unwrap());
        return Ok(true);
    }
    Ok(false)
}

pub(super) fn version(engine: &EngineProcess) -> Result<String, EmuBoxError> {
    engine.call("app/version", &[])
}

pub(super) fn apply_preferences(
    engine: &EngineProcess,
    discovery: bool,
) -> Result<(), EmuBoxError> {
    engine.call(
        "app/setPreferences",
        &[(
            "json",
            &json!({
                "dl_limit": 0,
                "limit_utp_rate": true,
                "up_limit": 65536,
                "dht": discovery,
                "pex": discovery,
                "lsd": false,
                "upnp": false,
                "autorun_enabled": false,
                "autorun_on_torrent_added_enabled": false,
            })
            .to_string(),
        )],
    )?;
    Ok(())
}

pub(super) fn add_torrent(
    engine: &EngineProcess,
    form: multipart::Form,
) -> Result<(), EmuBoxError> {
    let response = engine
        .request("torrents/add")
        .multipart(form)
        .send()
        .map_err(|_| failure("No se pudo enviar el torrent a qBittorrent"))?;
    if !added_one(&response_text(response)?) {
        return Err(failure("qBittorrent rechazo el descriptor o magnet"));
    }
    Ok(())
}

pub(super) fn statuses(
    engine: &EngineProcess,
    hash: Option<&str>,
) -> Result<Vec<Status>, EmuBoxError> {
    let text = match hash {
        Some(hash) => engine.call("torrents/info", &[("hashes", hash)])?,
        None => engine.call("torrents/info", &[])?,
    };
    serde_json::from_str(&text).map_err(|_| failure("Estado qBittorrent invalido"))
}

pub(super) fn start_torrent(engine: &EngineProcess, hash: &str) -> Result<(), EmuBoxError> {
    engine.call("torrents/start", &[("hashes", hash)])?;
    Ok(())
}

pub(super) fn stop_all(engine: &EngineProcess) {
    let _ = engine.call("torrents/stop", &[("hashes", "all")]);
}

pub(super) fn shutdown_app(engine: &EngineProcess) {
    let _ = engine.call("app/shutdown", &[]);
}

pub(super) fn completed_files(
    engine: &EngineProcess,
    payload: &Path,
    hash: &str,
) -> Result<Vec<PathBuf>, EmuBoxError> {
    validated_files(payload, &engine.call("torrents/files", &[("hash", hash)])?)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_response_accepts_one_success_only() {
        assert!(added_one(
            r#"{"success_count":1,"failure_count":0,"pending_count":0}"#
        ));
        assert!(!added_one(
            r#"{"success_count":0,"failure_count":1,"pending_count":0}"#
        ));
        assert!(!added_one("Fails."));
    }

    #[test]
    fn accepts_current_and_legacy_session_cookies() {
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
    }
}
