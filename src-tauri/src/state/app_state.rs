use crate::models::{EmuBoxConfig, RunningGameInfo};
use std::process::Child;
use std::sync::Mutex;

pub struct AppState {
    pub config: Mutex<Option<EmuBoxConfig>>,
    pub running_game: Mutex<Option<RunningGameInfo>>,
    pub active_process: Mutex<Option<Child>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: Mutex::new(None),
            running_game: Mutex::new(None),
            active_process: Mutex::new(None),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
