use crate::models::{Game, Platform, GameFilter, ScanGamesRequest, ScanGamesResult};
use crate::errors::EmuBoxError;

pub struct GameService;

impl GameService {
    pub fn get_games(_filter: Option<GameFilter>) -> Result<Vec<Game>, EmuBoxError> {
        Ok(vec![])
    }

    pub fn get_game_by_id(_id: String) -> Result<Option<Game>, EmuBoxError> {
        Ok(None)
    }

    pub fn scan_games(_request: Option<ScanGamesRequest>) -> Result<ScanGamesResult, EmuBoxError> {
        Ok(ScanGamesResult {
            scanned_count: 0,
            added_count: 0,
            updated_count: 0,
            errors: vec![],
        })
    }

    pub fn get_platforms() -> Result<Vec<Platform>, EmuBoxError> {
        Ok(vec![])
    }

    pub fn toggle_favorite(_game_id: String) -> Result<bool, EmuBoxError> {
        Ok(true)
    }
}
