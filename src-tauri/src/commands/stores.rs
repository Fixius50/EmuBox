use crate::{
    errors::EmuBoxError,
    models::{StoreAccount, StoreEntitlement, StoreProviderInfo},
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
