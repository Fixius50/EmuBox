use crate::errors::EmuBoxError;
use crate::models::GameEmulatorAssociation;
use crate::services::CompatibilityService;

#[tauri::command]
pub fn get_game_associations(game_id: String) -> Result<Vec<GameEmulatorAssociation>, EmuBoxError> {
    CompatibilityService::get_game_associations(game_id)
}

#[tauri::command]
pub fn set_game_association(association: GameEmulatorAssociation) -> Result<(), EmuBoxError> {
    CompatibilityService::set_game_association(association)
}

#[tauri::command]
pub fn remove_game_association(game_id: String, emulator_id: String) -> Result<(), EmuBoxError> {
    CompatibilityService::remove_game_association(game_id, emulator_id)
}
