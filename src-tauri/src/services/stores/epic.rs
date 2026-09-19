use super::{storage_error, StoreService};
use crate::{errors::EmuBoxError, models::StoreSyncResult, services::db_service::DatabaseService};
use rusqlite::params;
use serde_json::Value;
use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

const LEGENDARY: &str = "/var/lib/emubox/stores/epic/runtime/bin/legendary";
pub(super) fn start_authorization() -> Result<(), EmuBoxError> {
    let root = epic_root()?;
    private_dir(&root)?;
    require_legendary()?;
    Command::new("/usr/bin/xdg-open")
        .arg("https://legendary.gl/epiclogin")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| {
            EmuBoxError::ProcessFailed("No se pudo abrir la autorizacion de Epic".into())
        })?;
    set_state("authorization_required", None, None)?;
    Ok(())
}

pub(super) fn complete_authorization(code: String) -> Result<(), EmuBoxError> {
    let root = epic_root()?;
    private_dir(&root)?;
    require_legendary()?;
    let script = "import os,sys; from legendary.core import LegendaryCore; code=sys.stdin.read().strip(); core=LegendaryCore(os.environ['LEGENDARY_CONFIG_PATH']); ok=bool(code) and core.auth_code(code); core.exit(); raise SystemExit(0 if ok else 1)";
    private_python(
        &root,
        script,
        &code,
        "Epic no pudo completar la autorizacion",
    )?;
    set_authorization("connected")
}

pub(super) fn sync_library() -> Result<StoreSyncResult, EmuBoxError> {
    let root = epic_root()?;
    private_dir(&root)?;
    require_legendary()?;
    set_state("syncing", None, None)?;
    let status = match legendary_json(&root, &["status", "--json"]) {
        Ok(status) => status,
        Err(error) => return sync_failure(error),
    };
    let account_name = status
        .get("account")
        .and_then(Value::as_str)
        .filter(|name| *name != "<not logged in>")
        .ok_or_else(|| EmuBoxError::ProcessFailed("Epic no tiene una sesion valida".into()));
    let account_name = match account_name {
        Ok(name) => name,
        Err(error) => return sync_failure(error),
    };
    let identity = match read_identity(&root) {
        Ok(identity) => identity,
        Err(error) => return sync_failure(error),
    };
    let games = match legendary_json(&root, &["list", "--json"]) {
        Ok(Value::Array(games)) => games,
        Ok(_) => {
            return sync_failure(EmuBoxError::ProcessFailed(
                "Legendary devolvio una biblioteca invalida".into(),
            ))
        }
        Err(error) => return sync_failure(error),
    };
    let installed = match legendary_json(&root, &["list-installed", "--json"]) {
        Ok(Value::Array(games)) => games
            .iter()
            .filter_map(|game| {
                game.get("app_name")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            })
            .collect::<BTreeSet<_>>(),
        Ok(_) => {
            return sync_failure(EmuBoxError::ProcessFailed(
                "Legendary devolvio instalaciones invalidas".into(),
            ))
        }
        Err(error) => return sync_failure(error),
    };
    let now = unix_time()?;
    let account_id = format!("epic:{identity}");
    let mut connection = DatabaseService::get_connection()?;
    let transaction = connection.transaction().map_err(storage_error)?;
    transaction
        .execute(
            "INSERT INTO store_accounts(id,provider,external_account_id,display_name,status,last_login_at,last_sync_at) VALUES (?1,'epic',?2,?3,'authenticated',?4,?4) ON CONFLICT(provider,external_account_id) DO UPDATE SET id=excluded.id,display_name=excluded.display_name,status='authenticated',last_login_at=excluded.last_login_at,last_sync_at=excluded.last_sync_at",
            params![account_id, identity, account_name, now],
        )
        .map_err(storage_error)?;
    let mut imported = 0u32;
    for game in games {
        let Some(app_id) = game.get("app_name").and_then(Value::as_str) else {
            continue;
        };
        let Some(title) = game.get("app_title").and_then(Value::as_str) else {
            continue;
        };
        let metadata = serde_json::json!({"source": "legendary", "appName": app_id});
        transaction
            .execute(
                "INSERT INTO store_games(provider,external_game_id,title,metadata_json,updated_at) VALUES ('epic',?1,?2,?3,?4) ON CONFLICT(provider,external_game_id) DO UPDATE SET title=excluded.title,metadata_json=excluded.metadata_json,updated_at=excluded.updated_at",
                params![app_id, title, metadata.to_string(), now],
            )
            .map_err(storage_error)?;
        transaction
            .execute(
                "INSERT INTO store_entitlements(store_account_id,provider,external_game_id,owned,installed,last_seen_at) VALUES (?1,'epic',?2,1,?3,?4) ON CONFLICT(store_account_id,provider,external_game_id) DO UPDATE SET owned=1,installed=excluded.installed,last_seen_at=excluded.last_seen_at",
                params![account_id, app_id, installed.contains(app_id), now],
            )
            .map_err(storage_error)?;
        imported += 1;
    }
    transaction
        .execute(
            "UPDATE store_provider_states SET status='ready',last_sync_at=?1,error_message=NULL,updated_at=?1 WHERE provider='epic'",
            [now],
        )
        .map_err(storage_error)?;
    transaction.commit().map_err(storage_error)?;
    Ok(StoreSyncResult {
        provider: "epic".into(),
        imported_games: imported,
        installed_games: installed.len() as u32,
    })
}

pub(super) fn disconnect() -> Result<(), EmuBoxError> {
    let root = epic_root()?;
    private_dir(&root)?;
    require_legendary()?;
    legendary(&root, &["auth", "--delete"]).map_err(|_| {
        EmuBoxError::ProcessFailed("Legendary no pudo cerrar la sesion de Epic".into())
    })?;
    let now = unix_time()?;
    let connection = DatabaseService::get_connection()?;
    connection
        .execute(
            "UPDATE store_accounts SET status='signed_out' WHERE provider='epic'",
            [],
        )
        .map_err(storage_error)?;
    connection
        .execute("UPDATE store_provider_states SET status='authorization_required',authorization_status='required',error_message=NULL,updated_at=?1 WHERE provider='epic'", [now])
        .map_err(storage_error)?;
    Ok(())
}

fn epic_root() -> Result<PathBuf, EmuBoxError> {
    StoreService::session_directory("epic")
}

fn require_legendary() -> Result<(), EmuBoxError> {
    let metadata = fs::metadata(LEGENDARY).map_err(|_| {
        EmuBoxError::ExecutableMissing(
            "Instala el adaptador Epic de EmuBox antes de conectar la cuenta".into(),
        )
    })?;
    if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
        return Err(EmuBoxError::ExecutableMissing(
            "El adaptador Epic no es ejecutable".into(),
        ));
    }
    Ok(())
}

fn legendary_json(root: &Path, arguments: &[&str]) -> Result<Value, EmuBoxError> {
    let output = legendary(root, arguments)?;
    serde_json::from_slice(&output)
        .map_err(|_| EmuBoxError::ProcessFailed("Legendary devolvio JSON invalido".into()))
}

fn legendary(root: &Path, arguments: &[&str]) -> Result<Vec<u8>, EmuBoxError> {
    let output = Command::new("/usr/bin/timeout")
        .args(["--kill-after=1s", "90s", LEGENDARY])
        .args(arguments)
        .env("HOME", root.join("home"))
        .env("XDG_CONFIG_HOME", root.join("config"))
        .env("LEGENDARY_CONFIG_PATH", root.join("legendary"))
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .output()
        .map_err(|_| EmuBoxError::ProcessFailed("No se pudo ejecutar Legendary".into()))?;
    if !output.status.success() {
        return Err(EmuBoxError::ProcessFailed(
            "Legendary no pudo validar o sincronizar la cuenta Epic".into(),
        ));
    }
    Ok(output.stdout)
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
        .env("XDG_CONFIG_HOME", root.join("config"))
        .env("LEGENDARY_CONFIG_PATH", root.join("legendary"))
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

fn read_identity(root: &Path) -> Result<String, EmuBoxError> {
    let path = root.join("legendary/user.json");
    let metadata = fs::symlink_metadata(&path).map_err(|_| {
        EmuBoxError::ProcessFailed("Legendary no genero identidad de cuenta".into())
    })?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(EmuBoxError::ProcessFailed("Identidad Epic insegura".into()));
    }
    let value: Value = serde_json::from_slice(
        &fs::read(path)
            .map_err(|_| EmuBoxError::ProcessFailed("No se pudo leer la identidad Epic".into()))?,
    )
    .map_err(|_| EmuBoxError::ProcessFailed("Identidad Epic invalida".into()))?;
    value
        .get("account_id")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            EmuBoxError::ProcessFailed("Legendary no proporciono el identificador Epic".into())
        })
}

fn private_dir(path: &Path) -> Result<(), EmuBoxError> {
    if !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(EmuBoxError::InvalidConfiguration(
            "Directorio Epic invalido".into(),
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
    DatabaseService::get_connection()?
        .execute(
            "UPDATE store_provider_states SET status=?1,last_sync_at=?2,error_message=?3,updated_at=?4 WHERE provider='epic'",
            params![status, last_sync_at, error_message, now],
        )
        .map_err(storage_error)?;
    Ok(())
}

fn set_authorization(status: &str) -> Result<(), EmuBoxError> {
    let now = unix_time()?;
    DatabaseService::get_connection()?.execute("UPDATE store_provider_states SET authorization_status=?1,error_message=NULL,updated_at=?2 WHERE provider='epic'", params![status, now]).map_err(storage_error)?;
    Ok(())
}

fn sync_failure(error: EmuBoxError) -> Result<StoreSyncResult, EmuBoxError> {
    let _ = set_state(
        "error",
        None,
        Some("No se pudo sincronizar Epic; vuelve a iniciar sesion o revisa el adaptador."),
    );
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
