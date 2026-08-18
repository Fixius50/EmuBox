use crate::models::{LaunchGameRequest, LaunchResult, RunningGameInfo, ProcessStatus};
use crate::errors::EmuBoxError;

pub struct ProcessService;

impl ProcessService {
    pub fn launch_game(_request: LaunchGameRequest) -> Result<LaunchResult, EmuBoxError> {
        Ok(LaunchResult {
            success: true,
            message: "Simulado".to_string(),
            pid: Some(1234),
            executable: Some("retroarch".to_string()),
            start_time: Some(1700000000),
        })
    }

    pub fn stop_game() -> Result<(), EmuBoxError> {
        Ok(())
    }

    pub fn is_game_running() -> Result<bool, EmuBoxError> {
        Ok(false)
    }

    pub fn get_running_game() -> Result<Option<RunningGameInfo>, EmuBoxError> {
        Ok(None)
    }

    pub fn get_process_status() -> Result<ProcessStatus, EmuBoxError> {
        Ok(ProcessStatus {
            has_active_game: false,
            running_game: None,
            active_child_pids: vec![],
        })
    }

    pub fn kill_process(_pid: u32) -> Result<bool, EmuBoxError> {
        Ok(true)
    }
}
