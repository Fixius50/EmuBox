use super::{storage_error, StoreService};
use crate::{errors::EmuBoxError, models::StoreSyncResult, services::db_service::DatabaseService};
use reqwest::blocking::Client;
use rusqlite::params;
use serde::Deserialize;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const AUTH_SCRIPT: &str = "/opt/emubox/scripts/store-steam-auth.sh";

#[derive(Deserialize)]
struct Credentials {
    api_key: String,
    steam_id: String,
}

#[derive(Deserialize)]
struct OwnedGamesResponse {
    response: OwnedGames,
}

#[derive(Deserialize)]
struct OwnedGames {
    games: Option<Vec<OwnedGame>>,
}

#[derive(Deserialize)]
struct OwnedGame {
    appid: u64,
    name: String,
}

#[derive(Deserialize)]
struct PlayerSummariesResponse {
    response: PlayerSummaries,
}

#[derive(Deserialize)]
struct PlayerSummaries {
    players: Vec<Player>,
}

#[derive(Deserialize)]
struct Player {
    personaname: String,
}

pub(super) fn start_authorization() -> Result<(), EmuBoxError> {
    let root = steam_root()?;
    private_dir(&root)?;
    Command::new("/usr/bin/footclient")
        .arg("--")
        .arg(AUTH_SCRIPT)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| EmuBoxError::ProcessFailed("No se pudo abrir la terminal de Steam".into()))?;
    set_state("authorization_required", None, None)
}

pub(super) fn sync_library() -> Result<StoreSyncResult, EmuBoxError> {
    let root = steam_root()?;
    private_dir(&root)?;
    set_state("syncing", None, None)?;
    let credentials = match credentials(&root) {
        Ok(credentials) => credentials,
        Err(error) => return sync_failure(error),
    };
    let client = match Client::builder().timeout(Duration::from_secs(20)).build() {
        Ok(client) => client,
        Err(_) => {
            return sync_failure(EmuBoxError::ProcessFailed(
                "No se pudo preparar la conexion Steam".into(),
            ))
        }
    };
    let display_name = match account_name(&client, &credentials) {
        Ok(name) => name,
        Err(error) => return sync_failure(error),
    };
    let games = match owned_games(&client, &credentials) {
        Ok(games) => games,
        Err(error) => return sync_failure(error),
    };
    let now = unix_time()?;
    let account_id = format!("steam:{}", credentials.steam_id);
    let mut connection = DatabaseService::get_connection()?;
    let transaction = connection.transaction().map_err(storage_error)?;
    transaction
        .execute(
            "INSERT INTO store_accounts(id,provider,external_account_id,display_name,status,last_login_at,last_sync_at) VALUES (?1,'steam',?2,?3,'authenticated',?4,?4) ON CONFLICT(provider,external_account_id) DO UPDATE SET id=excluded.id,display_name=excluded.display_name,status='authenticated',last_login_at=excluded.last_login_at,last_sync_at=excluded.last_sync_at",
            params![account_id, credentials.steam_id, display_name, now],
        )
        .map_err(storage_error)?;
    let mut imported = 0u32;
    for game in games {
        let external_id = game.appid.to_string();
        let metadata = serde_json::json!({"source": "steam-web-api", "appId": game.appid});
        transaction
            .execute(
                "INSERT INTO store_games(provider,external_game_id,title,metadata_json,updated_at) VALUES ('steam',?1,?2,?3,?4) ON CONFLICT(provider,external_game_id) DO UPDATE SET title=excluded.title,metadata_json=excluded.metadata_json,updated_at=excluded.updated_at",
                params![external_id, game.name, metadata.to_string(), now],
            )
            .map_err(storage_error)?;
        transaction
            .execute(
                "INSERT INTO store_entitlements(store_account_id,provider,external_game_id,owned,installed,last_seen_at) VALUES (?1,'steam',?2,1,0,?3) ON CONFLICT(store_account_id,provider,external_game_id) DO UPDATE SET owned=1,last_seen_at=excluded.last_seen_at",
                params![account_id, external_id, now],
            )
            .map_err(storage_error)?;
        imported += 1;
    }
    transaction
        .execute(
            "UPDATE store_provider_states SET status='ready',last_sync_at=?1,error_message=NULL,updated_at=?1 WHERE provider='steam'",
            [now],
        )
        .map_err(storage_error)?;
    transaction.commit().map_err(storage_error)?;
    Ok(StoreSyncResult {
        provider: "steam".into(),
        imported_games: imported,
        installed_games: 0,
    })
}

pub(super) fn disconnect() -> Result<(), EmuBoxError> {
    let root = steam_root()?;
    private_dir(&root)?;
    let credentials = root.join("credentials.json");
    if fs::symlink_metadata(&credentials).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(EmuBoxError::ProcessFailed(
            "Credenciales Steam inseguras".into(),
        ));
    }
    if credentials.exists() {
        fs::remove_file(credentials)
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
    }
    let now = unix_time()?;
    let connection = DatabaseService::get_connection()?;
    connection
        .execute(
            "UPDATE store_accounts SET status='signed_out' WHERE provider='steam'",
            [],
        )
        .map_err(storage_error)?;
    connection.execute("UPDATE store_provider_states SET status='authorization_required',error_message=NULL,updated_at=?1 WHERE provider='steam'", [now]).map_err(storage_error)?;
    Ok(())
}

fn credentials(root: &Path) -> Result<Credentials, EmuBoxError> {
    let path = root.join("credentials.json");
    let metadata = fs::symlink_metadata(&path).map_err(|_| {
        EmuBoxError::ProcessFailed(
            "Configura tu clave Web API de Steam para conectar la cuenta".into(),
        )
    })?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.permissions().mode() & 0o077 != 0
    {
        return Err(EmuBoxError::ProcessFailed(
            "Credenciales Steam inseguras".into(),
        ));
    }
    let credentials: Credentials = serde_json::from_slice(&fs::read(path).map_err(|_| {
        EmuBoxError::ProcessFailed("No se pudieron leer las credenciales Steam".into())
    })?)
    .map_err(|_| EmuBoxError::ProcessFailed("Credenciales Steam invalidas".into()))?;
    if credentials.api_key.trim().is_empty()
        || !credentials
            .steam_id
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        return Err(EmuBoxError::ProcessFailed(
            "Credenciales Steam invalidas".into(),
        ));
    }
    Ok(credentials)
}

fn account_name(client: &Client, credentials: &Credentials) -> Result<String, EmuBoxError> {
    let response: PlayerSummariesResponse = get_json(
        client,
        "ISteamUser/GetPlayerSummaries/v0002/",
        credentials,
        &[("steamids", credentials.steam_id.as_str())],
    )?;
    Ok(response
        .response
        .players
        .into_iter()
        .next()
        .map(|player| player.personaname)
        .unwrap_or_else(|| credentials.steam_id.clone()))
}

fn owned_games(client: &Client, credentials: &Credentials) -> Result<Vec<OwnedGame>, EmuBoxError> {
    let response: OwnedGamesResponse = get_json(
        client,
        "IPlayerService/GetOwnedGames/v0001/",
        credentials,
        &[("include_appinfo", "1"), ("include_played_free_games", "1")],
    )?;
    Ok(response.response.games.unwrap_or_default())
}

fn get_json<T: for<'de> Deserialize<'de>>(
    client: &Client,
    endpoint: &str,
    credentials: &Credentials,
    extra: &[(&str, &str)],
) -> Result<T, EmuBoxError> {
    let mut query = vec![
        ("key", credentials.api_key.as_str()),
        ("steamid", credentials.steam_id.as_str()),
    ];
    query.extend_from_slice(extra);
    let response = client
        .get(format!("https://api.steampowered.com/{endpoint}"))
        .query(&query)
        .send()
        .map_err(|_| {
            EmuBoxError::ProcessFailed("Steam no respondio al consultar la biblioteca".into())
        })?
        .error_for_status()
        .map_err(|_| {
            EmuBoxError::ProcessFailed("Steam rechazo la clave Web API o la cuenta".into())
        })?;
    let body = response
        .text()
        .map_err(|_| EmuBoxError::ProcessFailed("Steam devolvio una respuesta invalida".into()))?;
    serde_json::from_str(&body)
        .map_err(|_| EmuBoxError::ProcessFailed("Steam devolvio una respuesta invalida".into()))
}

fn steam_root() -> Result<PathBuf, EmuBoxError> {
    StoreService::session_directory("steam")
}

fn private_dir(path: &Path) -> Result<(), EmuBoxError> {
    if !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(EmuBoxError::InvalidConfiguration(
            "Directorio Steam invalido".into(),
        ));
    }
    fs::create_dir_all(path).map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))
}

fn set_state(
    status: &str,
    last_sync_at: Option<i64>,
    error_message: Option<&str>,
) -> Result<(), EmuBoxError> {
    let now = unix_time()?;
    DatabaseService::get_connection()?.execute("UPDATE store_provider_states SET status=?1,last_sync_at=?2,error_message=?3,updated_at=?4 WHERE provider='steam'", params![status, last_sync_at, error_message, now]).map_err(storage_error)?;
    Ok(())
}

fn sync_failure(error: EmuBoxError) -> Result<StoreSyncResult, EmuBoxError> {
    let _ = set_state("error", None, Some("No se pudo sincronizar Steam; revisa la clave Web API y la visibilidad de la biblioteca."));
    Err(error)
}

fn unix_time() -> Result<i64, EmuBoxError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .map_err(|_| EmuBoxError::StorageUnavailable("Reloj del sistema invalido".into()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn private_directory_rejects_parent_components() {
        assert!(super::private_dir(std::path::Path::new("/tmp/../escape")).is_err());
    }
}
