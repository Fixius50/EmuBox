use crate::{
    errors::EmuBoxError,
    models::{StoreAccount, StoreEntitlement, StoreProviderInfo, StoreSyncResult},
    services::StoreService,
};

#[tauri::command]
pub fn get_store_providers() -> Result<Vec<StoreProviderInfo>, EmuBoxError> {
    StoreService::providers()
}

#[tauri::command]
pub fn get_store_accounts() -> Result<Vec<StoreAccount>, EmuBoxError> {
    StoreService::accounts()
}

#[tauri::command]
pub fn get_store_entitlements(account_id: String) -> Result<Vec<StoreEntitlement>, EmuBoxError> {
    StoreService::entitlements(&account_id)
}

#[tauri::command]
pub async fn start_epic_authorization() -> Result<(), EmuBoxError> {
    super::blocking(StoreService::start_epic_authorization).await
}

#[tauri::command]
pub async fn sync_epic_library() -> Result<StoreSyncResult, EmuBoxError> {
    super::blocking(StoreService::sync_epic_library).await
}

#[tauri::command]
pub async fn disconnect_epic() -> Result<(), EmuBoxError> {
    super::blocking(StoreService::disconnect_epic).await
}

#[tauri::command]
pub async fn start_gog_authorization() -> Result<(), EmuBoxError> {
    super::blocking(StoreService::start_gog_authorization).await
}

#[tauri::command]
pub async fn sync_gog_library() -> Result<StoreSyncResult, EmuBoxError> {
    super::blocking(StoreService::sync_gog_library).await
}

#[tauri::command]
pub async fn disconnect_gog() -> Result<(), EmuBoxError> {
    super::blocking(StoreService::disconnect_gog).await
}

#[tauri::command]
pub async fn start_steam_authorization() -> Result<(), EmuBoxError> {
    super::blocking(StoreService::start_steam_authorization).await
}

#[tauri::command]
pub async fn sync_steam_library() -> Result<StoreSyncResult, EmuBoxError> {
    super::blocking(StoreService::sync_steam_library).await
}

#[tauri::command]
pub async fn disconnect_steam() -> Result<(), EmuBoxError> {
    super::blocking(StoreService::disconnect_steam).await
}
