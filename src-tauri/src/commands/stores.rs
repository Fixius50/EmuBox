use crate::{
    errors::EmuBoxError,
    models::{StoreAccount, StoreEntitlement, StoreProviderInfo, StoreSyncResult},
    services::StoreService,
};
use tauri::Manager;

fn store_url_allowed(provider: &str, url: &tauri::Url) -> bool {
    if url.scheme() != "https" { return false; }
    let Some(host) = url.host_str() else { return false; };
    let owned_domain = match provider {
        "epic" if host == "legendary.gl" => return true,
        "epic" => "epicgames.com",
        "gog" => "gog.com",
        "steam" => return ["steamcommunity.com", "steampowered.com"]
            .iter()
            .any(|domain| host == *domain || host.ends_with(&format!(".{domain}"))),
        _ => return false,
    };
    host == owned_domain || host.ends_with(&format!(".{owned_domain}"))
}

fn open_store_window(app: &tauri::AppHandle, provider: &'static str, target: &str) -> Result<(), EmuBoxError> {
    let url = tauri::Url::parse(target)
        .map_err(|_| EmuBoxError::InvalidConfiguration("URL de tienda invalida".into()))?;
    if !store_url_allowed(provider, &url) {
        return Err(EmuBoxError::InvalidConfiguration("Origen de autorizacion no admitido".into()));
    }
    let label = format!("store-{provider}");
    if let Some(window) = app.get_webview_window(&label) {
        return window.set_focus().map_err(|error| EmuBoxError::ProcessFailed(error.to_string()));
    }
    let title = match provider {
        "epic" => "Epic Games",
        "gog" => "GOG",
        "steam" => "Steam",
        _ => return Err(EmuBoxError::InvalidConfiguration("Tienda desconocida".into())),
    };
    tauri::WebviewWindowBuilder::new(app, label, tauri::WebviewUrl::External(url))
        .title(title)
        .inner_size(1200.0, 800.0)
        .on_navigation(move |url| store_url_allowed(provider, url))
        .on_new_window(|_, _| tauri::webview::NewWindowResponse::Deny)
        .build()
        .map_err(|error| EmuBoxError::ProcessFailed(error.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod webview_tests {
    use super::store_url_allowed;

    #[test]
    fn store_windows_stay_on_their_own_https_domains() {
        for (provider, url) in [
            ("epic", "https://legendary.gl/epiclogin"),
            ("epic", "https://www.epicgames.com/id/login"),
            ("gog", "https://auth.gog.com/auth?client_id=public"),
            ("steam", "https://steamcommunity.com/dev/apikey"),
            ("steam", "https://store.steampowered.com/login"),
        ] {
            assert!(store_url_allowed(provider, &tauri::Url::parse(url).unwrap()));
        }
        for (provider, url) in [
            ("epic", "http://www.epicgames.com/id/login"),
            ("epic", "https://epicgames.com.evil.test"),
            ("gog", "https://auth.gog.com.evil.test"),
            ("steam", "https://steamcommunity.com@evil.test"),
            ("gog", "https://steamcommunity.com"),
        ] {
            assert!(!store_url_allowed(provider, &tauri::Url::parse(url).unwrap()));
        }
    }
}

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
pub async fn start_epic_authorization(app: tauri::AppHandle) -> Result<(), EmuBoxError> {
    super::blocking(StoreService::start_epic_authorization).await?;
    open_store_window(&app, "epic", "https://legendary.gl/epiclogin")
}

#[tauri::command]
pub async fn complete_epic_authorization(code: String) -> Result<(), EmuBoxError> {
    super::blocking(move || StoreService::complete_epic_authorization(code)).await
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
pub async fn start_gog_authorization(app: tauri::AppHandle) -> Result<(), EmuBoxError> {
    let url = super::blocking(StoreService::start_gog_authorization).await?;
    open_store_window(&app, "gog", &url)
}

#[tauri::command]
pub async fn complete_gog_authorization(code: String) -> Result<(), EmuBoxError> {
    super::blocking(move || StoreService::complete_gog_authorization(code)).await
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
pub async fn start_steam_authorization(app: tauri::AppHandle) -> Result<(), EmuBoxError> {
    super::blocking(StoreService::start_steam_authorization).await?;
    open_store_window(&app, "steam", "https://steamcommunity.com/dev/apikey")
}

#[tauri::command]
pub async fn complete_steam_authorization(
    steam_id: String,
    api_key: String,
) -> Result<(), EmuBoxError> {
    super::blocking(move || StoreService::complete_steam_authorization(steam_id, api_key)).await
}

#[tauri::command]
pub async fn sync_steam_library() -> Result<StoreSyncResult, EmuBoxError> {
    super::blocking(StoreService::sync_steam_library).await
}

#[tauri::command]
pub async fn disconnect_steam() -> Result<(), EmuBoxError> {
    super::blocking(StoreService::disconnect_steam).await
}
