mod epic;
mod gog;
mod steam;

use crate::{
    errors::EmuBoxError,
    models::{StoreAccount, StoreEntitlement, StoreProviderInfo, StoreSyncResult, StoreSyncState},
    services::{db_service::DatabaseService, infrastructure::telemetry, paths},
};
use log::Level;
use rusqlite::OptionalExtension;
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

pub trait StoreProvider {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
}

struct BuiltInStoreProvider {
    id: &'static str,
    name: &'static str,
}

impl StoreProvider for BuiltInStoreProvider {
    fn id(&self) -> &'static str {
        self.id
    }

    fn name(&self) -> &'static str {
        self.name
    }
}

const PROVIDERS: [BuiltInStoreProvider; 3] = [
    BuiltInStoreProvider {
        id: "steam",
        name: "Steam",
    },
    BuiltInStoreProvider {
        id: "epic",
        name: "Epic Games",
    },
    BuiltInStoreProvider {
        id: "gog",
        name: "GOG",
    },
];

pub struct StoreService;

impl StoreService {
    pub fn start_epic_authorization() -> Result<(), EmuBoxError> {
        operation("epic", "connect.open", epic::start_authorization)?;
        store_event("epic", "connection.opened", Level::Info, json!({}));
        Ok(())
    }

    pub fn complete_epic_authorization(code: String) -> Result<(), EmuBoxError> {
        operation("epic", "connect.complete", || {
            epic::complete_authorization(code)
        })?;
        store_event("epic", "connection.connected", Level::Info, json!({}));
        Ok(())
    }

    pub fn sync_epic_library() -> Result<StoreSyncResult, EmuBoxError> {
        let result = operation("epic", "sync", epic::sync_library)?;
        sync_event(&result);
        Ok(result)
    }

    pub fn disconnect_epic() -> Result<(), EmuBoxError> {
        operation("epic", "connect.disconnect", epic::disconnect)?;
        store_event("epic", "connection.disconnected", Level::Info, json!({}));
        Ok(())
    }

    pub fn start_gog_authorization() -> Result<(), EmuBoxError> {
        operation("gog", "connect.open", gog::start_authorization)?;
        store_event("gog", "connection.opened", Level::Info, json!({}));
        Ok(())
    }

    pub fn complete_gog_authorization(code: String) -> Result<(), EmuBoxError> {
        operation("gog", "connect.complete", || {
            gog::complete_authorization(code)
        })?;
        store_event("gog", "connection.connected", Level::Info, json!({}));
        Ok(())
    }

    pub fn sync_gog_library() -> Result<StoreSyncResult, EmuBoxError> {
        let result = operation("gog", "sync", gog::sync_library)?;
        sync_event(&result);
        Ok(result)
    }

    pub fn disconnect_gog() -> Result<(), EmuBoxError> {
        operation("gog", "connect.disconnect", gog::disconnect)?;
        store_event("gog", "connection.disconnected", Level::Info, json!({}));
        Ok(())
    }

    pub fn start_steam_authorization() -> Result<(), EmuBoxError> {
        operation("steam", "connect.open", steam::start_authorization)?;
        store_event("steam", "connection.opened", Level::Info, json!({}));
        Ok(())
    }

    pub fn complete_steam_authorization(
        steam_id: String,
        api_key: String,
    ) -> Result<(), EmuBoxError> {
        operation("steam", "connect.complete", || {
            steam::complete_authorization(steam_id, api_key)
        })?;
        store_event("steam", "connection.connected", Level::Info, json!({}));
        Ok(())
    }

    pub fn sync_steam_library() -> Result<StoreSyncResult, EmuBoxError> {
        let result = operation("steam", "sync", steam::sync_library)?;
        sync_event(&result);
        Ok(result)
    }

    pub fn disconnect_steam() -> Result<(), EmuBoxError> {
        operation("steam", "connect.disconnect", steam::disconnect)?;
        store_event("steam", "connection.disconnected", Level::Info, json!({}));
        Ok(())
    }

    pub fn providers() -> Result<Vec<StoreProviderInfo>, EmuBoxError> {
        let connection = DatabaseService::get_connection()?;
        let authenticated = connection
            .prepare("SELECT DISTINCT provider FROM store_accounts WHERE status='authenticated'")
            .map_err(storage_error)?
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(storage_error)?
            .collect::<Result<BTreeSet<_>, _>>()
            .map_err(storage_error)?;
        let states = connection
            .prepare("SELECT provider,status,authorization_status,last_sync_at,error_message FROM store_provider_states")
            .map_err(storage_error)?
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    StoreSyncState {
                        status: row.get(1)?,
                        authorization_status: row.get(2)?,
                        last_sync_at: row.get(3)?,
                        error_message: row.get(4)?,
                    },
                ))
            })
            .map_err(storage_error)?
            .collect::<Result<BTreeMap<_, _>, _>>()
            .map_err(storage_error)?;
        let counts = connection
            .prepare("SELECT entitlement.provider,COUNT(DISTINCT entitlement.store_account_id),COUNT(*),COUNT(DISTINCT link.canonical_game_id) FROM store_entitlements AS entitlement LEFT JOIN store_game_links AS link ON link.provider=entitlement.provider AND link.external_game_id=entitlement.external_game_id GROUP BY entitlement.provider")
            .map_err(storage_error)?
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    (row.get::<_, u32>(1)?, row.get::<_, u32>(2)?, row.get::<_, u32>(3)?),
                ))
            })
            .map_err(storage_error)?
            .collect::<Result<BTreeMap<_, _>, _>>()
            .map_err(storage_error)?;
        Ok(PROVIDERS
            .iter()
            .map(|provider| {
                let (account_count, entitlement_count, linked_game_count) =
                    counts.get(provider.id()).copied().unwrap_or_default();
                StoreProviderInfo {
                    id: provider.id().into(),
                    name: provider.name().into(),
                    authenticated: authenticated.contains(provider.id()),
                    sync: states
                        .get(provider.id())
                        .cloned()
                        .unwrap_or(StoreSyncState {
                            status: "authorization_required".into(),
                            authorization_status: if authenticated.contains(provider.id()) {
                                "connected".into()
                            } else {
                                "required".into()
                            },
                            last_sync_at: None,
                            error_message: None,
                        }),
                    account_count,
                    entitlement_count,
                    linked_game_count,
                }
            })
            .collect())
    }

    pub fn session_directory(provider: &str) -> Result<PathBuf, EmuBoxError> {
        if !PROVIDERS.iter().any(|candidate| candidate.id() == provider) {
            return Err(EmuBoxError::InvalidConfiguration(
                "Proveedor de tienda no admitido".into(),
            ));
        }
        Ok(PathBuf::from(paths::stores_dir()).join(provider))
    }

    pub fn accounts() -> Result<Vec<StoreAccount>, EmuBoxError> {
        let connection = DatabaseService::get_connection()?;
        let mut statement = connection
            .prepare("SELECT id,provider,external_account_id,display_name,avatar_url,status,last_login_at,last_sync_at FROM store_accounts ORDER BY provider,display_name")
            .map_err(storage_error)?;
        let accounts = statement
            .query_map([], |row| {
                Ok(StoreAccount {
                    id: row.get(0)?,
                    provider: row.get(1)?,
                    external_account_id: row.get(2)?,
                    display_name: row.get(3)?,
                    avatar_url: row.get(4)?,
                    status: row.get(5)?,
                    last_login_at: row.get(6)?,
                    last_sync_at: row.get(7)?,
                })
            })
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        Ok(accounts)
    }

    pub fn entitlements(account_id: &str) -> Result<Vec<StoreEntitlement>, EmuBoxError> {
        let connection = DatabaseService::get_connection()?;
        let account = connection
            .query_row(
                "SELECT id FROM store_accounts WHERE id=?1",
                [account_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(storage_error)?;
        if account.is_none() {
            return Err(EmuBoxError::NotFound("Cuenta de tienda inexistente".into()));
        }
        let mut statement = connection
            .prepare("SELECT entitlement.store_account_id,entitlement.provider,entitlement.external_game_id,game.title,entitlement.owned,entitlement.installed,link.canonical_game_id,entitlement.last_seen_at FROM store_entitlements AS entitlement JOIN store_games AS game ON game.provider=entitlement.provider AND game.external_game_id=entitlement.external_game_id LEFT JOIN store_game_links AS link ON link.provider=entitlement.provider AND link.external_game_id=entitlement.external_game_id WHERE entitlement.store_account_id=?1 ORDER BY game.title")
            .map_err(storage_error)?;
        let entitlements = statement
            .query_map([account_id], |row| {
                Ok(StoreEntitlement {
                    account_id: row.get(0)?,
                    provider: row.get(1)?,
                    external_game_id: row.get(2)?,
                    title: row.get(3)?,
                    owned: row.get(4)?,
                    installed: row.get(5)?,
                    canonical_game_id: row.get(6)?,
                    last_seen_at: row.get(7)?,
                })
            })
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        Ok(entitlements)
    }
}

fn storage_error(error: rusqlite::Error) -> EmuBoxError {
    EmuBoxError::StorageUnavailable(error.to_string())
}

fn operation<T>(
    provider: &str,
    action: &str,
    execute: impl FnOnce() -> Result<T, EmuBoxError>,
) -> Result<T, EmuBoxError> {
    store_event(
        provider,
        "operation.started",
        Level::Info,
        json!({"action": action}),
    );
    match execute() {
        Ok(result) => Ok(result),
        Err(error) => {
            store_event(
                provider,
                "operation.failed",
                Level::Error,
                json!({"action": action}),
            );
            Err(error)
        }
    }
}

fn sync_event(result: &StoreSyncResult) {
    store_event(
        &result.provider,
        "sync.completed",
        Level::Info,
        json!({"importedGames": result.imported_games, "installedGames": result.installed_games}),
    );
}

fn store_event(provider: &str, event: &str, level: Level, data: serde_json::Value) {
    telemetry::event(
        level,
        "stores",
        event,
        "Operacion de proveedor de tienda",
        json!({"provider": provider, "result": data}),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn providers_use_fixed_private_directories() {
        assert!(StoreService::session_directory("../steam").is_err());
        assert_eq!(
            StoreService::session_directory("steam").unwrap(),
            PathBuf::from(paths::stores_dir()).join("steam")
        );
        assert_eq!(
            StoreService::providers()
                .unwrap()
                .iter()
                .map(|provider| provider.id.as_str())
                .collect::<Vec<_>>(),
            ["steam", "epic", "gog"]
        );
    }

    #[test]
    fn store_schema_keeps_credentials_out_of_sqlite() {
        let connection = DatabaseService::get_connection().unwrap();
        let columns = connection
            .prepare("PRAGMA table_info(store_accounts)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        for forbidden in ["password", "token", "cookie", "secret"] {
            assert!(!columns.iter().any(|column| column.contains(forbidden)));
        }
    }

    #[test]
    fn provider_summary_and_entitlement_keep_canonical_links_explicit() {
        let connection = DatabaseService::get_connection().unwrap();
        connection.execute("INSERT OR IGNORE INTO systems(id,name,short_name,manufacturer,generation,release_year,color,icon,default_emulator_id,extensions_json) VALUES ('store-test','Store test','Store','EmuBox',0,2026,'#000000','store','', '[]')", []).unwrap();
        connection.execute("INSERT OR IGNORE INTO canonical_games(id,authority,authority_id,title,normalized_title,platform_id,updated_at) VALUES ('store-canonical','fixture','store-canonical','Store fixture','store fixture','store-test',0)", []).unwrap();
        connection.execute("INSERT OR REPLACE INTO store_accounts(id,provider,external_account_id,display_name,status) VALUES ('store-account','steam','fixture-account','Fixture account','authenticated')", []).unwrap();
        connection.execute("INSERT OR REPLACE INTO store_games(provider,external_game_id,title,updated_at) VALUES ('steam','fixture-game','Fixture game',0)", []).unwrap();
        connection.execute("INSERT OR REPLACE INTO store_entitlements(store_account_id,provider,external_game_id,owned,installed,last_seen_at) VALUES ('store-account','steam','fixture-game',1,0,0)", []).unwrap();
        connection.execute("INSERT OR REPLACE INTO store_game_links(provider,external_game_id,canonical_game_id) VALUES ('steam','fixture-game','store-canonical')", []).unwrap();
        connection.execute("UPDATE store_provider_states SET status='authorization_required',authorization_status='connected',error_message=NULL WHERE provider='steam'", []).unwrap();

        let steam = StoreService::providers()
            .unwrap()
            .into_iter()
            .find(|provider| provider.id == "steam")
            .unwrap();
        assert!(steam.authenticated);
        assert!(steam.account_count >= 1);
        assert!(steam.entitlement_count >= 1);
        assert!(steam.linked_game_count >= 1);
        assert_eq!(steam.sync.status, "authorization_required");
        assert_eq!(steam.sync.authorization_status, "connected");

        let entitlement = StoreService::entitlements("store-account")
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(
            entitlement.canonical_game_id.as_deref(),
            Some("store-canonical")
        );
    }
}
