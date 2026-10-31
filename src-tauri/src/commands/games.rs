use crate::errors::EmuBoxError;
use crate::models::{Game, GameFilter, Platform, ScanGamesRequest, ScanGamesResult};
use crate::services::GameService;

#[tauri::command]
pub async fn get_games(filter: Option<GameFilter>) -> Result<Vec<Game>, EmuBoxError> {
    let result = super::blocking(move || {
        crate::services::infrastructure::telemetry::operation("library.read", "get-games", || {
            GameService::get_games(filter)
        })
    })
    .await;
    match &result {
        Ok(games) => eprintln!("[IPC] get_games -> {} juegos", games.len()),
        Err(e) => eprintln!("[IPC] get_games FALLÓ: {}", e),
    }
    result
}

#[tauri::command]
pub async fn get_game_by_id(id: String) -> Result<Option<Game>, EmuBoxError> {
    super::blocking(move || GameService::get_game_by_id(id)).await
}

#[tauri::command]
pub async fn get_canonical_game_options(
    id: String,
) -> Result<crate::models::CanonicalGameOptions, EmuBoxError> {
    super::blocking(move || {
        crate::services::infrastructure::telemetry::operation(
            "library.options",
            "canonical-options",
            || crate::services::game_database::options(&id),
        )
    })
    .await
}

#[tauri::command]
pub async fn scan_games(request: Option<ScanGamesRequest>) -> Result<ScanGamesResult, EmuBoxError> {
    let result = super::blocking(move || {
        crate::services::infrastructure::telemetry::operation(
            "library.scan",
            "requested-scan",
            || GameService::scan_games(request),
        )
    })
    .await;
    match &result {
        Ok(r) => eprintln!(
            "[IPC] scan_games -> total={} añadidos={} actualizados={} eliminados={} errores={:?}",
            r.total_count, r.added_count, r.updated_count, r.removed_count, r.errors
        ),
        Err(e) => eprintln!("[IPC] scan_games FALLÓ: {}", e),
    }
    result
}

#[tauri::command]
pub async fn get_platforms() -> Result<Vec<Platform>, EmuBoxError> {
    let result = super::blocking(GameService::get_platforms).await;
    if let Err(e) = &result {
        eprintln!("[IPC] get_platforms FALLÓ: {}", e);
    }
    result
}

#[tauri::command]
pub async fn toggle_favorite(game_id: String) -> Result<bool, EmuBoxError> {
    super::blocking(move || GameService::toggle_favorite(game_id)).await
}
