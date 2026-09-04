use crate::models::{Game, Platform, GameFilter, ScanGamesRequest, ScanGamesResult};
use crate::services::GameService;
use crate::errors::EmuBoxError;

#[tauri::command]
pub fn get_games(filter: Option<GameFilter>) -> Result<Vec<Game>, EmuBoxError> {
    let result = GameService::get_games(filter);
    match &result {
        Ok(games) => eprintln!("[IPC] get_games -> {} juegos", games.len()),
        Err(e) => eprintln!("[IPC] get_games FALLÓ: {}", e),
    }
    result
}

#[tauri::command]
pub fn get_game_by_id(id: String) -> Result<Option<Game>, EmuBoxError> {
    GameService::get_game_by_id(id)
}

#[tauri::command]
pub fn scan_games(request: Option<ScanGamesRequest>) -> Result<ScanGamesResult, EmuBoxError> {
    let result = GameService::scan_games(request);
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
pub fn get_platforms() -> Result<Vec<Platform>, EmuBoxError> {
    let result = GameService::get_platforms();
    if let Err(e) = &result {
        eprintln!("[IPC] get_platforms FALLÓ: {}", e);
    }
    result
}

#[tauri::command]
pub fn toggle_favorite(game_id: String) -> Result<bool, EmuBoxError> {
    GameService::toggle_favorite(game_id)
}
