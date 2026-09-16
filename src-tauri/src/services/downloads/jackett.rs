use crate::{
    errors::EmuBoxError,
    models::{DownloadSource, DownloadSourceType, JackettResult},
    services::{DownloadService, GameService},
};
use ring::rand::{SecureRandom, SystemRandom};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs,
    io::Read,
    os::unix::fs::MetadataExt,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

struct Candidate {
    game_id: String,
    source: DownloadSource,
    created: Instant,
}
static CANDIDATES: OnceLock<Mutex<HashMap<String, Candidate>>> = OnceLock::new();
static SEARCH: Mutex<()> = Mutex::new(());
fn candidates() -> &'static Mutex<HashMap<String, Candidate>> {
    CANDIDATES.get_or_init(|| Mutex::new(HashMap::new()))
}
fn error(message: &str) -> EmuBoxError {
    EmuBoxError::ProcessFailed(message.into())
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Config {
    #[serde(rename = "APIKey")]
    api_key: String,
    port: u16,
    allow_external: bool,
    local_bind_address: String,
}

fn configuration() -> Result<Config, EmuBoxError> {
    let path = "/var/lib/emubox/jackett/Jackett/ServerConfig.json";
    let metadata = fs::symlink_metadata(path).map_err(|_| {
        error("Jackett no configurado; ejecuta el instalador local de servicios de busqueda")
    })?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > 65536
        || metadata.mode() & 0o007 != 0
    {
        return Err(error(
            "Configuracion Jackett debe ser privada, regular y menor de 64 KiB",
        ));
    }
    let config: Config = serde_json::from_slice(
        &fs::read(path).map_err(|_| error("No se puede leer configuracion Jackett"))?,
    )
    .map_err(|_| error("Configuracion Jackett invalida"))?;
    if config.allow_external
        || config.local_bind_address != "127.0.0.1"
        || config.port == 0
        || config.api_key.len() < 16
        || config.api_key.len() > 256
    {
        return Err(error(
            "Jackett debe escuchar solo en 127.0.0.1 y tener una clave local valida",
        ));
    }
    Ok(config)
}

fn parse_results(body: &Value) -> Vec<(String, String, String, Option<u64>, Option<u64>)> {
    body["Results"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let title = entry["Title"].as_str()?.trim();
            let magnet = entry["MagnetUri"].as_str().or_else(|| {
                entry["Link"]
                    .as_str()
                    .filter(|uri| uri.starts_with("magnet:"))
            })?;
            if title.is_empty()
                || title.len() > 1024
                || magnet.len() > 8192
                || !super::resolver::valid_magnet(magnet)
            {
                return None;
            }
            Some((
                title.into(),
                entry["Tracker"]
                    .as_str()
                    .unwrap_or("Jackett")
                    .chars()
                    .take(120)
                    .collect(),
                magnet.into(),
                entry["Size"].as_u64(),
                entry["Seeders"].as_u64(),
            ))
        })
        .take(50)
        .collect()
}

pub fn search(game_id: &str) -> Result<Vec<JackettResult>, EmuBoxError> {
    let _search = SEARCH
        .try_lock()
        .map_err(|_| error("Ya hay una busqueda Jackett en curso"))?;
    let game = GameService::get_game_by_id(game_id.into())?
        .ok_or_else(|| error("Version de juego inexistente"))?;
    let query = game.canonical_title.as_deref().unwrap_or(&game.title);
    let config = configuration()?;
    let client = reqwest::blocking::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(2))
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|_| error("No se pudo preparar cliente Jackett"))?;
    let response = client
        .get(format!(
            "http://127.0.0.1:{}/api/v2.0/indexers/all/results",
            config.port
        ))
        .query(&[
            ("apikey", config.api_key.as_str()),
            ("Query", query),
            ("Category[]", "1000"),
            ("Category[]", "4000"),
        ])
        .send()
        .map_err(|_| {
            error(
                "Jackett local no responde dentro de 30 segundos; comprueba emubox-jackett.service",
            )
        })?;
    if !response.status().is_success() {
        return Err(error(
            "Jackett rechazo la consulta; comprueba indexadores y configuracion local",
        ));
    }
    let mut bytes = Vec::new();
    response
        .take(2 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| error("Respuesta Jackett incompleta"))?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err(error("Respuesta Jackett excede 2 MiB"));
    }
    let body: Value =
        serde_json::from_slice(&bytes).map_err(|_| error("Respuesta Jackett invalida"))?;
    if !body["Results"].is_array() {
        return Err(error("Jackett no devolvio resultados validos"));
    }
    let mut cache = candidates()
        .lock()
        .map_err(|_| error("Cache de resultados no disponible"))?;
    cache.retain(|_, value| {
        value.created.elapsed() < Duration::from_secs(600) && value.game_id != game_id
    });
    if cache.len() > 200 {
        cache.clear();
    }
    let mut results = Vec::new();
    for (title, tracker, uri, size_bytes, seeders) in parse_results(&body) {
        let mut random = [0u8; 16];
        SystemRandom::new()
            .fill(&mut random)
            .map_err(|_| error("No se pudo crear identificador de resultado"))?;
        let id = random
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let source = DownloadSource {
            id: format!(
                "jackett-{:x}",
                Sha256::digest(format!("{game_id}\0{uri}").as_bytes())
            ),
            game_id: game_id.into(),
            name: format!("{tracker}: {title}"),
            source_type: DownloadSourceType::Magnet,
            uri,
            size_bytes,
            checksum: None,
            available: true,
        };
        cache.insert(
            id.clone(),
            Candidate {
                game_id: game_id.into(),
                source,
                created: Instant::now(),
            },
        );
        results.push(JackettResult {
            id,
            title,
            tracker,
            size_bytes,
            seeders,
        });
    }
    Ok(results)
}

pub fn select(game_id: &str, result_id: &str) -> Result<DownloadSource, EmuBoxError> {
    let cache = candidates()
        .lock()
        .map_err(|_| error("Cache de resultados no disponible"))?;
    let candidate = cache
        .get(result_id)
        .filter(|candidate| {
            candidate.game_id == game_id && candidate.created.elapsed() < Duration::from_secs(600)
        })
        .ok_or_else(|| error("Resultado caducado o ajeno a la version; repite la busqueda"))?;
    if GameService::get_game_by_id(game_id.into())?.is_none() {
        return Err(error("Version inexistente"));
    }
    DownloadService::create_source(candidate.source.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn results_keep_only_valid_magnets_without_private_links() {
        let body = serde_json::json!({"Results":[
            {"Title":"Fixture","Tracker":"Local","MagnetUri":"magnet:?xt=urn:btih:0123456789012345678901234567890123456789","Size":4,"Seeders":1},
            {"Title":"Private","Link":"http://127.0.0.1:9117/dl?apikey=hidden"},
            {"Title":"Invalid","MagnetUri":"magnet:?xt=invalid"}
        ]});
        let parsed = parse_results(&body);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].0, "Fixture");
        assert_eq!(parsed[0].3, Some(4));
        assert!(select("other-game", "absent").is_err());
    }
}
