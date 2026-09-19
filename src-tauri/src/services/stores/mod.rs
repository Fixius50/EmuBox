use crate::{
    errors::EmuBoxError,
    models::{StoreAccount, StoreEntitlement, StoreProviderInfo},
    services::{db_service::DatabaseService, paths},
};
use rusqlite::OptionalExtension;
use std::{collections::BTreeSet, path::PathBuf};

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
    pub fn providers() -> Result<Vec<StoreProviderInfo>, EmuBoxError> {
        let connection = DatabaseService::get_connection()?;
        let authenticated = connection
            .prepare("SELECT DISTINCT provider FROM store_accounts WHERE status='authenticated'")
            .map_err(storage_error)?
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(storage_error)?
            .collect::<Result<BTreeSet<_>, _>>()
            .map_err(storage_error)?;
        Ok(PROVIDERS
            .iter()
            .map(|provider| {
                StoreProviderInfo {
                    id: provider.id().into(),
                    name: provider.name().into(),
                    authenticated: authenticated.contains(provider.id()),
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
}
