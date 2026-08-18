use crate::models::{LaunchGameRequest, LaunchResult, RunningGameInfo, ProcessStatus};
use crate::services::ProcessService;
use crate::errors::EmuBoxError;

#[tauri::command]
pub fn launch_game(request: LaunchGameRequest) -> Result<LaunchResult, EmuBoxError> {
    ProcessService::launch_game(request)
}

#[tauri::command]
pub fn stop_game() -> Result<(), EmuBoxError> {
    ProcessService::stop_game()
}

#[tauri::command]
pub fn is_game_running() -> Result<bool, EmuBoxError> {
    ProcessService::is_game_running()
}

#[tauri::command]
pub fn get_running_game() -> Result<Option<RunningGameInfo>, EmuBoxError> {
    ProcessService::get_running_game()
}

#[tauri::command]
pub fn get_process_status() -> Result<ProcessStatus, EmuBoxError> {
    ProcessService::get_process_status()
}

#[tauri::command]
pub fn kill_process(pid: u32) -> Result<bool, EmuBoxError> {
    ProcessService::kill_process(pid)
}
