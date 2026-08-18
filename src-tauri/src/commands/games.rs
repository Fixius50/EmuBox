use crate::models::{Game, Platform, GameFilter, ScanGamesRequest, ScanGamesResult};
use crate::services::GameService;
use crate::errors::EmuBoxError;

#[tauri::command]
pub fn get_games(filter: Option<GameFilter>) -> Result<Vec<Game>, EmuBoxError> {
    GameService::get_games(filter)
}

#[tauri::command]
pub fn get_game_by_id(id: String) -> Result<Option<Game>, EmuBoxError> {
    GameService::get_game_by_id(id)
}

#[tauri::command]
pub fn scan_games(request: Option<ScanGamesRequest>) -> Result<ScanGamesResult, EmuBoxError> {
    GameService::scan_games(request)
}

#[tauri::command]
pub fn get_platforms() -> Result<Vec<Platform>, EmuBoxError> {
    GameService::get_platforms()
}

#[tauri::command]
pub fn toggle_favorite(game_id: String) -> Result<bool, EmuBoxError> {
    GameService::toggle_favorite(game_id)
}
