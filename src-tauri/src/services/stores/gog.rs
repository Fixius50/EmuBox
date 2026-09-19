use super::{storage_error, StoreService};
use crate::{errors::EmuBoxError, models::StoreSyncResult, services::db_service::DatabaseService};
use reqwest::{blocking::Client, header::AUTHORIZATION};
use rusqlite::params;
use serde_json::Value;
use std::{
    fs,
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const GOGDL: &str = "/var/lib/emubox/stores/gog/runtime/bin/gogdl";
struct Credentials {
    access_token: String,
    user_id: String,
}

pub(super) fn start_authorization() -> Result<(), EmuBoxError> {
    let root = gog_root()?;
    private_dir(&root)?;
    require_gogdl()?;
    let script = "import os,webbrowser; from urllib.parse import parse_qs,quote,urlparse; from gogdl.auth import CLIENT_ID,CODE_URL; redirect=parse_qs(urlparse(CODE_URL).query)['redirect_uri'][0]; webbrowser.open('https://auth.gog.com/auth?client_id={}&redirect_uri={}&response_type=code'.format(quote(CLIENT_ID,safe=''),quote(redirect,safe='')))";
    Command::new(root.join("runtime/bin/python"))
        .args(["-c", script])
        .env("HOME", root.join("home"))
        .env("GOGDL_CONFIG_PATH", root.join("config"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| {
            EmuBoxError::ProcessFailed("No se pudo abrir la autorizacion de GOG".into())
        })?;
    set_state("authorization_required", None, None)
}

pub(super) fn complete_authorization(code: String) -> Result<(), EmuBoxError> {
    let root = gog_root()?;
    private_dir(&root)?;
    require_gogdl()?;
    let script = "import contextlib,io,os,sys; from types import SimpleNamespace; from gogdl.auth import AuthorizationManager; code=sys.stdin.read().strip(); manager=AuthorizationManager(os.environ['GOG_AUTH_FILE']); args=SimpleNamespace(authorization_code=code,client_id=None,client_secret=None); out=io.StringIO();\nwith contextlib.redirect_stdout(out): manager.handle_cli(args, []);\nraise SystemExit(0 if code and '\"error\": true' not in out.getvalue().lower() else 1)";
    private_python(
        &root,
        script,
        &code,
        "GOG no pudo completar la autorizacion",
    )?;
    set_authorization("connected")
}

pub(super) fn sync_library() -> Result<StoreSyncResult, EmuBoxError> {
    let root = gog_root()?;
    private_dir(&root)?;
    require_gogdl()?;
    set_state("syncing", None, None)?;
    let credentials = match credentials(&root) {
        Ok(credentials) => credentials,
        Err(error) => return sync_failure(error),
    };
    let client = match Client::builder().timeout(Duration::from_secs(20)).build() {
        Ok(client) => client,
        Err(_) => {
            return sync_failure(EmuBoxError::ProcessFailed(
                "No se pudo preparar la conexion GOG".into(),
            ))
        }
    };
    let display_name = match account_name(&client, &credentials) {
        Ok(name) => name,
        Err(error) => return sync_failure(error),
    };
    let games = match library(&client, &credentials) {
        Ok(games) => games,
        Err(error) => return sync_failure(error),
    };
    let now = unix_time()?;
    let account_id = format!("gog:{}", credentials.user_id);
    let mut connection = DatabaseService::get_connection()?;
    let transaction = connection.transaction().map_err(storage_error)?;
    transaction
        .execute(
            "INSERT INTO store_accounts(id,provider,external_account_id,display_name,status,last_login_at,last_sync_at) VALUES (?1,'gog',?2,?3,'authenticated',?4,?4) ON CONFLICT(provider,external_account_id) DO UPDATE SET id=excluded.id,display_name=excluded.display_name,status='authenticated',last_login_at=excluded.last_login_at,last_sync_at=excluded.last_sync_at",
            params![account_id, credentials.user_id, display_name, now],
        )
        .map_err(storage_error)?;
    let mut imported = 0u32;
    for game in games {
        let Some(external_id) = game.get("external_id").and_then(value_string) else {
            continue;
        };
        let title = game
            .get("title")
            .and_then(value_string)
            .or_else(|| game.get("name").and_then(value_string))
            .unwrap_or_else(|| format!("GOG {external_id}"));
        let metadata =
            serde_json::json!({"source": "gog-galaxy-library", "externalId": external_id});
        transaction
            .execute(
                "INSERT INTO store_games(provider,external_game_id,title,metadata_json,updated_at) VALUES ('gog',?1,?2,?3,?4) ON CONFLICT(provider,external_game_id) DO UPDATE SET title=excluded.title,metadata_json=excluded.metadata_json,updated_at=excluded.updated_at",
                params![external_id, title, metadata.to_string(), now],
            )
            .map_err(storage_error)?;
        transaction
            .execute(
                "INSERT INTO store_entitlements(store_account_id,provider,external_game_id,owned,installed,last_seen_at) VALUES (?1,'gog',?2,1,0,?3) ON CONFLICT(store_account_id,provider,external_game_id) DO UPDATE SET owned=1,last_seen_at=excluded.last_seen_at",
                params![account_id, external_id, now],
            )
            .map_err(storage_error)?;
        imported += 1;
    }
    transaction
        .execute(
            "UPDATE store_provider_states SET status='ready',last_sync_at=?1,error_message=NULL,updated_at=?1 WHERE provider='gog'",
            [now],
        )
        .map_err(storage_error)?;
    transaction.commit().map_err(storage_error)?;
    Ok(StoreSyncResult {
        provider: "gog".into(),
        imported_games: imported,
        installed_games: 0,
    })
}

pub(super) fn disconnect() -> Result<(), EmuBoxError> {
    let root = gog_root()?;
    private_dir(&root)?;
    let auth = root.join("auth.json");
    if fs::symlink_metadata(&auth).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(EmuBoxError::ProcessFailed("Sesion GOG insegura".into()));
    }
    if auth.exists() {
        fs::remove_file(auth)
            .map_err(|error| EmuBoxError::StorageUnavailable(error.to_string()))?;
    }
    let now = unix_time()?;
    let connection = DatabaseService::get_connection()?;
    connection
        .execute(
            "UPDATE store_accounts SET status='signed_out' WHERE provider='gog'",
            [],
        )
        .map_err(storage_error)?;
    connection.execute("UPDATE store_provider_states SET status='authorization_required',authorization_status='required',error_message=NULL,updated_at=?1 WHERE provider='gog'", [now]).map_err(storage_error)?;
    Ok(())
}

fn credentials(root: &Path) -> Result<Credentials, EmuBoxError> {
    let output = Command::new("/usr/bin/timeout")
        .args(["--kill-after=1s", "30s", GOGDL, "--auth-config-path"])
        .arg(root.join("auth.json"))
        .arg("auth")
        .env("GOGDL_CONFIG_PATH", root.join("config"))
        .env("HOME", root.join("home"))
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .output()
        .map_err(|_| EmuBoxError::ProcessFailed("No se pudo ejecutar gogdl".into()))?;
    if !output.status.success() {
        return Err(EmuBoxError::ProcessFailed(
            "GOG no pudo validar la sesion".into(),
        ));
    }
    let value: Value = serde_json::from_slice(&output.stdout)
        .map_err(|_| EmuBoxError::ProcessFailed("gogdl devolvio una sesion invalida".into()))?;
    let access_token = value
        .get("access_token")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| EmuBoxError::ProcessFailed("GOG no tiene una sesion valida".into()))?;
    let user_id = value.get("user_id").and_then(value_string).ok_or_else(|| {
        EmuBoxError::ProcessFailed("GOG no proporciono identidad de cuenta".into())
    })?;
    Ok(Credentials {
        access_token: access_token.into(),
        user_id,
    })
}

fn private_python(
    root: &Path,
    script: &str,
    input: &str,
    message: &str,
) -> Result<(), EmuBoxError> {
    let mut child = Command::new(root.join("runtime/bin/python"))
        .args(["-c", script])
        .env("HOME", root.join("home"))
        .env("GOGDL_CONFIG_PATH", root.join("config"))
        .env("GOG_AUTH_FILE", root.join("auth.json"))
        .env("LC_ALL", "C")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| EmuBoxError::ProcessFailed(message.into()))?;
    child
        .stdin
        .take()
        .ok_or_else(|| EmuBoxError::ProcessFailed(message.into()))?
        .write_all(input.as_bytes())
        .map_err(|_| EmuBoxError::ProcessFailed(message.into()))?;
    if !child
        .wait()
        .map_err(|_| EmuBoxError::ProcessFailed(message.into()))?
        .success()
    {
        return Err(EmuBoxError::ProcessFailed(message.into()));
    }
    Ok(())
}

fn account_name(client: &Client, credentials: &Credentials) -> Result<String, EmuBoxError> {
    let response = client
        .get(format!(
            "https://users.gog.com/users/{}",
            credentials.user_id
        ))
        .header(
            AUTHORIZATION,
            format!("Bearer {}", credentials.access_token),
        )
        .send()
        .map_err(|_| EmuBoxError::ProcessFailed("GOG no respondio al consultar la cuenta".into()))?
        .error_for_status()
        .map_err(|_| EmuBoxError::ProcessFailed("GOG rechazo la sesion de cuenta".into()))?;
    let value = response_json(response, "GOG devolvio una cuenta invalida")?;
    Ok(value
        .get("username")
        .and_then(Value::as_str)
        .unwrap_or(&credentials.user_id)
        .into())
}

fn library(client: &Client, credentials: &Credentials) -> Result<Vec<Value>, EmuBoxError> {
    let mut games = Vec::new();
    let mut page_token: Option<String> = None;
    for _ in 0..1000 {
        let mut request = client
            .get(format!(
                "https://galaxy-library.gog.com/users/{}/releases",
                credentials.user_id
            ))
            .header(
                AUTHORIZATION,
                format!("Bearer {}", credentials.access_token),
            );
        if let Some(token) = &page_token {
            request = request.query(&[("page_token", token)]);
        }
        let response = request
            .send()
            .map_err(|_| {
                EmuBoxError::ProcessFailed("GOG no respondio al consultar la biblioteca".into())
            })?
            .error_for_status()
            .map_err(|_| {
                EmuBoxError::ProcessFailed("GOG rechazo la sesion de biblioteca".into())
            })?;
        let response = response_json(response, "GOG devolvio una biblioteca invalida")?;
        if let Some(items) = response.get("items").and_then(Value::as_array) {
            games.extend(
                items
                    .iter()
                    .filter(|item| item.get("platform_id").and_then(Value::as_str) == Some("gog"))
                    .cloned(),
            );
        }
        page_token = response
            .get("next_page_token")
            .and_then(Value::as_str)
            .filter(|token| !token.is_empty())
            .map(str::to_owned);
        if page_token.is_none() {
            return Ok(games);
        }
    }
    Err(EmuBoxError::ProcessFailed(
        "La biblioteca GOG excedio el limite de paginas".into(),
    ))
}

fn gog_root() -> Result<PathBuf, EmuBoxError> {
    StoreService::session_directory("gog")
}

fn require_gogdl() -> Result<(), EmuBoxError> {
    let metadata = fs::metadata(GOGDL).map_err(|_| {
        EmuBoxError::ExecutableMissing(
            "Instala el adaptador GOG de EmuBox antes de conectar la cuenta".into(),
        )
    })?;
    if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
        return Err(EmuBoxError::ExecutableMissing(
            "El adaptador GOG no es ejecutable".into(),
        ));
    }
    Ok(())
}

fn private_dir(path: &Path) -> Result<(), EmuBoxError> {
    if !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(EmuBoxError::InvalidConfiguration(
            "Directorio GOG invalido".into(),
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
    DatabaseService::get_connection()?.execute("UPDATE store_provider_states SET status=?1,last_sync_at=?2,error_message=?3,updated_at=?4 WHERE provider='gog'", params![status, last_sync_at, error_message, now]).map_err(storage_error)?;
    Ok(())
}

fn set_authorization(status: &str) -> Result<(), EmuBoxError> {
    let now = unix_time()?;
    DatabaseService::get_connection()?.execute("UPDATE store_provider_states SET authorization_status=?1,error_message=NULL,updated_at=?2 WHERE provider='gog'", params![status, now]).map_err(storage_error)?;
    Ok(())
}

fn sync_failure(error: EmuBoxError) -> Result<StoreSyncResult, EmuBoxError> {
    let _ = set_state(
        "error",
        None,
        Some("No se pudo sincronizar GOG; vuelve a iniciar sesion o revisa el adaptador."),
    );
    Err(error)
}

fn value_string(value: &Value) -> Option<String> {
    value
        .as_str()
        .map(str::to_owned)
        .or_else(|| value.as_i64().map(|value| value.to_string()))
}

fn response_json(
    response: reqwest::blocking::Response,
    message: &str,
) -> Result<Value, EmuBoxError> {
    let body = response
        .text()
        .map_err(|_| EmuBoxError::ProcessFailed(message.into()))?;
    serde_json::from_str(&body).map_err(|_| EmuBoxError::ProcessFailed(message.into()))
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
