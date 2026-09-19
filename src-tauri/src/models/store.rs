use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreProviderInfo {
    pub id: String,
    pub name: String,
    pub authenticated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreAccount {
    pub id: String,
    pub provider: String,
    pub external_account_id: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub status: String,
    pub last_login_at: Option<i64>,
    pub last_sync_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreEntitlement {
    pub account_id: String,
    pub provider: String,
    pub external_game_id: String,
    pub title: String,
    pub owned: bool,
    pub installed: bool,
    pub canonical_game_id: Option<String>,
    pub last_seen_at: i64,
}
