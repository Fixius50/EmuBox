use super::{game_sandbox, launch_policy};
use crate::errors::EmuBoxError;
use crate::models::{LaunchGameRequest, LaunchResult, ProcessStatus, RunningGameInfo};
use crate::services::compatibility_service::CompatibilityService;
use crate::services::game_service::GameService;
use std::process::Child;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

struct RunningGame {
    info: RunningGameInfo,
    child: Child,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unrelated_process_cannot_be_stopped_over_ipc() {
        assert!(ProcessService::kill_process(std::process::id()).is_err());
    }
}

static CURRENT_RUNNING_GAME: Mutex<Option<RunningGame>> = Mutex::new(None);

pub struct ProcessService;

impl ProcessService {
    fn now_epoch_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    pub fn launch_game(request: LaunchGameRequest) -> Result<LaunchResult, EmuBoxError> {
        let mut current = CURRENT_RUNNING_GAME
            .lock()
            .map_err(|error| launch_policy::failure(error.to_string()))?;
        if let Some(running) = current.as_mut() {
            if running
                .child
                .try_wait()
                .map_err(|error| launch_policy::failure(error.to_string()))?
                .is_none()
            {
                let info = &running.info;
                return Ok(LaunchResult {
                    success: false,
                    message: format!(
                        "Ya hay un juego en ejecución: {} (PID: {})",
                        info.game_title, info.pid
                    ),
                    pid: Some(info.pid),
                    executable: Some(info.executable.clone()),
                    start_time: Some(info.start_time),
                });
            }
            *current = None;
        }

        // 2. Obtener metadatos del juego
        let game = GameService::get_game_by_id(request.game_id.clone())?.ok_or_else(|| {
            EmuBoxError::NotFound(format!("Juego no encontrado: {}", request.game_id))
        })?;

        // 3. Obtener metadatos del emulador solicitado (o resolver emulador por defecto de la plataforma)
        let requested_id = request.emulator_id.trim();
        let (emulator, association_args, association_config) =
            CompatibilityService::resolve_for_game(
                &game,
                (!requested_id.is_empty()).then_some(requested_id),
            )?;

        launch_policy::validate_request(
            &request,
            &association_args,
            association_config.as_deref(),
        )?;
        let policy = launch_policy::resolve(&emulator.id, &game.platform)?;
        let executable_path = policy.executable.to_string_lossy().into_owned();
        let mut command = game_sandbox::command(
            &game,
            &emulator.id,
            &policy,
            request.use_gamescope.unwrap_or(false),
        )?;
        let final_args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        let child = game_sandbox::spawn(&mut command)?;

        let pid = child.id();
        let start_time = Self::now_epoch_secs();

        let running_info = RunningGameInfo {
            pid,
            game_id: game.id.clone(),
            game_title: game.title.clone(),
            platform_id: game.platform.clone(),
            emulator_id: emulator.id.clone(),
            emulator_name: emulator.name.clone(),
            executable: executable_path.clone(),
            arguments: final_args,
            start_time,
            cpu_percent: None,
            memory_mb: None,
            status: "running".to_string(),
        };

        *current = Some(RunningGame {
            info: running_info,
            child,
        });

        Ok(LaunchResult {
            success: true,
            message: format!(
                "{} iniciado correctamente con {}",
                game.title, emulator.name
            ),
            pid: Some(pid),
            executable: Some(executable_path),
            start_time: Some(start_time),
        })
    }

    pub fn stop_game() -> Result<(), EmuBoxError> {
        let mut current = CURRENT_RUNNING_GAME
            .lock()
            .map_err(|error| launch_policy::failure(error.to_string()))?;
        if let Some(running) = current.as_mut() {
            Self::terminate(running)?;
        }
        *current = None;
        Ok(())
    }

    pub fn is_game_running() -> Result<bool, EmuBoxError> {
        Ok(Self::get_running_game()?.is_some())
    }

    pub fn get_running_game() -> Result<Option<RunningGameInfo>, EmuBoxError> {
        let mut current = CURRENT_RUNNING_GAME
            .lock()
            .map_err(|error| launch_policy::failure(error.to_string()))?;
        if let Some(running) = current.as_mut() {
            if running
                .child
                .try_wait()
                .map_err(|error| launch_policy::failure(error.to_string()))?
                .is_none()
            {
                return Ok(Some(running.info.clone()));
            }
        }
        *current = None;
        Ok(None)
    }

    pub fn get_process_status() -> Result<ProcessStatus, EmuBoxError> {
        let running_game = Self::get_running_game()?;
        let has_active_game = running_game.is_some();
        let active_child_pids = running_game
            .as_ref()
            .map(|g| vec![g.pid])
            .unwrap_or_default();

        Ok(ProcessStatus {
            has_active_game,
            running_game,
            active_child_pids,
        })
    }

    pub fn kill_process(pid: u32) -> Result<bool, EmuBoxError> {
        let mut current = CURRENT_RUNNING_GAME
            .lock()
            .map_err(|error| launch_policy::failure(error.to_string()))?;
        let running = current
            .as_mut()
            .filter(|running| running.child.id() == pid)
            .ok_or_else(|| {
                launch_policy::failure("Solo se puede detener el sandbox de juego activo")
            })?;
        Self::terminate(running)?;
        *current = None;
        Ok(true)
    }

    fn terminate(running: &mut RunningGame) -> Result<(), EmuBoxError> {
        if running
            .child
            .try_wait()
            .map_err(|error| launch_policy::failure(error.to_string()))?
            .is_none()
        {
            running
                .child
                .kill()
                .map_err(|error| launch_policy::failure(error.to_string()))?;
        }
        running
            .child
            .wait()
            .map_err(|error| launch_policy::failure(error.to_string()))?;
        Ok(())
    }
}
